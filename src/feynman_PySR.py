"""
feynman_PySR.py
======================================================
Feynman 方程式のシンボリック回帰 - PySR 版
  - N_SAMPLES  : 750   (EML-SR 実験と同一)
  - niterations: 100 (本番) / 40 (--smoke)
  - --smoke: first_AI ok 式のうち CSV 順先頭 3 式
  - first_AI / cursor ベースラインとの比較指標を出力
======================================================
"""

import sys
import os
import json
import time
import numpy as np
import pandas as pd

# ===== パス設定 =====
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
sys.path.insert(0, os.path.join(PROJECT_ROOT, "texts"))

from PySR_feynman_report import write_report  # noqa: E402

try:
    import pysr
    from pysr import PySRRegressor
except ImportError as exc:
    print("[ERROR] PySR is not installed. Run: pip install pysr")
    raise SystemExit(1) from exc

CSV_PATH = os.path.join(PROJECT_ROOT, "data", "FeynmanEquations.csv")
RESULTS_PATH = os.path.join(PROJECT_ROOT, "results", "PySR_feynman_results.json")
REPORT_PATH = os.path.join(PROJECT_ROOT, "texts", "PySR_feynman_report.md")
BASELINE_FIRST_AI_PATH = os.path.join(
    PROJECT_ROOT, "results", "eml_sr_model_first_AI_feynman_results.json"
)
BASELINE_CURSOR_PATH = os.path.join(
    PROJECT_ROOT, "results", "eml_sr_model_cursor_feynman_results.json"
)

# ===== 実験設定 =====
N_SAMPLES = 750
RMSE_THRESHOLD = 1e-4
RMSE_PARTIAL = 1e-2
SMOKE_OK_COUNT = 3

POPULATIONS = 24
POPULATION_SIZE = 33
MAXSIZE = 20
NITERATIONS_FULL = 100
NITERATIONS_SMOKE = 40

BINARY_OPERATORS = ["+", "-", "*", "/"]
UNARY_OPERATORS = ["exp", "log", "sin", "cos", "tan", "sqrt", "abs"]
NESTED_CONSTRAINTS = {
    "sin": {"sin": 0, "cos": 0, "tan": 0, "exp": 0, "log": 0},
    "cos": {"sin": 0, "cos": 0, "tan": 0, "exp": 0, "log": 0},
    "tan": {"sin": 0, "cos": 0, "tan": 0, "exp": 0, "log": 0},
    "exp": {"exp": 0, "log": 0},
    "log": {"exp": 0, "log": 0},
}


# ------------------------------------------------------------------
# ユーティリティ
# ------------------------------------------------------------------

def _build_eval_env():
    return {
        "exp": np.exp, "sqrt": np.sqrt,
        "sin": np.sin, "cos": np.cos, "tan": np.tan,
        "arcsin": np.arcsin, "arccos": np.arccos, "arctan": np.arctan,
        "log": np.log, "ln": np.log,
        "abs": np.abs, "tanh": np.tanh,
        "pi": np.pi, "e": np.e,
        "__builtins__": None,
    }


def load_equations(csv_path):
    df = pd.read_csv(csv_path)
    df = df.dropna(subset=["Filename", "Formula"])
    df = df[df["Filename"].str.strip() != ""]

    equations = []
    for _, row in df.iterrows():
        filename = str(row["Filename"]).strip()
        formula = str(row["Formula"]).strip()
        output_var = str(row["Output"]).strip()
        n_vars = int(row["# variables"])

        var_names, var_ranges = [], []
        for i in range(1, n_vars + 1):
            nc, lc, hc = f"v{i}_name", f"v{i}_low", f"v{i}_high"
            if nc in row and pd.notna(row[nc]):
                var_names.append(str(row[nc]).strip())
                var_ranges.append((float(row[lc]), float(row[hc])))

        if len(var_names) != n_vars:
            continue

        equations.append({
            "filename": filename, "formula": formula,
            "output_var": output_var, "n_vars": n_vars,
            "var_names": var_names, "var_ranges": var_ranges,
        })
    return equations


def generate_dataset(eq, n_samples, seed=42):
    rng = np.random.default_rng(seed)
    base_env = _build_eval_env()

    X_list, y_list = [], []
    max_attempts = n_samples * 6
    attempt = 0

    while len(y_list) < n_samples and attempt < max_attempts:
        attempt += 1
        point = []
        local_env = base_env.copy()
        for name, (low, high) in zip(eq["var_names"], eq["var_ranges"]):
            val = rng.uniform(low, high)
            point.append(val)
            local_env[name] = val
        try:
            y = float(eval(eq["formula"], {"__builtins__": None}, local_env))
            if np.isfinite(y):
                X_list.append(point)
                y_list.append(y)
        except Exception:
            continue

    return np.array(X_list, dtype=float), np.array(y_list, dtype=float)


