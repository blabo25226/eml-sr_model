"""
eml_sr_fable_test3_common.py
======================================================
test3 / test3_v2 の共通ロジック — v4 演算子拡張 (abs / sigmoid /
min / max / 連続指数 Pow) の検証ベンチマーク。

28 式構成:
  A. 新関数の直接テスト (9式)
  B. 新関数の組合せ・改造 (9式)
  C. スパース性 (10式): a 変数のデータ中、真の f は b<a 変数のみ使用。
     合否とは別に「変数特定」(発見式に現れる変数集合 == 真の使用集合)
     を専用指標として記録する。

条件は 1.test 系と同一 (BEAM=1000, N=750, CPLX=6, 110s/式)、
ホールドアウト 250 点 (seed 分離) のテスト RMSE で合否判定。
seed: 訓練 1000+i / テスト 9000+i / ノイズ 777+i (一般ベンチと同規約)。
======================================================
"""

import os
import re
import json
import time
import numpy as np

from feynman_fable_common import PROJECT_ROOT, run_sr

N_TRAIN = 750
N_TEST  = 250

BASE_SEARCHER_PARAMS = dict(
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


def sigmoid(z):
    return 1.0 / (1.0 + np.exp(-z))


def _r(lo, hi, n):
    """n 変数すべて同一範囲。"""
    return [(lo, hi)] * n


# (名前, セクション, 関数, 変数範囲リスト, 真に使用する変数 index 集合)
# 使用変数 None は「全変数使用」(A/B セクション)。
BENCH = [
    # ---------- A. 新関数の直接テスト (9式) ----------
    ("A1_abs_diff",     "A", lambda X: np.abs(X[:,0] - X[:,1]),                    _r(-3, 3, 2),  None),
    ("A2_abs_affine",   "A", lambda X: 2*np.abs(X[:,0]) + 1,                       _r(-3, 3, 1),  None),
    ("A3_min",          "A", lambda X: np.minimum(X[:,0], X[:,1]),                 _r(-3, 3, 2),  None),
    ("A4_max_sq",       "A", lambda X: np.maximum(X[:,0]**2, X[:,1]),              [(-2, 2), (-2, 4)], None),
    ("A5_sigmoid_2x",   "A", lambda X: sigmoid(2*X[:,0]),                          _r(-3, 3, 1),  None),
    ("A6_sigmoid_diff", "A", lambda X: sigmoid(X[:,0] - X[:,1]),                   _r(-2, 2, 2),  None),
    ("A7_pow_1_7",      "A", lambda X: X[:,0]**1.7,                                [(0.1, 4)],    None),
    ("A8_hill",         "A", lambda X: 3*X[:,0]**2.5 / (X[:,0]**2.5 + 2**2.5),     [(0.1, 5)],    None),
    ("A9_min_prod",     "A", lambda X: np.minimum(X[:,0]*X[:,1], X[:,2]),          _r(0.1, 3, 3), None),
    # ---------- B. 組合せ・改造 (9式) ----------
    ("B1_absdiff_scale","B", lambda X: np.abs(X[:,0] - X[:,1]) * X[:,2],           [(-2, 2), (-2, 2), (0.5, 3)], None),
    ("B2_gated",        "B", lambda X: X[:,2] * sigmoid(X[:,0] - X[:,1]),          [(-2, 2), (-2, 2), (-2, 2)],  None),
    ("B3_sigmoid_prod", "B", lambda X: sigmoid(X[:,0] * X[:,1]),                   _r(-2, 2, 2),  None),
    ("B4_range",        "B", lambda X: np.maximum(X[:,0], X[:,1]) - np.minimum(X[:,0], X[:,1]), _r(-3, 3, 2), None),
    ("B5_pow_abs",      "B", lambda X: X[:,0]**1.5 + np.abs(X[:,1]),               [(0.1, 4), (-3, 3)], None),
    ("B6_min_quad",     "B", lambda X: np.minimum(X[:,0]**2, X[:,1] + X[:,2]),     [(-2, 2), (0, 3), (0, 3)], None),
    ("B7_max_scale",    "B", lambda X: 2*np.maximum(X[:,0], X[:,1]**2),            [(-3, 3), (-1.5, 1.5)], None),
    ("B8_abs_sin",      "B", lambda X: np.abs(np.sin(X[:,0]) * X[:,1]),            _r(-3, 3, 2),  None),
    ("B9_hill_mono",    "B", lambda X: X[:,1] * X[:,0]**2 / (X[:,0]**2 + 1.5**2),  [(0.1, 5), (0.5, 3)], None),
    # ---------- C. スパース性 (10式): (b 使用, a 総変数) ----------
    # C1 (2, 5)
    ("C1_b2_a5_sigmoid",  "C", lambda X: sigmoid(X[:,0] - X[:,2]),                 _r(-2, 2, 5),   {0, 2}),
    # C2 (3, 7)
    ("C2_b3_a7_ratio",    "C", lambda X: X[:,1]*X[:,4]/X[:,5],                     _r(0.5, 3, 7),  {1, 4, 5}),
    # C3 (4, 20)
    ("C3_b4_a20_mono",    "C", lambda X: X[:,2]*X[:,7]*X[:,11]/X[:,16],            _r(0.5, 3, 20), {2, 7, 11, 16}),
    # C4 (5, 15)
    ("C4_b5_a15_sum",     "C", lambda X: X[:,0]*X[:,1] + X[:,2]*X[:,3] + X[:,4],   _r(-2, 2, 15),  {0, 1, 2, 3, 4}),
    # C5 (6, 20)
    ("C5_b6_a20_mono",    "C", lambda X: X[:,1]*X[:,4]*X[:,8]*X[:,10]/(X[:,13]*X[:,18]), _r(0.5, 3, 20), {1, 4, 8, 10, 13, 18}),
    # C6 (2, 10)
    ("C6_b2_a10_absdiff", "C", lambda X: np.abs(X[:,1] - X[:,6]),                  _r(-3, 3, 10),  {1, 6}),
    # C7 (3, 12)
    ("C7_b3_a12_min",     "C", lambda X: np.minimum(X[:,0]*X[:,3], X[:,8]),        _r(0.5, 3, 12), {0, 3, 8}),
    # C8 (2, 8)
    ("C8_b2_a8_pow",      "C", lambda X: X[:,2]**1.5 * X[:,5],                     _r(0.5, 3, 8),  {2, 5}),
    # C9 (4, 10)
    ("C9_b4_a10_sum",     "C", lambda X: X[:,0]*X[:,7] + X[:,1]*X[:,4],            _r(-2, 2, 10),  {0, 7, 1, 4}),
    # C10 (5, 25)
    ("C10_b5_a25_mono",   "C", lambda X: X[:,2]*X[:,6]*X[:,10]*X[:,17]*X[:,21],    _r(0.5, 2, 25), {2, 6, 10, 17, 21}),
]


def gen(ranges, n, seed):
    rng = np.random.default_rng(seed)
    return np.column_stack([rng.uniform(lo, hi, n) for lo, hi in ranges])


def extract_vars(formula):
    """発見式に現れる変数 index の集合 (v_{i} 表記を解析)。"""
    if not formula:
        return set()
    return {int(m) for m in re.findall(r"v_\{(\d+)\}", str(formula))}


def run_suite(noise_rel, results_path, report_path, title, description_lines,
              searcher_params=None, smoke_ids=None,
              outlier_frac=0.0, outlier_scale=10.0):
    """outlier_frac > 0 でガウシアンノイズに加えて外れ値を注入する:
    訓練点の outlier_frac 割合に ±outlier_scale·σ を加算 (seed=555+行番号)。
    合否閾値はガウシアン σ 基準のまま (外れ値は閾値を緩めない)。"""
    params = dict(BASE_SEARCHER_PARAMS)
    if noise_rel > 0:
        params["early_exit_threshold"] = 9e-3
    if searcher_params:
        params.update(searcher_params)

    bench = BENCH
    if smoke_ids:
        bench = [b for b in BENCH if b[0] in smoke_ids]

    print("=" * 62)
    print(f"  {title}")
    print(f"  {len(bench)} equations, N={N_TRAIN}, holdout {N_TEST}, "
          f"noise_rel={noise_rel}")
    print("=" * 62)

    results = []
    t_all = time.time()
    for i, (name, section, fn, ranges, true_vars) in enumerate(bench):
        idx = next(j for j, b in enumerate(BENCH) if b[0] == name)
        X  = gen(ranges, N_TRAIN, seed=1000 + idx)
        Xt = gen(ranges, N_TEST,  seed=9000 + idx)
        y, yt = fn(X), fn(Xt)
        mask = np.isfinite(y);  X,  y  = X[mask],  y[mask]
        mask = np.isfinite(yt); Xt, yt = Xt[mask], yt[mask]

        sigma = 0.0
        y_train = y
        if noise_rel > 0:
            sigma = noise_rel * float(np.std(y))
            rng_n = np.random.default_rng(777 + idx)
            y_train = y + rng_n.normal(0.0, sigma, size=len(y))
        if outlier_frac > 0 and sigma > 0:
            rng_o = np.random.default_rng(555 + idx)
            n_out = max(1, int(round(outlier_frac * len(y_train))))
            pos = rng_o.choice(len(y_train), size=n_out, replace=False)
            signs = rng_o.choice([-1.0, 1.0], size=n_out)
            y_train = y_train.copy()
            y_train[pos] += signs * outlier_scale * sigma

        r = run_sr(name, X, y_train, params, X_test=Xt, y_test=yt, sigma=sigma)
        r["index"]    = i + 1
        r["section"]  = section
        r["n_vars"]   = len(ranges)
        r["var_names"] = [f"x{j}" for j in range(len(ranges))]

        expected = true_vars if true_vars is not None else set(range(len(ranges)))
        found = extract_vars(r.get("found_formula"))
        r["true_vars"]  = sorted(expected)
        r["found_vars"] = sorted(found)
        r["var_id_ok"]  = (found == expected)
        results.append(r)

        tr = r.get("test_rmse")
        print(f"[{i+1:02d}/{len(bench)}] {name:20s} ({section}) "
              f"status={r['status']:7s} "
              f"test_rmse={tr if tr is None else f'{tr:.3e}'} "
              f"var_id={'OK' if r['var_id_ok'] else 'NG'} "
              f"t={r['elapsed_s']:.1f}s", flush=True)

        # 途中保存（クラッシュ耐性）
        _save(results, bench, params, noise_rel, time.time() - t_all, results_path,
              outlier_frac, outlier_scale)

    total = time.time() - t_all
    summary = _save(results, bench, params, noise_rel, total, results_path,
                    outlier_frac, outlier_scale)
    _write_report(summary, report_path, title, description_lines)
    ok = summary["ok"]
    print(f"\nSUMMARY: {ok}/{len(bench)} ok  "
          f"(A {summary['section_stats']['A']['ok']}/{summary['section_stats']['A']['total']}, "
          f"B {summary['section_stats']['B']['ok']}/{summary['section_stats']['B']['total']}, "
          f"C {summary['section_stats']['C']['ok']}/{summary['section_stats']['C']['total']})  "
          f"var_id C: {summary['var_id_ok_C']}/{summary['section_stats']['C']['total']}  "
          f"({total/60:.1f} min)")
    print(f"Report: {report_path}")
    return summary


def _save(results, bench, params, noise_rel, elapsed, results_path,
          outlier_frac=0.0, outlier_scale=10.0):
    section_stats = {}
    for s in ("A", "B", "C"):
        rs = [r for r in results if r["section"] == s]
        section_stats[s] = {
            "total":   sum(1 for b in bench if b[1] == s),
            "done":    len(rs),
            "ok":      sum(1 for r in rs if r["status"] == "ok"),
            "partial": sum(1 for r in rs if r["status"] == "partial"),
        }
    summary = {
        "total": len(bench),
        "ok": sum(1 for r in results if r["status"] == "ok"),
        "partial": sum(1 for r in results if r["status"] == "partial"),
        "failed": sum(1 for r in results
                      if r["status"] not in ("ok", "partial")),
        "var_id_ok_C": sum(1 for r in results
                           if r["section"] == "C" and r["var_id_ok"]),
        "section_stats": section_stats,
        "total_elapsed_s": elapsed,
        "settings": {**params, "N_TRAIN": N_TRAIN, "N_TEST": N_TEST,
                     "NOISE_REL": noise_rel,
                     "OUTLIER_FRAC": outlier_frac,
                     "OUTLIER_SCALE": outlier_scale},
        "results": results,
    }
    os.makedirs(os.path.dirname(results_path), exist_ok=True)
    with open(results_path, "w", encoding="utf-8") as f:
        json.dump(summary, f, ensure_ascii=False, indent=2)
    return summary


def _write_report(summary, report_path, title, description_lines):
    from datetime import datetime
    results = summary["results"]
    L = [f"# {title}", ""]
    L.append(f"**生成日時:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}  ")
    L.append("**エンジン:** `eml-sr_fable` v4 (Rust / PyO3)  ")
    L.append("")
    L.extend(description_lines)
    L.append("")
    L.append("## 総合結果")
    L.append("")
    L.append("| セクション | 内容 | ok | partial | 失敗 | 計 |")
    L.append("|-----------|------|----|---------|------|-----|")
    labels = {"A": "新関数の直接テスト", "B": "組合せ・改造", "C": "スパース性"}
    for s in ("A", "B", "C"):
        st = summary["section_stats"][s]
        failed = st["done"] - st["ok"] - st["partial"]
        L.append(f"| {s} | {labels[s]} | {st['ok']} | {st['partial']} "
                 f"| {failed} | {st['total']} |")
    L.append(f"| **計** | | **{summary['ok']}** | {summary['partial']} "
             f"| {summary['failed']} | {summary['total']} |")
    L.append("")
    L.append(f"**スパース性 (C) の変数特定成功: "
             f"{summary['var_id_ok_C']}/{summary['section_stats']['C']['total']}**"
             f" (発見式に現れる変数集合 == 真の使用変数集合)")
    L.append("")
    L.append(f"総時間: {summary['total_elapsed_s']/60:.1f} 分")
    L.append("")
    L.append("## 各式の結果")
    L.append("")
    L.append("| # | 式 | 変数(使用/総数) | ステータス | test RMSE | 変数特定 | 複雑度 | 発見式 | 時間(s) |")
    L.append("|---|-----|----------------|-----------|-----------|---------|--------|--------|---------|")
    for r in results:
        f_ = str(r.get("found_formula") or "N/A")
        if len(f_) > 55:
            f_ = f_[:52] + "..."
        tr = r.get("test_rmse")
        trs = "N/A" if tr is None else f"{tr:.3e}"
        icon = {"ok": "✅", "partial": "🟡"}.get(r["status"], "❌")
        vid = "✅" if r["var_id_ok"] else "❌"
        L.append(f"| {r['index']} | `{r['eq_id']}` "
                 f"| {len(r['true_vars'])}/{r['n_vars']} "
                 f"| {icon} {r['status']} | {trs} | {vid} "
                 f"| {r.get('complexity','?')} | `{f_}` | {r['elapsed_s']:.1f} |")
    L.append("")
    L.append("## スパース性 (C) の変数特定詳細")
    L.append("")
    L.append("| 式 | 真の使用変数 | 発見式の変数 | 特定 |")
    L.append("|-----|-------------|-------------|------|")
    for r in results:
        if r["section"] != "C":
            continue
        tv = ", ".join(f"x{j}" for j in r["true_vars"])
        fv = ", ".join(f"x{j}" for j in r["found_vars"]) or "（なし）"
        vid = "✅" if r["var_id_ok"] else "❌"
        L.append(f"| `{r['eq_id']}` | {tv} | {fv} | {vid} |")
    L.append("")
    os.makedirs(os.path.dirname(report_path), exist_ok=True)
    with open(report_path, "w", encoding="utf-8") as f:
        f.write("\n".join(L))
    print(f"[INFO] Report written to {report_path}")
