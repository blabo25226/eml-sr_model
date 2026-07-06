"""
feynman_fable_common.py
======================================================
eml-sr_fable の Feynman ベンチマーク共通ロジック。
test1 (高速条件) / test2 (緩和条件) の両スクリプトから使う。

- データ生成は eml-sr_model_cursor 実験と同一 (seed=42+csv_idx)
- 評価: RMSE < 1e-4 → ok, < 1e-2 → partial (cursor と同一)
- first_AI / cursor ベースラインとの比較指標を出力
======================================================
"""

import os
import sys
import json
import time
import numpy as np
import pandas as pd

SCRIPT_DIR   = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)

CSV_PATH = os.path.join(PROJECT_ROOT, "data", "FeynmanEquations.csv")
FIRST_AI_RESULTS_PATH = os.path.join(
    PROJECT_ROOT, "results", "eml_sr_model_first_AI_feynman_results.json")
CURSOR_RESULTS_PATH = os.path.join(
    PROJECT_ROOT, "results", "eml_sr_model_cursor_feynman_results.json")

RMSE_THRESHOLD = 1e-4
RMSE_PARTIAL   = 1e-2


# ------------------------------------------------------------------
# データ生成（cursor 実験と同一手順・同一シード）
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


def load_equations(csv_path=CSV_PATH):
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


# ------------------------------------------------------------------
# ベースライン比較
# ------------------------------------------------------------------

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


def compare_with_baseline(fable_ok_ids, baseline_ok_ids, attempted_ids=None):
    fable_ok = set(fable_ok_ids)
    baseline_ok = set(baseline_ok_ids)
    if attempted_ids is not None:
        baseline_ok = baseline_ok & set(attempted_ids)
    return {
        "baseline_ok_count": len(baseline_ok),
        "delta_ok": len(fable_ok) - len(baseline_ok),
        "new_ok_ids": sorted(fable_ok - baseline_ok),
        "lost_ok_ids": sorted(baseline_ok - fable_ok),
    }


# ------------------------------------------------------------------
# 1 式の探索
# ------------------------------------------------------------------

def run_sr(eq_id, X_train, y_train, searcher_params,
           X_test=None, y_test=None, sigma=0.0):
    """1 式の探索。

    - 候補選択は訓練 RMSE で行うが、最良から 5% 以内の候補があれば
      最も複雑度の低いものを選ぶ（ノイズ下での過適合候補を避ける）。
    - X_test/y_test が与えられた場合、合否判定はホールドアウトの
      テスト RMSE（クリーンなターゲットとの比較）で行う。
    - sigma > 0 のとき閾値をノイズ水準に比例させる:
      ok = max(1e-4, 0.15*sigma), partial = max(1e-2, 0.5*sigma)
    """
    import eml_sr_fable

    base = {
        "eq_id": eq_id, "status": "failed",
        "found_formula": None, "found_python": None,
        "rmse": None, "train_rmse": None, "test_rmse": None,
        "sigma": sigma, "complexity": None,
        "elapsed_s": 0.0, "candidates": [],
        "all_candidates": [],
    }
    try:
        inputs_list  = X_train.tolist()
        targets_list = y_train.tolist()

        searcher = eml_sr_fable.Searcher(**searcher_params)
        start_t    = time.time()
        candidates = searcher.find_candidates(inputs_list, targets_list)
        elapsed    = time.time() - start_t

        base["elapsed_s"] = elapsed

        if not candidates:
            base["status"] = "no_candidates"
            return base

        cand_infos = []
        scored = []
        for cand in candidates:
            try:
                preds     = np.array(cand.predict(inputs_list), dtype=float)
                cand_rmse = rmse(y_train, preds) if np.all(np.isfinite(preds)) else float("inf")
            except Exception:
                cand_rmse = float("inf")
            cand_infos.append({
                "formula"   : cand.formula,
                "python"    : cand.to_python(),
                "complexity": cand.complexity,
                "rmse"      : None if not np.isfinite(cand_rmse) else cand_rmse,
                "error"     : cand.error,
            })
            scored.append((cand, cand_rmse))

        base["all_candidates"] = cand_infos

        finite = [(c, r) for c, r in scored if np.isfinite(r)]
        if not finite:
            base["status"] = "prediction_failed"
            return base
        best_train = min(r for _, r in finite)
        # 節約的タイブレーク: 最良訓練 RMSE の 5% 帯内で最小複雑度
        band = [(c, r) for c, r in finite
                if r <= best_train * 1.05 + 1e-300]
        best_cand, train_rmse_val = min(band, key=lambda cr: (cr[0].complexity, cr[1]))

        base["found_formula"] = best_cand.formula
        base["found_python"]  = best_cand.to_python()
        base["complexity"]    = best_cand.complexity
        base["train_rmse"]    = train_rmse_val
        base["candidates"]    = cand_infos

        if X_test is not None and y_test is not None:
            try:
                preds_t = np.array(best_cand.predict(X_test.tolist()), dtype=float)
                eval_rmse = rmse(y_test, preds_t) if np.all(np.isfinite(preds_t)) else float("inf")
            except Exception:
                eval_rmse = float("inf")
            base["test_rmse"] = None if not np.isfinite(eval_rmse) else eval_rmse
        else:
            eval_rmse = train_rmse_val
        base["rmse"] = None if not np.isfinite(eval_rmse) else eval_rmse

        thr_ok      = max(RMSE_THRESHOLD, 0.15 * sigma)
        thr_partial = max(RMSE_PARTIAL,   0.50 * sigma)
        if np.isfinite(eval_rmse) and eval_rmse < thr_ok:
            base["status"] = "ok"
        elif np.isfinite(eval_rmse) and eval_rmse < thr_partial:
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


