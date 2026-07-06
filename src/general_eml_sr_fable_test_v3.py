"""
general_eml_sr_fable_test_v3.py
======================================================
非 Feynman 一般ベンチマーク — eml-sr_fable v3 の汎用性測定

pointout_cursor.md の「Feynman 特化」指摘への直接の検知器。
多項式(Nguyen風)・有理式・ロジスティック/tanh・指数・三角を含む
合成 20 式で、**負値域の変数**と**符号混在のターゲット**を過半数に含む。

条件は 1.test と同一 (BEAM=1000, N=750, CPLX=6, 110s/式)、
ホールドアウト 250 点 (別シード) のテスト RMSE で合否判定。

使い方:
  python3 general_eml_sr_fable_test_v3.py
======================================================
"""

import os
import json
import time
import numpy as np

from feynman_fable_common import PROJECT_ROOT, run_sr

RESULTS_PATH = os.path.join(PROJECT_ROOT, "results", "eml_sr_fable_general_v3_results.json")
REPORT_PATH  = os.path.join(PROJECT_ROOT, "texts", "eml_sr_fable_general_v3_report.md")

N_TRAIN = 750
N_TEST  = 250

SEARCHER_PARAMS = dict(
    max_complexity=6,
    complexity_penalty=0.08,
    beam_width=1000,
    time_budget_s=110.0,
    subsample_size=256,
    early_exit_threshold=1e-9,
    refinement_top_k=8,
    snap_constants=True,
    powerlaw_stage=True,
    ratio_search=True,
    rational_stage=True,
    affine_scaling=True,
    max_boost_terms=6,
    verbose=False,
)

# (名前, 関数, [(low, high)] 変数範囲, カテゴリ)
BENCH = [
    # --- 多項式 / Nguyen 風 (負値域) ---
    ("poly3",       lambda X: X[:,0]**3 + X[:,0]**2 + X[:,0],          [(-3, 3)],           "polynomial"),
    ("poly_cross",  lambda X: X[:,0]**2*X[:,1] + X[:,0]*X[:,1],        [(-2, 2), (-2, 2)],  "polynomial"),
    ("nguyen5",     lambda X: np.sin(X[:,0]**2)*np.cos(X[:,0]) - 1,    [(-2, 2)],           "trig-comp"),
    ("nguyen6",     lambda X: np.sin(X[:,0]) + np.sin(X[:,0] + X[:,0]**2), [(-2, 2)],       "trig-comp"),
    ("nguyen7",     lambda X: np.log(X[:,0] + 1) + np.log(X[:,0]**2 + 1), [(0.1, 4)],       "log"),
    # --- 有理式 ---
    ("rational1",   lambda X: (X[:,0]**2 + X[:,1]) / (1 + X[:,0]**2),  [(-3, 3), (-3, 3)],  "rational"),
    ("rational2",   lambda X: 1.0 / (1 + X[:,0]**2),                   [(-4, 4)],           "rational"),
    ("rational3",   lambda X: (X[:,0]*X[:,1]) / (X[:,0] + X[:,1]),     [(0.5, 4), (0.5, 4)],"rational"),
    ("shifted_inv", lambda X: 3.0 / (X[:,0] + 2.5),                    [(-2, 4)],           "rational"),
    # --- ロジスティック / tanh / 指数飽和 ---
    ("logistic1",   lambda X: 1.0 / (1 + np.exp(-2*X[:,0])),           [(-3, 3)],           "logistic"),
    ("logistic2",   lambda X: 1.0 / (1 + np.exp(X[:,1] - 2*X[:,0])),   [(-2, 2), (-2, 2)],  "logistic"),
    ("tanh_sum",    lambda X: np.tanh(X[:,0] + X[:,1]),                [(-2, 2), (-2, 2)],  "logistic"),
    ("saturation",  lambda X: 2.0*(1 - np.exp(-0.8*X[:,0])),           [(0.1, 5)],          "exp"),
    ("exp_decay",   lambda X: 3.0*np.exp(-X[:,0]**2/2),                [(-3, 3)],           "exp"),
    # --- 積・混合構造 (符号混在) ---
    ("mixed_prod",  lambda X: X[:,0]*np.exp(-X[:,1]**2),               [(-2, 2), (-2, 2)],  "mixed"),
    ("sine_amp",    lambda X: X[:,0]*np.sin(X[:,1]),                   [(-3, 3), (-3, 3)],  "mixed"),
    ("gauss2d",     lambda X: np.exp(-(X[:,0]**2 + X[:,1]**2)/2),      [(-2, 2), (-2, 2)],  "exp"),
    ("diff_sq",     lambda X: (X[:,0] - X[:,1])**2 + 1,                [(-3, 3), (-3, 3)],  "polynomial"),
    ("cos_diff",    lambda X: 2*np.cos(X[:,0] - X[:,1]),               [(-3, 3), (-3, 3)],  "trig-comp"),
    ("sqrt_sum",    lambda X: np.sqrt(X[:,0]**2 + X[:,1]**2),          [(-3, 3), (-3, 3)],  "mixed"),
]


