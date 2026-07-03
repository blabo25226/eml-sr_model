"""
feynman_eml_sr_model_first_AI.py
======================================================
Feynman 方程式のシンボリック回帰 - eml-sr_model_first_AI 版
  - BEAM_WIDTH : 1000  (従来 eml-sr 実験と同一)
  - N_SAMPLES  : 750   (従来 eml-sr 実験と同一)
  - MAX_COMPLEXITY: 6  (従来 eml-sr 実験と同一)
  - 失敗した式も出力数式を記録しレポートに含める
======================================================
"""

import sys
import os
import json
import time
import numpy as np
import pandas as pd
import eml_sr_model_first_AI

# ===== パス設定 =====
SCRIPT_DIR   = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = SCRIPT_DIR

CSV_PATH     = os.path.join(PROJECT_ROOT, "data", "FeynmanEquations.csv")
RESULTS_PATH = os.path.join(PROJECT_ROOT, "results", "eml_sr_model_first_AI_feynman_results.json")
REPORT_PATH  = os.path.join(PROJECT_ROOT, "texts", "eml_sr_model_first_AI_feynman_report.md")

# ===== 実験設定 =====
N_SAMPLES       = 750
MAX_COMPLEXITY  = 6
BEAM_WIDTH      = 1000
RMSE_THRESHOLD  = 1e-4   # ok 判定閾値
RMSE_PARTIAL    = 1e-2   # partial 判定閾値


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
        filename   = str(row["Filename"]).strip()
        formula    = str(row["Formula"]).strip()
        output_var = str(row["Output"]).strip()
        n_vars     = int(row["# variables"])

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
    rng      = np.random.default_rng(seed)
    base_env = _build_eval_env()

    X_list, y_list = [], []
    max_attempts   = n_samples * 6
    attempt        = 0

    while len(y_list) < n_samples and attempt < max_attempts:
        attempt += 1
        point     = []
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


def run_sr(eq_id, X_train, y_train):
    """eml-sr_model_first_AI を実行して結果 dict を返す"""
    base = {
        "eq_id": eq_id, "status": "failed",
        "found_formula": None, "found_python": None,
        "rmse": None, "complexity": None,
        "elapsed_s": 0.0, "candidates": [],
        "all_candidates": [],
    }
    try:
        inputs_list  = X_train.tolist()
        targets_list = y_train.tolist()

        searcher   = eml_sr_model_first_AI.Searcher(max_complexity=MAX_COMPLEXITY, beam_width=BEAM_WIDTH)
        start_t    = time.time()
        candidates = searcher.find_candidates(inputs_list, targets_list)
        elapsed    = time.time() - start_t

        base["elapsed_s"] = elapsed

        if not candidates:
            base["status"] = "no_candidates"
            return base

        best_cand, best_rmse_val = None, float("inf")
        cand_infos = []

        for cand in candidates:
            try:
                preds     = np.array(cand.predict(inputs_list), dtype=float)
                cand_rmse = rmse(y_train, preds) if np.all(np.isfinite(preds)) else float("inf")
            except Exception:
                cand_rmse = float("inf")

            info = {
                "formula"   : cand.formula,
                "python"    : cand.to_python(),
                "complexity": cand.complexity,
                "rmse"      : None if not np.isfinite(cand_rmse) else cand_rmse,
                "error"     : cand.error,
            }
            cand_infos.append(info)

            if cand_rmse < best_rmse_val:
                best_rmse_val = cand_rmse
                best_cand     = cand

        base["all_candidates"] = cand_infos

        if best_cand is None:
            base["status"] = "prediction_failed"
            return base

        base["found_formula"] = best_cand.formula
        base["found_python"]  = best_cand.to_python()
        base["complexity"]    = best_cand.complexity
        base["rmse"]          = best_rmse_val if np.isfinite(best_rmse_val) else None
        base["candidates"]    = cand_infos

        if np.isfinite(best_rmse_val) and best_rmse_val < RMSE_THRESHOLD:
            base["status"] = "ok"
        elif np.isfinite(best_rmse_val) and best_rmse_val < RMSE_PARTIAL:
            base["status"] = "partial"
        else:
            base["status"] = "failed"

        return base

    except Exception as exc:
        base["status"]    = "error"
        base["error_msg"] = str(exc)
        return base