def write_report(summary, report_path, title, description_lines):
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
    L.append(f"# {title}")
    L.append("")
    L.append(f"**生成日時:** {now_str}  ")
    L.append(f"**エンジン:** `eml-sr_fable` (Rust / PyO3)  ")
    L.append("")
    L.append("---")
    L.append("")
    L.append("## 1. 実験概要")
    L.append("")
    for line in description_lines:
        L.append(line)
    L.append("")
    L.append("### 実験設定")
    L.append("")
    L.append("| パラメータ | 値 |")
    L.append("|-----------|-----|")
    for k, v in settings.items():
        L.append(f"| {k} | {v} |")
    L.append(f"| 合計実行時間 | {total_elapsed:.0f}s ({total_elapsed/60:.1f}min) |")
    L.append("")
    L.append("---")
    L.append("")
    L.append("## 2. 総合結果サマリー")
    L.append("")
    L.append("| 指標 | 値 |")
    L.append("|-----|-----|")
    L.append(f"| 対象方程式数 | {total} |")
    L.append(f"| ✅ 回収成功 (RMSE < 1e-4) | {ok_count} ({ok_rate:.1f}%) |")
    L.append(f"| 🟡 部分的回収 (RMSE < 1e-2) | {partial_count} ({partial_count/total*100 if total else 0:.1f}%) |")
    L.append(f"| ❌ 失敗 | {status_counts.get('failed', 0)} |")
    L.append(f"| ⏭️ スキップ | {skipped} |")
    L.append(f"| **成功+部分合計** | **{ok_count + partial_count} ({ok_partial_rate:.1f}%)** |")
    L.append("")

    for label, comparison in summary.get("comparisons", {}).items():
        L.append(f"### {label} ベースラインとの比較")
        L.append("")
        L.append("| 指標 | 値 |")
        L.append("|-----|-----|")
        L.append(f"| {label} ok_count | {comparison.get('baseline_ok_count', 'N/A')} |")
        L.append(f"| fable ok_count | {ok_count} |")
        L.append(f"| **delta_ok** | **{comparison.get('delta_ok', 'N/A')}** |")
        new_ids = comparison.get("new_ok_ids", [])
        lost_ids = comparison.get("lost_ok_ids", [])
        L.append(f"| new_ok_ids | {', '.join(f'`{x}`' for x in new_ids) or '（なし）'} |")
        L.append(f"| lost_ok_ids | {', '.join(f'`{x}`' for x in lost_ids) or '（なし）'} |")
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
    L.append("| # | 式ID | 変数 | RMSE | 発見式 |")
    L.append("|---|------|------|------|--------|")

    failed_results = [r for r in results if r.get("status") in (
        "failed", "error", "no_candidates", "prediction_failed")]
    for r in failed_results:
        idx     = r.get("index", "?")
        eq_id   = r.get("eq_id", "?")
        vnames  = ", ".join(r.get("var_names", []))
        rmse_s  = _fmt_rmse(r.get("rmse"))
        formula = str(r.get("found_formula") or "N/A")
        if len(formula) > 70:
            formula = formula[:67] + "..."
        L.append(f"| {idx} | `{eq_id}` | {vnames} | {rmse_s} | `{formula}` |")

    L.append("")
    L.append("---")
    L.append("")
    L.append("## 7. 変数数別の成功率")
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
    L.append("*本レポートは自動生成されました。*")
    L.append("")

    os.makedirs(os.path.dirname(report_path), exist_ok=True)
    with open(report_path, "w", encoding="utf-8") as f:
        f.write("\n".join(L))

    print(f"[INFO] Report written to {report_path}")


# ------------------------------------------------------------------
# ベンチマーク実行
# ------------------------------------------------------------------

