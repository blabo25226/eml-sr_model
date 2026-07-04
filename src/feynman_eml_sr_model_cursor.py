"""
feynman_eml_sr_model_cursor.py
======================================================
Feynman 方程式のシンボリック回帰 - eml-sr_model_cursor 版
  - BEAM_WIDTH : 1000
  - N_SAMPLES  : 750   (first_AI / 従来実験と同一)
  - MAX_COMPLEXITY: 6
  - COMPLEXITY_PENALTY: 0.08
  - --smoke: first_AI ok 式のうち CSV 順先頭 3 式
  - 失敗した式も出力数式を記録しレポートに含める
  - first_AI ベースラインとの比較指標 (delta_ok, new_ok_ids, lost_ok_ids) を出力
======================================================
"""

import sys
import os
import json
import time
import numpy as np
import pandas as pd
import eml_sr_model_cursor

# ===== パス設定 =====
SCRIPT_DIR   = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)

CSV_PATH              = os.path.join(PROJECT_ROOT, "data", "FeynmanEquations.csv")
RESULTS_PATH          = os.path.join(PROJECT_ROOT, "results", "eml_sr_model_cursor_feynman_results.json")
REPORT_PATH           = os.path.join(PROJECT_ROOT, "texts", "eml_sr_model_cursor_feynman_report.md")
BASELINE_RESULTS_PATH = os.path.join(
    PROJECT_ROOT, "results", "eml_sr_model_first_AI_feynman_results.json"
)

# ===== 実験設定（本番・スモーク共通） =====
N_SAMPLES          = 750
MAX_COMPLEXITY     = 6
BEAM_WIDTH         = 1000
COMPLEXITY_PENALTY = 0.08
RMSE_THRESHOLD     = 1e-4
RMSE_PARTIAL       = 1e-2
SMOKE_OK_COUNT     = 3   # --smoke 時: first_AI ok 式の CSV 順先頭 N 式


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


def load_baseline_ok_ids(path):
    """first_AI ベースラインの ok 式 ID 一覧を読み込む"""
    if not os.path.isfile(path):
        return set()
    with open(path, encoding="utf-8") as f:
        data = json.load(f)
    return {
        r["eq_id"]
        for r in data.get("results", [])
        if r.get("status") == "ok"
    }


def compare_with_baseline(cursor_ok_ids, baseline_ok_ids, attempted_ids=None):
    """cursor と first_AI の ok 集合を比較する。

    attempted_ids を渡すと、実際に実行した式のみを比較対象とする
    （スモーク等の部分実行で、未実行の式が lost として誤検出されるのを防ぐ）。
    """
    cursor_ok = set(cursor_ok_ids)
    baseline_ok = set(baseline_ok_ids)
    if attempted_ids is not None:
        attempted = set(attempted_ids)
        baseline_ok = baseline_ok & attempted
    return {
        "baseline_ok_count": len(baseline_ok),
        "delta_ok": len(cursor_ok) - len(baseline_ok),
        "new_ok_ids": sorted(cursor_ok - baseline_ok),
        "lost_ok_ids": sorted(baseline_ok - cursor_ok),
    }