# ------------------------------------------------------------------
# レポート生成
# ------------------------------------------------------------------

def _fmt_rmse(v):
    if v is None:
        return "N/A"
    return f"{float(v):.3e}"


def _status_icon(s):
    return {"ok": "✅", "partial": "🟡", "failed": "❌",
            "skipped": "⏭️", "no_candidates": "🔲",
            "prediction_failed": "⚠️", "error": "💥"}.get(s, "❓")


def write_report(summary, report_path):
    results       = summary.get("results", [])
    total         = summary.get("total", 0)
    ok_count      = summary.get("ok", 0)
    skipped       = summary.get("skipped", 0)
    total_elapsed = summary.get("total_elapsed_s", 0.0)
    settings      = summary.get("settings", {})

    status_counts = {}
    for r in results:
        s = r.get("status", "unknown")
        status_counts[s] = status_counts.get(s, 0) + 1

    partial_count   = status_counts.get("partial", 0)
    ok_rate         = ok_count / total * 100 if total else 0
    ok_partial_rate = (ok_count + partial_count) / total * 100 if total else 0

    from datetime import datetime
    now_str = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

    L = []
    L.append("# EML-SR Model First AI — Feynman Equations 全式推定レポート")
    L.append("")
    L.append(f"**生成日時:** {now_str}  ")
    L.append(f"**エンジン:** `eml-sr_model_first_AI` (Rust / PyO3)  ")
    L.append("")
    L.append("---")
    L.append("")
    L.append("## 1. 実験概要")
    L.append("")
    L.append("### 目的")
    L.append("")
    L.append(
        "従来の Python 版 eml-sr (`sample_code/gh3_allfunc_moreestimate_sr.py`) と同一の"
        "ハイパーパラメータ (BEAM_WIDTH=1000, N_SAMPLES=750, MAX_COMPLEXITY=6) を用い、"
        "新モデル **eml-sr_model_first_AI** により Feynman 方程式ベンチマークを実行する。"
        "従来実装との回収率・実行時間・出力数式の差異を比較可能な形式で記録する。"
    )
    L.append("")
    L.append("### 実験設定")
    L.append("")
    L.append("| パラメータ | 値 | 備考 |")
    L.append("|-----------|-----|------|")
    L.append(f"| N_SAMPLES | {settings.get('N_SAMPLES')} | 従来実験と同一 |")
    L.append(f"| BEAM_WIDTH | {settings.get('BEAM_WIDTH')} | 従来実験と同一 |")
    L.append(f"| MAX_COMPLEXITY | {settings.get('MAX_COMPLEXITY')} | 従来実験と同一 |")
    L.append(f"| RMSE 閾値 (ok) | {settings.get('RMSE_THRESHOLD')} | 従来実験と同一 |")
    L.append(f"| RMSE 閾値 (partial) | {settings.get('RMSE_PARTIAL')} | 従来実験と同一 |")
    L.append(f"| 合計実行時間 | {total_elapsed:.0f}s ({total_elapsed/60:.1f}min) | — |")
    L.append("")
    L.append("---")
    L.append("")
    L.append("## 2. 総合結果サマリー")
    L.append("")
    L.append("| 指標 | 値 |")
    L.append("|-----|-----|")
    L.append(f"| 対象方程式数 | {total} |")
    L.append(f"| ✅ 回収成功 (RMSE < 1e-4) | {ok_count} ({ok_rate:.1f}%) |")
    L.append(f"| 🟡 部分的回収 (RMSE < 1e-2) | {partial_count} ({partial_count/total*100:.1f}%) |")
    L.append(f"| ❌ 失敗 | {status_counts.get('failed', 0)} |")
    L.append(f"| ⏭️ スキップ | {skipped} |")
    L.append(f"| **成功+部分合計** | **{ok_count + partial_count} ({ok_partial_rate:.1f}%)** |")
    L.append("")
    L.append("---")
    L.append("")

    L.append("## 3. 各方程式の詳細結果")
    L.append("")
    L.append("| # | 式ID | 変数数 | ステータス | RMSE | 複雑度 | 発見式 | 時間(s) |")
    L.append("|---|------|--------|------------|------|--------|--------|---------|")

    for r in results:
        idx     = r.get("index", "?")
        eq_id   = r.get("eq_id", "?")
        n_vars  = r.get("n_vars", "?")
        status  = r.get("status", "unknown")
        icon    = _status_icon(status)
        rmse_s  = _fmt_rmse(r.get("rmse"))
        cplx    = r.get("complexity", "N/A")
        formula = str(r.get("found_formula") or "N/A")
        if len(formula) > 55:
            formula = formula[:52] + "..."
        elapsed = r.get("elapsed_s", 0)
        L.append(f"| {idx} | `{eq_id}` | {n_vars} | {icon} {status} | {rmse_s} | {cplx} | `{formula}` | {elapsed:.1f} |")

    L.append("")
    L.append("---")
    L.append("")

    L.append("## 4. 回収成功した方程式の詳細")
    L.append("")
    ok_results = [r for r in results if r.get("status") == "ok"]
    if ok_results:
        for r in ok_results:
            L.append(f"### `{r['eq_id']}`")
            L.append("")
            vn = ", ".join(r.get("var_names", []))
            L.append(f"- **変数:** {vn}")
            L.append(f"- **発見式:** `{r.get('found_formula', 'N/A')}`")
            L.append(f"- **Python 表現:** `{r.get('found_python', 'N/A')}`")
            L.append(f"- **RMSE:** {_fmt_rmse(r.get('rmse'))}")
            L.append(f"- **複雑度:** {r.get('complexity', 'N/A')}")
            L.append(f"- **実行時間:** {r.get('elapsed_s', 0):.2f}s")
            L.append("")
    else:
        L.append("（成功した方程式はありませんでした）")
        L.append("")

    L.append("---")
    L.append("")

    L.append("## 5. 部分的回収の方程式")
    L.append("")
    partial_results = [r for r in results if r.get("status") == "partial"]
    if partial_results:
        for r in partial_results:
            vn = ", ".join(r.get("var_names", []))
            L.append(f"### `{r['eq_id']}`")
            L.append(f"- **変数:** {vn}")
            L.append(f"- **発見式:** `{r.get('found_formula', 'N/A')}`")
            L.append(f"- **RMSE:** {_fmt_rmse(r.get('rmse'))}")
            L.append(f"- **複雑度:** {r.get('complexity', 'N/A')}")
            L.append("")
    else:
        L.append("（部分的回収の方程式はありませんでした）")
        L.append("")

    L.append("---")
    L.append("")

    L.append("## 6. 失敗した方程式と出力数式")
    L.append("")
    L.append(
        "以下は推定に失敗（RMSE >= 1e-4）した方程式の一覧である。"
        "Pareto front の最良候補を記録する。"
    )
    L.append("")
    L.append("| # | 式ID | 変数 | RMSE | 発見式 | Python 表現 |")
    L.append("|---|------|------|------|--------|-------------|")

    failed_results = [r for r in results if r.get("status") in (
        "failed", "error", "no_candidates", "prediction_failed")]
    for r in failed_results:
        idx     = r.get("index", "?")
        eq_id   = r.get("eq_id", "?")
        vnames  = ", ".join(r.get("var_names", []))
        rmse_s  = _fmt_rmse(r.get("rmse"))
        formula = str(r.get("found_formula") or "N/A")
        python  = str(r.get("found_python") or "N/A")
        if len(formula) > 60:
            formula = formula[:57] + "..."
        if len(python) > 60:
            python = python[:57] + "..."
        L.append(f"| {idx} | `{eq_id}` | {vnames} | {rmse_s} | `{formula}` | `{python}` |")

    L.append("")
    L.append("---")
    L.append("")

    L.append("## 7. 失敗した方程式の Pareto Front 全候補")
    L.append("")
    for r in failed_results:
        eq_id  = r.get("eq_id", "?")
        vnames = ", ".join(r.get("var_names", []))
        cands  = r.get("all_candidates", r.get("candidates", []))
        if not cands:
            continue

        L.append(f"### `{eq_id}` （変数: {vnames}）")
        L.append("")
        L.append("| 複雑度 | RMSE | 式 | Python 表現 |")
        L.append("|--------|------|-----|------------|")
        for c in sorted(cands, key=lambda x: x.get("complexity", 99)):
            crmse    = _fmt_rmse(c.get("rmse"))
            cformula = str(c.get("formula", "N/A"))
            cpython  = str(c.get("python", "N/A"))
            if len(cformula) > 55:
                cformula = cformula[:52] + "..."
            if len(cpython) > 55:
                cpython = cpython[:52] + "..."
            L.append(f"| {c.get('complexity', '?')} | {crmse} | `{cformula}` | `{cpython}` |")
        L.append("")

    L.append("---")
    L.append("")
    L.append("## 8. 変数数別の成功率")
    L.append("")
    var_stats = {}
    for r in results:
        nv = r.get("n_vars", 0)
        st = r.get("status", "unknown")
        if nv not in var_stats:
            var_stats[nv] = {"ok": 0, "partial": 0, "total": 0}
        var_stats[nv]["total"] += 1
        if st == "ok":
            var_stats[nv]["ok"] += 1
        elif st == "partial":
            var_stats[nv]["partial"] += 1

    L.append("| 変数数 | 対象数 | 成功 (ok) | 部分回収 | 合計回収率 |")
    L.append("|--------|--------|-----------|----------|-----------|")
    for nv in sorted(var_stats):
        t  = var_stats[nv]["total"]
        o  = var_stats[nv]["ok"]
        p  = var_stats[nv]["partial"]
        r_ = (o + p) / t * 100 if t else 0
        L.append(f"| {nv} | {t} | {o} | {p} | {r_:.0f}% |")

    L.append("")
    L.append("---")
    L.append("")
    L.append("## 9. 考察")
    L.append("")
    L.append("### 9.1 従来 eml-sr 実験との比較")
    L.append("")
    L.append(
        "従来実験 (`sample_code/allfunc_moreestimate_sr_report.md`) では "
        "99式中 9式 (9.1%) を完全回収、部分回収含め 10式 (10.1%) であった。"
        "本実験は同一データ・同一ハイパーパラメータで eml-sr_model_first_AI を適用し、"
        "Rust 実装による速度・数値安定性・探索挙動の差異を評価する。"
    )
    L.append("")
    L.append("### 9.2 EML 演算子と失敗パターン")
    L.append("")
    L.append(
        "EML 演算子 $\\mathrm{EML}(a,b) = e^a - \\ln b$ は指数・対数構造をコンパクトに表現できるが、"
        "Gaussian 型 ($\\exp(-\\theta^2/2)$)、相対論的因子 ($1/\\sqrt{1-v^2/c^2}$)、"
        "有理数係数 ($1/2$, $3/2$)、4変数以上の式は複雑度 6 の範囲では依然困難である。"
    )
    L.append("")
    L.append(
        f"今回の実験では {total} 個の方程式に対して "
        f"**{ok_count} 式 ({ok_rate:.1f}%)** を完全回収し、"
        f"部分的回収も含めると **{ok_count + partial_count} 式 ({ok_partial_rate:.1f}%)** となった。"
    )
    L.append("")
    L.append("---")
    L.append("")
    L.append("*本レポートは `feynman_eml_sr_model_first_AI.py` により自動生成されました。*")
    L.append("")

    os.makedirs(os.path.dirname(report_path), exist_ok=True)
    with open(report_path, "w", encoding="utf-8") as f:
        f.write("\n".join(L))

    print(f"[INFO] Report written to {report_path}")


