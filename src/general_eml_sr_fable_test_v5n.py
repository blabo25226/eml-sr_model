"""
general_eml_sr_fable_test_v5n.py
======================================================
非 Feynman 一般ベンチマーク (v3 と同一の合成 20 式) に
**1% ガウシアンノイズ**を注入したノイズ付き版。

v5 方針 (実データ指向) により、誤差なしベンチは行わず
このノイズ付き版を回帰チェックの標準とする。
v4/v5 エンジンの比較には --tag を使う (出力ファイル名に付加)。

条件: BEAM=1000, N=750, CPLX=6, 110s/式, early_exit=9e-3,
ホールドアウト 250 点 (クリーン) で σ 比例閾値判定。
ノイズ: y += N(0, (0.01·std(y))²), seed=777+行番号。

使い方:
  python3 general_eml_sr_fable_test_v5n.py --tag v4base
  python3 general_eml_sr_fable_test_v5n.py --tag v5
======================================================
"""

import os
import sys
import json
import time
import numpy as np

from feynman_fable_common import PROJECT_ROOT, run_sr
from general_eml_sr_fable_test_v3 import BENCH, gen, N_TRAIN, N_TEST

NOISE_REL = 0.01

SEARCHER_PARAMS = dict(
    max_complexity=6,
    complexity_penalty=0.08,
    beam_width=1000,
    time_budget_s=110.0,
    subsample_size=256,
    early_exit_threshold=9e-3,
    refinement_top_k=8,
    snap_constants=True,
    powerlaw_stage=True,
    ratio_search=True,
    rational_stage=True,
    affine_scaling=True,
    max_boost_terms=6,
    verbose=False,
)


def main():
    tag = "v5"
    if "--tag" in sys.argv:
        tag = sys.argv[sys.argv.index("--tag") + 1]

    results_path = os.path.join(
        PROJECT_ROOT, "results", f"eml_sr_fable_general_v5n_{tag}_results.json")
    report_path = os.path.join(
        PROJECT_ROOT, "texts", f"eml_sr_fable_general_v5n_{tag}_report.md")

    print("=" * 62)
    print(f"  EML-SR Fable: General benchmark + 1% noise  [v5n, tag={tag}]")
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

        sigma = NOISE_REL * float(np.std(y))
        rng_n = np.random.default_rng(777 + i)
        y_train = y + rng_n.normal(0.0, sigma, size=len(y))

        r = run_sr(name, X, y_train, SEARCHER_PARAMS,
                   X_test=Xt, y_test=yt, sigma=sigma)
        r["category"] = cat
        r["index"] = i + 1
        r["n_vars"] = len(ranges)
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
        "settings": {**SEARCHER_PARAMS, "N_TRAIN": N_TRAIN, "N_TEST": N_TEST,
                     "NOISE_REL": NOISE_REL, "TAG": tag},
        "results": results,
    }
    os.makedirs(os.path.dirname(results_path), exist_ok=True)
    with open(results_path, "w", encoding="utf-8") as f:
        json.dump(summary, f, ensure_ascii=False, indent=2)

    L = [f"# EML-SR Fable — 非 Feynman 一般ベンチ + 1% ノイズ (v5n, tag={tag})", ""]
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
    os.makedirs(os.path.dirname(report_path), exist_ok=True)
    with open(report_path, "w", encoding="utf-8") as f:
        f.write("\n".join(L))

    print(f"\nSUMMARY: {ok}/{len(BENCH)} ok  ({total/60:.1f} min)")
    print(f"Report: {report_path}")


if __name__ == "__main__":
    main()