def gen(ranges, n, seed):
    rng = np.random.default_rng(seed)
    return np.column_stack([rng.uniform(lo, hi, n) for lo, hi in ranges])


def main():
    print("=" * 62)
    print("  EML-SR Fable: General (non-Feynman) benchmark  [v3]")
    print(f"  {len(BENCH)} synthetic equations, holdout {N_TEST} points")
    print("=" * 62)

    results = []
    ok = 0
    t_all = time.time()
    for i, (name, fn, ranges, cat) in enumerate(BENCH):
        X = gen(ranges, N_TRAIN, seed=1000 + i)
        Xt = gen(ranges, N_TEST, seed=9000 + i)
        y, yt = fn(X), fn(Xt)
        mask = np.isfinite(y); X, y = X[mask], y[mask]
        mask = np.isfinite(yt); Xt, yt = Xt[mask], yt[mask]

        r = run_sr(name, X, y, SEARCHER_PARAMS, X_test=Xt, y_test=yt)
        r["category"] = cat
        r["index"] = i + 1
        r["n_vars"] = len(ranges)
        r["var_names"] = [f"x{j}" for j in range(len(ranges))]
        results.append(r)
        if r["status"] == "ok":
            ok += 1
        tr = r.get("test_rmse")
        print(f"[{i+1:02d}/{len(BENCH)}] {name:12s} ({cat:10s}) "
              f"status={r['status']:7s} test_rmse={tr if tr is None else f'{tr:.3e}'} "
              f"t={r['elapsed_s']:.1f}s", flush=True)

    total = time.time() - t_all
    summary = {
        "total": len(BENCH), "ok": ok,
        "partial": sum(1 for r in results if r["status"] == "partial"),
        "failed": sum(1 for r in results if r["status"] not in ("ok", "partial")),
        "total_elapsed_s": total,
        "settings": {**SEARCHER_PARAMS, "N_TRAIN": N_TRAIN, "N_TEST": N_TEST},
        "results": results,
    }
    os.makedirs(os.path.dirname(RESULTS_PATH), exist_ok=True)
    with open(RESULTS_PATH, "w", encoding="utf-8") as f:
        json.dump(summary, f, ensure_ascii=False, indent=2)

    # 簡易レポート
    L = ["# EML-SR Fable — 非 Feynman 一般ベンチ (v3)", ""]
    L.append(f"合成 {len(BENCH)} 式 (負値域変数・符号混在ターゲットを含む)。"
             f"1.test 条件 + ホールドアウト {N_TEST} 点評価。")
    L.append("")
    L.append(f"**結果: 完全回収 {ok}/{len(BENCH)}、部分 {summary['partial']}、"
             f"失敗 {summary['failed']}、総時間 {total/60:.1f} 分**")
    L.append("")
    L.append("| # | 式 | カテゴリ | ステータス | test RMSE | 複雑度 | 発見式 | 時間(s) |")
    L.append("|---|-----|---------|-----------|-----------|--------|--------|---------|")
    for r in results:
        f_ = str(r.get("found_formula") or "N/A")
        if len(f_) > 50:
            f_ = f_[:47] + "..."
        tr = r.get("test_rmse")
        trs = "N/A" if tr is None else f"{tr:.3e}"
        icon = {"ok": "✅", "partial": "🟡"}.get(r["status"], "❌")
        L.append(f"| {r['index']} | `{r['eq_id']}` | {r['category']} | {icon} {r['status']} "
                 f"| {trs} | {r.get('complexity','?')} | `{f_}` | {r['elapsed_s']:.1f} |")
    L.append("")
    os.makedirs(os.path.dirname(REPORT_PATH), exist_ok=True)
    with open(REPORT_PATH, "w", encoding="utf-8") as f:
        f.write("\n".join(L))

    print(f"\nSUMMARY: {ok}/{len(BENCH)} ok  ({total/60:.1f} min)")
    print(f"Report: {REPORT_PATH}")


if __name__ == "__main__":
    main()