def run_benchmark(searcher_params, n_samples, results_path, report_path,
                  title, description_lines, limit=None, smoke_ids=None,
                  extra_baselines=None, n_test=0, noise_rel=0.0):
    """extra_baselines: {label: results_json_path} で追加の比較対象を指定できる。

    n_test > 0 でホールドアウト評価（テスト点は seed=4242+行番号で新規生成、
    クリーンなターゲットに対する RMSE で合否判定）。
    noise_rel > 0 で訓練ターゲットにガウスノイズ
    y += N(0, (noise_rel*std(y))^2) を注入（seed=777+行番号）。"""
    equations = load_equations()
    baselines = {
        "first_AI": load_baseline_ok_ids(FIRST_AI_RESULTS_PATH),
        "cursor":   load_baseline_ok_ids(CURSOR_RESULTS_PATH),
    }
    for label, path in (extra_baselines or {}).items():
        baselines[label] = load_baseline_ok_ids(path)

    if smoke_ids:
        equations = [(i, eq) for i, eq in enumerate(equations)
                     if eq["filename"] in smoke_ids]
    elif limit:
        equations = list(enumerate(equations[:limit]))
    else:
        equations = list(enumerate(equations))

    attempted_ids = [eq["filename"] for _, eq in equations]
    print(f"[INFO] Running {len(equations)} equations")

    results     = []
    ok_count    = 0
    total_start = time.time()

    for run_idx, (csv_idx, eq) in enumerate(equations):
        eq_id  = eq["filename"]
        n_vars = eq["n_vars"]
        print(f"\n[{run_idx+1:03d}/{len(equations)}] {eq_id}  (vars={n_vars})", flush=True)

        X, y = generate_dataset(eq, n_samples, seed=42 + csv_idx)

        X_test, y_test = (None, None)
        if n_test > 0:
            X_test, y_test = generate_dataset(eq, n_test, seed=4242 + csv_idx)
            if len(y_test) < 10:
                X_test, y_test = (None, None)
            else:
                mask_t = np.isfinite(y_test)
                X_test, y_test = X_test[mask_t], y_test[mask_t]

        if len(y) < 10 or np.isfinite(y).sum() < 10:
            print("  [SKIP] too few valid samples")
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
        X, y = X[mask], y[mask]

        sigma = 0.0
        y_train = y
        if noise_rel > 0.0:
            sigma = noise_rel * float(np.std(y))
            rng_n = np.random.default_rng(777 + csv_idx)
            y_train = y + rng_n.normal(0.0, sigma, size=len(y))

        result = run_sr(eq_id, X, y_train, searcher_params,
                        X_test=X_test, y_test=y_test, sigma=sigma)
        result["index"]     = run_idx + 1
        result["csv_index"] = csv_idx + 1
        result["n_vars"]    = n_vars
        result["var_names"] = eq["var_names"]
        results.append(result)

        rmse_val = result.get("rmse")
        rmse_str = f"{rmse_val:.3e}" if rmse_val is not None else "N/A"
        formula  = str(result.get("found_formula") or "N/A")
        if len(formula) > 90:
            formula = formula[:87] + "..."
        print(f"  Found:  {formula}")
        print(f"  RMSE:   {rmse_str}  Status: {result['status']}  Time: {result['elapsed_s']:.2f}s",
              flush=True)

        if result["status"] == "ok":
            ok_count += 1

        # 途中保存（クラッシュ耐性）
        fable_ok_ids = [r["eq_id"] for r in results if r.get("status") == "ok"]
        partial_summary = {
            "total": len(equations),
            "ok": ok_count,
            "completed": run_idx + 1,
            "comparisons": {
                label: compare_with_baseline(fable_ok_ids, ok_ids, attempted_ids)
                for label, ok_ids in baselines.items()
            },
            "settings": {**searcher_params, "N_SAMPLES": n_samples,
                         "N_TEST": n_test, "NOISE_REL": noise_rel,
                         "RMSE_THRESHOLD": RMSE_THRESHOLD, "RMSE_PARTIAL": RMSE_PARTIAL},
            "results": results,
        }
        os.makedirs(os.path.dirname(results_path), exist_ok=True)
        with open(results_path, "w", encoding="utf-8") as f:
            json.dump(partial_summary, f, ensure_ascii=False, indent=2)

    total_elapsed = time.time() - total_start
    fable_ok_ids = [r["eq_id"] for r in results if r.get("status") == "ok"]

    summary = {
        "total"          : len(equations),
        "ok"             : ok_count,
        "skipped"        : sum(1 for r in results if r.get("status") == "skipped"),
        "failed"         : sum(1 for r in results if r.get("status") not in ("ok", "skipped", "partial")),
        "partial"        : sum(1 for r in results if r.get("status") == "partial"),
        "total_elapsed_s": total_elapsed,
        "comparisons"    : {
            label: compare_with_baseline(fable_ok_ids, ok_ids, attempted_ids)
            for label, ok_ids in baselines.items()
        },
        "settings": {**searcher_params, "N_SAMPLES": n_samples,
                     "N_TEST": n_test, "NOISE_REL": noise_rel,
                     "RMSE_THRESHOLD": RMSE_THRESHOLD, "RMSE_PARTIAL": RMSE_PARTIAL},
        "results": results,
    }

    with open(results_path, "w", encoding="utf-8") as f:
        json.dump(summary, f, ensure_ascii=False, indent=2)
    print(f"\n[INFO] Results saved to {results_path}")

    write_report(summary, report_path, title, description_lines)

    print("\n" + "=" * 62)
    print(f"  SUMMARY: {ok_count} / {len(equations)} equations recovered")
    for label, comp in summary["comparisons"].items():
        print(f"  vs {label}: delta_ok={comp['delta_ok']}  lost={comp['lost_ok_ids']}")
    print(f"  Total time: {total_elapsed:.1f}s ({total_elapsed/60:.1f}min)")
    print("=" * 62)
    return summary