def rmse(y_true, y_pred):
    return float(np.sqrt(np.mean((np.asarray(y_true) - np.asarray(y_pred)) ** 2)))


def load_baseline_ok_ids(path):
    if not os.path.isfile(path):
        return set()
    with open(path, encoding="utf-8") as f:
        data = json.load(f)
    return {
        r["eq_id"]
        for r in data.get("results", [])
        if r.get("status") == "ok"
    }


def compare_with_baseline(pysr_ok_ids, baseline_ok_ids, attempted_ids=None):
    pysr_ok = set(pysr_ok_ids)
    baseline_ok = set(baseline_ok_ids)
    if attempted_ids is not None:
        attempted = set(attempted_ids)
        baseline_ok = baseline_ok & attempted
    return {
        "baseline_ok_count": len(baseline_ok),
        "delta_ok": len(pysr_ok) - len(baseline_ok),
        "new_ok_ids": sorted(pysr_ok - baseline_ok),
        "lost_ok_ids": sorted(baseline_ok - pysr_ok),
    }


def _make_pysr_model(niterations):
    return PySRRegressor(
        niterations=niterations,
        populations=POPULATIONS if niterations > NITERATIONS_SMOKE else 15,
        population_size=POPULATION_SIZE,
        maxsize=MAXSIZE,
        binary_operators=BINARY_OPERATORS,
        unary_operators=UNARY_OPERATORS,
        nested_constraints=NESTED_CONSTRAINTS,
        random_state=42,
        verbosity=0,
        parallelism="serial",
        deterministic=True,
        temp_equation_file=True,
        update_verbosity=False,
    )


def _extract_candidates(model, X_train, y_train):
    """PySR equations_ DataFrame から候補リストを構築する。"""
    cand_infos = []
    eq_df = getattr(model, "equations_", None)
    if eq_df is None or len(eq_df) == 0:
        return cand_infos

    for _, row in eq_df.iterrows():
        formula = str(row.get("equation", "N/A"))
        python_expr = str(row.get("lambda_format", formula))
        complexity = int(row["complexity"]) if pd.notna(row.get("complexity")) else None
        loss = row.get("loss")
        cand_rmse = None
        if loss is not None and pd.notna(loss) and float(loss) >= 0:
            cand_rmse = float(np.sqrt(float(loss)))

        # 学習データ上で RMSE を再計算（より正確な評価）
        try:
            idx = int(row.name) if hasattr(row, "name") else None
            if idx is not None and hasattr(model, "predict"):
                preds = model.predict(X_train, index=idx)
                if np.all(np.isfinite(preds)):
                    cand_rmse = rmse(y_train, preds)
        except Exception:
            pass

        cand_infos.append({
            "formula": formula,
            "python": python_expr,
            "complexity": complexity,
            "rmse": cand_rmse,
            "loss": float(loss) if loss is not None and pd.notna(loss) else None,
        })
    return cand_infos


def run_pysr(eq_id, X_train, y_train, niterations):
    """PySR を実行して結果 dict を返す。"""
    base = {
        "eq_id": eq_id, "status": "failed",
        "found_formula": None, "found_python": None,
        "rmse": None, "complexity": None,
        "elapsed_s": 0.0, "candidates": [],
        "all_candidates": [],
    }
    try:
        model = _make_pysr_model(niterations)
        start_t = time.time()
        model.fit(X_train, y_train)
        elapsed = time.time() - start_t
        base["elapsed_s"] = elapsed

        cand_infos = _extract_candidates(model, X_train, y_train)
        base["all_candidates"] = cand_infos
        base["candidates"] = cand_infos

        if not cand_infos:
            base["status"] = "no_candidates"
            return base

        try:
            y_pred = model.predict(X_train)
            best_rmse_val = rmse(y_train, y_pred) if np.all(np.isfinite(y_pred)) else float("inf")
        except Exception:
            best_rmse_val = float("inf")

        # equations_ の loss 最小行を最良候補として採用
        best_cand = min(
            cand_infos,
            key=lambda c: c["rmse"] if c["rmse"] is not None else float("inf"),
        )

        if best_cand["rmse"] is not None and best_cand["rmse"] < best_rmse_val:
            best_rmse_val = best_cand["rmse"]

        base["found_formula"] = best_cand["formula"]
        base["found_python"] = best_cand["python"]
        base["complexity"] = best_cand["complexity"]
        base["rmse"] = best_rmse_val if np.isfinite(best_rmse_val) else None

        try:
            sympy_eq = model.sympy()
            if sympy_eq is not None:
                base["found_formula"] = str(sympy_eq)
        except Exception:
            pass

        if not np.isfinite(best_rmse_val):
            base["status"] = "prediction_failed"
            return base

        if best_rmse_val < RMSE_THRESHOLD:
            base["status"] = "ok"
        elif best_rmse_val < RMSE_PARTIAL:
            base["status"] = "partial"
        else:
            base["status"] = "failed"

        return base

    except Exception as exc:
        base["status"] = "error"
        base["error_msg"] = str(exc)
        return base