# ------------------------------------------------------------------
# メイン
# ------------------------------------------------------------------

def main():
    limit = None
    if len(sys.argv) > 1:
        try:
            limit = int(sys.argv[1])
        except ValueError:
            pass

    print("=" * 62)
    print("  EML-SR Model First AI: Feynman Equations")
    print(f"  BEAM_WIDTH={BEAM_WIDTH}  N_SAMPLES={N_SAMPLES}  MAX_COMPLEXITY={MAX_COMPLEXITY}")
    if limit:
        print(f"  LIMIT={limit} equations (debug mode)")
    print("=" * 62)

    equations = load_equations(CSV_PATH)
    if limit:
        equations = equations[:limit]
    print(f"[INFO] Loaded {len(equations)} equations from CSV")

    results     = []
    ok_count    = 0
    total_start = time.time()

    for idx, eq in enumerate(equations):
        eq_id  = eq["filename"]
        n_vars = eq["n_vars"]
        print(f"\n[{idx+1:03d}/{len(equations)}] {eq_id}  (vars={n_vars})")

        X, y = generate_dataset(eq, N_SAMPLES, seed=42 + idx)

        if len(y) < 10:
            print(f"  [SKIP] too few valid samples ({len(y)})")
            results.append({
                "index": idx + 1, "eq_id": eq_id,
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
                "index": idx + 1, "eq_id": eq_id,
                "n_vars": n_vars, "var_names": eq["var_names"],
                "status": "skipped", "reason": "too_few_finite_y",
                "found_formula": None, "found_python": None,
                "rmse": None, "complexity": None,
                "elapsed_s": 0.0, "candidates": [], "all_candidates": [],
            })
            continue

        X, y = X[mask], y[mask]

        result = run_sr(eq_id, X, y)
        result["index"]     = idx + 1
        result["n_vars"]    = n_vars
        result["var_names"] = eq["var_names"]
        results.append(result)

        rmse_val = result.get("rmse")
        rmse_str = f"{rmse_val:.3e}" if rmse_val is not None else "N/A"
        print(f"  Found:  {result.get('found_formula', 'N/A')}")
        print(f"  RMSE:   {rmse_str}  Status: {result['status']}  Time: {result['elapsed_s']:.2f}s")

        if result["status"] == "ok":
            ok_count += 1

        # 中間保存（長時間実行対策）
        os.makedirs(os.path.dirname(RESULTS_PATH), exist_ok=True)
        partial_summary = {
            "total": len(equations),
            "ok": ok_count,
            "completed": idx + 1,
            "settings": {
                "N_SAMPLES": N_SAMPLES,
                "MAX_COMPLEXITY": MAX_COMPLEXITY,
                "BEAM_WIDTH": BEAM_WIDTH,
                "RMSE_THRESHOLD": RMSE_THRESHOLD,
                "RMSE_PARTIAL": RMSE_PARTIAL,
            },
            "results": results,
        }
        with open(RESULTS_PATH, "w", encoding="utf-8") as f:
            json.dump(partial_summary, f, ensure_ascii=False, indent=2)

    total_elapsed = time.time() - total_start

    summary = {
        "total"          : len(equations),
        "ok"             : ok_count,
        "skipped"        : sum(1 for r in results if r.get("status") == "skipped"),
        "failed"         : sum(1 for r in results if r.get("status") not in ("ok", "skipped", "partial")),
        "partial"        : sum(1 for r in results if r.get("status") == "partial"),
        "total_elapsed_s": total_elapsed,
        "settings"       : {
            "N_SAMPLES"     : N_SAMPLES,
            "MAX_COMPLEXITY": MAX_COMPLEXITY,
            "BEAM_WIDTH"    : BEAM_WIDTH,
            "RMSE_THRESHOLD": RMSE_THRESHOLD,
            "RMSE_PARTIAL"  : RMSE_PARTIAL,
        },
        "results": results,
    }

    with open(RESULTS_PATH, "w", encoding="utf-8") as f:
        json.dump(summary, f, ensure_ascii=False, indent=2)
    print(f"\n[INFO] Results saved to {RESULTS_PATH}")

    write_report(summary, REPORT_PATH)

    print("\n" + "=" * 62)
    print(f"  SUMMARY: {ok_count} / {len(equations)} equations recovered")
    print(f"  Total time: {total_elapsed:.1f}s ({total_elapsed/60:.1f}min)")
    print("=" * 62)


if __name__ == "__main__":
    main()