def run_sr(eq_id, X_train, y_train):
    """eml-sr_model_cursor を実行して結果 dict を返す"""
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

        searcher = eml_sr_model_cursor.Searcher(
            max_complexity=MAX_COMPLEXITY,
            complexity_penalty=COMPLEXITY_PENALTY,
            beam_width=BEAM_WIDTH,
        )
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
    comparison    = summary.get("comparison", {})

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
    L.append("# EML-SR Model Cursor — Feynman Equations 全式推定レポート")
    L.append("")
    L.append(f"**生成日時:** {now_str}  ")
    L.append(f"**エンジン:** `eml-sr_model_cursor` (Rust / PyO3)  ")
    L.append("")
    L.append("---")
    L.append("")
    L.append("## 1. 実験概要")
    L.append("")
    L.append("### 目的")
    L.append("")
    L.append(
        "精度改良版 **eml-sr_model_cursor** により Feynman 方程式ベンチマークを実行し、"
        "ベースライン **eml-sr_model_first_AI** (9式回収) を上回る完全回収数を目指す。"
        "Square/Cube/制限付き Pow 演算子の追加、単一 Param シード＋末尾多スタート refinement、"
        "真の Levenberg-Marquardt 最適化（BFS 内 25 反復・refinement 80 反復の二段構成）"
        "等の改良を統合した評価である（探索設定は first_AI と同一の complexity=6, beam=1000）。"
    )
    L.append("")
    L.append("### 実験設定")
    L.append("")
    L.append("| パラメータ | 値 | 備考 |")
    L.append("|-----------|-----|------|")
    L.append(f"| N_SAMPLES | {settings.get('N_SAMPLES')} | first_AI と同一 |")
    L.append(f"| BEAM_WIDTH | {settings.get('BEAM_WIDTH')} | first_AI と同一 |")
    L.append(f"| MAX_COMPLEXITY | {settings.get('MAX_COMPLEXITY')} | first_AI と同一 |")
    L.append(f"| COMPLEXITY_PENALTY | {settings.get('COMPLEXITY_PENALTY')} | cursor 専用 (first_AI=0.1) |")
    L.append(f"| RMSE 閾値 (ok) | {settings.get('RMSE_THRESHOLD')} | first_AI と同一 |")
    L.append(f"| RMSE 閾値 (partial) | {settings.get('RMSE_PARTIAL')} | first_AI と同一 |")
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
    if comparison:
        L.append("### first_AI ベースラインとの比較")
        L.append("")
        L.append("| 指標 | 値 |")
        L.append("|-----|-----|")
        L.append(f"| first_AI ok_count | {comparison.get('baseline_ok_count', 'N/A')} |")
        L.append(f"| cursor ok_count | {ok_count} |")
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
    L.append("### 9.1 first_AI ベースラインとの比較")
    L.append("")
    if comparison:
        L.append(
            f"first_AI では {comparison.get('baseline_ok_count', 9)} 式を完全回収した。"
            f"cursor 版では **{ok_count} 式** を回収し、"
            f"増分 delta_ok = **{comparison.get('delta_ok', 0)}** である。"
        )
        if comparison.get("lost_ok_ids"):
            L.append(
                f"回帰が発生した式: {', '.join(comparison['lost_ok_ids'])}。"
                "演算子追加や探索空間拡大による beam 打ち切りが原因の可能性がある。"
            )
        if comparison.get("new_ok_ids"):
            L.append(
                f"新規回収式: {', '.join(comparison['new_ok_ids'])}。"
            )
    else:
        L.append("ベースライン結果ファイルが見つからないため比較未実施。")
    L.append("")
    L.append("### 9.2 改良点の効果")
    L.append("")
    L.append(
        "Square/Cube 演算子は二乗・三乗構造の複雑度を削減し、"
        "制限付き Pow は指数定数の表現を可能にする。"
        "真の LM 最適化（BFS 内 25 反復で候補を篩い分け、末尾 refinement で多スタート 80 反復の精密化）"
        "は partial → ok への昇格に寄与する。定数は数値近似で合格判定 (RMSE のみ) とする。"
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
    L.append("*本レポートは `feynman_eml_sr_model_cursor.py` により自動生成されました。*")
    L.append("")

    os.makedirs(os.path.dirname(report_path), exist_ok=True)
    with open(report_path, "w", encoding="utf-8") as f:
        f.write("\n".join(L))

    print(f"[INFO] Report written to {report_path}")


# ------------------------------------------------------------------
# メイン
# ------------------------------------------------------------------

def main():
    limit      = None
    smoke_mode = "--smoke" in sys.argv
    results_path = RESULTS_PATH
    report_path  = REPORT_PATH

    for arg in sys.argv[1:]:
        if arg == "--smoke":
            continue
        try:
            limit = int(arg)
        except ValueError:
            pass

    if smoke_mode:
        results_path = os.path.join(
            PROJECT_ROOT, "results", "eml_sr_model_cursor_feynman_smoke_results.json"
        )
        report_path = os.path.join(
            PROJECT_ROOT, "texts", "eml_sr_model_cursor_feynman_smoke_report.md"
        )

    print("=" * 62)
    print("  EML-SR Model Cursor: Feynman Equations")
    print(
        f"  BEAM_WIDTH={BEAM_WIDTH}  N_SAMPLES={N_SAMPLES}  "
        f"MAX_COMPLEXITY={MAX_COMPLEXITY}  PENALTY={COMPLEXITY_PENALTY}"
    )
    if smoke_mode:
        print(f"  MODE=smoke (first_AI ok 式の CSV 順先頭 {SMOKE_OK_COUNT} 式)")
    elif limit:
        print(f"  LIMIT={limit} equations (debug mode)")
    print("=" * 62)

    equations = load_equations(CSV_PATH)
    baseline_ok_ids = load_baseline_ok_ids(BASELINE_RESULTS_PATH)

    if smoke_mode:
        if not baseline_ok_ids:
            print("[ERROR] --smoke requires baseline results at", BASELINE_RESULTS_PATH)
            sys.exit(1)
        ok_in_csv_order = [
            (i, eq) for i, eq in enumerate(equations)
            if eq["filename"] in baseline_ok_ids
        ]
        equations = ok_in_csv_order[:SMOKE_OK_COUNT]
        smoke_ids = [eq["filename"] for _, eq in equations]
        print(f"[INFO] Smoke test: {smoke_ids}")
    elif limit:
        equations = [(i, eq) for i, eq in enumerate(equations[:limit])]
    else:
        equations = list(enumerate(equations))

    print(f"[INFO] Running {len(equations)} equations")

    # 実際に実行する式のみを比較対象とする（部分実行での lost 誤検出を防ぐ）
    attempted_ids = [eq["filename"] for _, eq in equations]

    if baseline_ok_ids:
        print(f"[INFO] Baseline first_AI ok_ids ({len(baseline_ok_ids)}): "
              f"{', '.join(sorted(baseline_ok_ids))}")

    results     = []
    ok_count    = 0
    total_start = time.time()

    for run_idx, (csv_idx, eq) in enumerate(equations):
        eq_id  = eq["filename"]
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

        result = run_sr(eq_id, X, y)
        result["index"]     = run_idx + 1
        result["csv_index"] = csv_idx + 1
        result["n_vars"]    = n_vars
        result["var_names"] = eq["var_names"]
        results.append(result)

        rmse_val = result.get("rmse")
        rmse_str = f"{rmse_val:.3e}" if rmse_val is not None else "N/A"
        print(f"  Found:  {result.get('found_formula', 'N/A')}")
        print(f"  RMSE:   {rmse_str}  Status: {result['status']}  Time: {result['elapsed_s']:.2f}s")

        if result["status"] == "ok":
            ok_count += 1

        cursor_ok_ids = [r["eq_id"] for r in results if r.get("status") == "ok"]
        comparison = compare_with_baseline(cursor_ok_ids, baseline_ok_ids, attempted_ids)

        os.makedirs(os.path.dirname(results_path), exist_ok=True)
        partial_summary = {
            "total": len(equations),
            "ok": ok_count,
            "completed": run_idx + 1,
            "comparison": comparison,
            "settings": {
                "N_SAMPLES": N_SAMPLES,
                "MAX_COMPLEXITY": MAX_COMPLEXITY,
                "BEAM_WIDTH": BEAM_WIDTH,
                "COMPLEXITY_PENALTY": COMPLEXITY_PENALTY,
                "RMSE_THRESHOLD": RMSE_THRESHOLD,
                "RMSE_PARTIAL": RMSE_PARTIAL,
            },
            "results": results,
        }
        with open(results_path, "w", encoding="utf-8") as f:
            json.dump(partial_summary, f, ensure_ascii=False, indent=2)

    total_elapsed = time.time() - total_start
    cursor_ok_ids = [r["eq_id"] for r in results if r.get("status") == "ok"]
    comparison = compare_with_baseline(cursor_ok_ids, baseline_ok_ids, attempted_ids)

    summary = {
        "total"          : len(equations),
        "ok"             : ok_count,
        "skipped"        : sum(1 for r in results if r.get("status") == "skipped"),
        "failed"         : sum(1 for r in results if r.get("status") not in ("ok", "skipped", "partial")),
        "partial"        : sum(1 for r in results if r.get("status") == "partial"),
        "total_elapsed_s": total_elapsed,
        "comparison"     : comparison,
        "smoke_mode"     : smoke_mode,
        "settings"       : {
            "N_SAMPLES"         : N_SAMPLES,
            "MAX_COMPLEXITY"    : MAX_COMPLEXITY,
            "BEAM_WIDTH"        : BEAM_WIDTH,
            "COMPLEXITY_PENALTY": COMPLEXITY_PENALTY,
            "RMSE_THRESHOLD"    : RMSE_THRESHOLD,
            "RMSE_PARTIAL"      : RMSE_PARTIAL,
        },
        "results": results,
    }

    with open(results_path, "w", encoding="utf-8") as f:
        json.dump(summary, f, ensure_ascii=False, indent=2)
    print(f"\n[INFO] Results saved to {results_path}")

    write_report(summary, report_path)

    print("\n" + "=" * 62)
    print(f"  SUMMARY: {ok_count} / {len(equations)} equations recovered")
    if comparison:
        print(f"  vs first_AI: delta_ok={comparison['delta_ok']}  "
              f"new={comparison['new_ok_ids']}  lost={comparison['lost_ok_ids']}")
    print(f"  Total time: {total_elapsed:.1f}s ({total_elapsed/60:.1f}min)")
    print("=" * 62)


if __name__ == "__main__":
    main()