def _build_settings(niterations):
    return {
        "N_SAMPLES": N_SAMPLES,
        "NITERATIONS": niterations,
        "POPULATIONS": POPULATIONS if niterations > NITERATIONS_SMOKE else 15,
        "POPULATION_SIZE": POPULATION_SIZE,
        "MAXSIZE": MAXSIZE,
        "BINARY_OPERATORS": ", ".join(BINARY_OPERATORS),
        "UNARY_OPERATORS": ", ".join(UNARY_OPERATORS),
        "RMSE_THRESHOLD": RMSE_THRESHOLD,
        "RMSE_PARTIAL": RMSE_PARTIAL,
        "PYSR_VERSION": getattr(pysr, "__version__", "unknown"),
    }


def main():
    limit = None
    smoke_mode = "--smoke" in sys.argv
    results_path = RESULTS_PATH
    report_path = REPORT_PATH
    niterations = NITERATIONS_SMOKE if smoke_mode else NITERATIONS_FULL

    for arg in sys.argv[1:]:
        if arg == "--smoke":
            continue
        try:
            limit = int(arg)
        except ValueError:
            pass

    if smoke_mode:
        results_path = os.path.join(
            PROJECT_ROOT, "results", "PySR_feynman_smoke_results.json"
        )
        report_path = os.path.join(
            PROJECT_ROOT, "texts", "PySR_feynman_smoke_report.md"
        )

    print("=" * 62)
    print("  PySR: Feynman Equations")
    print(
        f"  N_SAMPLES={N_SAMPLES}  niterations={niterations}  "
        f"maxsize={MAXSIZE}  populations={POPULATIONS}"
    )
    if smoke_mode:
        print(f"  MODE=smoke (first_AI ok 式の CSV 順先頭 {SMOKE_OK_COUNT} 式)")
    elif limit:
        print(f"  LIMIT={limit} equations (debug mode)")
    print("=" * 62)

    equations = load_equations(CSV_PATH)
    baseline_first_ids = load_baseline_ok_ids(BASELINE_FIRST_AI_PATH)
    baseline_cursor_ids = load_baseline_ok_ids(BASELINE_CURSOR_PATH)

    if smoke_mode:
        if not baseline_first_ids:
            print("[ERROR] --smoke requires baseline results at", BASELINE_FIRST_AI_PATH)
            sys.exit(1)
        ok_in_csv_order = [
            (i, eq) for i, eq in enumerate(equations)
            if eq["filename"] in baseline_first_ids
        ]
        equations = ok_in_csv_order[:SMOKE_OK_COUNT]
        smoke_ids = [eq["filename"] for _, eq in equations]
        print(f"[INFO] Smoke test: {smoke_ids}")
    elif limit:
        equations = [(i, eq) for i, eq in enumerate(equations[:limit])]
    else:
        equations = list(enumerate(equations))

    print(f"[INFO] Running {len(equations)} equations")

    attempted_ids = [eq["filename"] for _, eq in equations]

    if baseline_first_ids:
        print(f"[INFO] Baseline first_AI ok_ids ({len(baseline_first_ids)}): "
              f"{', '.join(sorted(baseline_first_ids))}")

    results = []
    ok_count = 0
    total_start = time.time()

    for run_idx, (csv_idx, eq) in enumerate(equations):
        eq_id = eq["filename"]
        n_vars = eq["n_vars"]
        print(f"\n[{run_idx+1:03d}/{len(equations)}] {eq_id}  (vars={n_vars})")

        X, y = generate_dataset(eq, N_SAMPLES, seed=42 + csv_idx)

        if len(y) < 10:
            print(f"  [SKIP] too few valid samples ({len(y)})")
            results.append({
                "index": run_idx + 1, "csv_index": csv_idx + 1, "eq_id": eq_id,
                "n_vars": n_vars, "var_names": eq["var_names"],
                "status": "skipped", "reason": "too_few_valid_samples",
                "found_formula": None, "found_python": None,
                "rmse": None, "complexity": None,
                "elapsed_s": 0.0, "candidates": [], "all_candidates": [],
            })
            continue

        mask = np.isfinite(y)
        if mask.sum() < 10:
            print(f"  [SKIP] too few finite y ({mask.sum()})")
            results.append({
                "index": run_idx + 1, "csv_index": csv_idx + 1, "eq_id": eq_id,
                "n_vars": n_vars, "var_names": eq["var_names"],
                "status": "skipped", "reason": "too_few_finite_y",
                "found_formula": None, "found_python": None,
                "rmse": None, "complexity": None,
                "elapsed_s": 0.0, "candidates": [], "all_candidates": [],
            })
            continue

        X, y = X[mask], y[mask]

        result = run_pysr(eq_id, X, y, niterations)
        result["index"] = run_idx + 1
        result["csv_index"] = csv_idx + 1
        result["n_vars"] = n_vars
        result["var_names"] = eq["var_names"]
        results.append(result)

        rmse_val = result.get("rmse")
        rmse_str = f"{rmse_val:.3e}" if rmse_val is not None else "N/A"
        print(f"  Found:  {result.get('found_formula', 'N/A')}")
        print(f"  RMSE:   {rmse_str}  Status: {result['status']}  Time: {result['elapsed_s']:.2f}s")

        if result["status"] == "ok":
            ok_count += 1

        pysr_ok_ids = [r["eq_id"] for r in results if r.get("status") == "ok"]
        comparison_first = compare_with_baseline(
            pysr_ok_ids, baseline_first_ids, attempted_ids
        )
        comparison_cursor = compare_with_baseline(
            pysr_ok_ids, baseline_cursor_ids, attempted_ids
        )

        os.makedirs(os.path.dirname(results_path), exist_ok=True)
        partial_summary = {
            "total": len(equations),
            "ok": ok_count,
            "completed": run_idx + 1,
            "comparison_first_AI": comparison_first,
            "comparison_cursor": comparison_cursor,
            "settings": _build_settings(niterations),
            "results": results,
        }
        with open(results_path, "w", encoding="utf-8") as f:
            json.dump(partial_summary, f, ensure_ascii=False, indent=2)

    total_elapsed = time.time() - total_start
    pysr_ok_ids = [r["eq_id"] for r in results if r.get("status") == "ok"]
    comparison_first = compare_with_baseline(
        pysr_ok_ids, baseline_first_ids, attempted_ids
    )
    comparison_cursor = compare_with_baseline(
        pysr_ok_ids, baseline_cursor_ids, attempted_ids
    )

    summary = {
        "total": len(equations),
        "ok": ok_count,
        "skipped": sum(1 for r in results if r.get("status") == "skipped"),
        "failed": sum(
            1 for r in results
            if r.get("status") not in ("ok", "skipped", "partial")
        ),
        "partial": sum(1 for r in results if r.get("status") == "partial"),
        "total_elapsed_s": total_elapsed,
        "comparison_first_AI": comparison_first,
        "comparison_cursor": comparison_cursor,
        "smoke_mode": smoke_mode,
        "settings": _build_settings(niterations),
        "results": results,
    }

    with open(results_path, "w", encoding="utf-8") as f:
        json.dump(summary, f, ensure_ascii=False, indent=2)
    print(f"\n[INFO] Results saved to {results_path}")

    write_report(summary, report_path)

    print("\n" + "=" * 62)
    print(f"  SUMMARY: {ok_count} / {len(equations)} equations recovered")
    if comparison_first:
        print(f"  vs first_AI: delta_ok={comparison_first['delta_ok']}  "
              f"new={comparison_first['new_ok_ids']}  lost={comparison_first['lost_ok_ids']}")
    if comparison_cursor:
        print(f"  vs cursor:   delta_ok={comparison_cursor['delta_ok']}  "
              f"new={comparison_cursor['new_ok_ids']}  lost={comparison_cursor['lost_ok_ids']}")
    print(f"  Total time: {total_elapsed:.1f}s ({total_elapsed/60:.1f}min)")
    print("=" * 62)


if __name__ == "__main__":
    main()
