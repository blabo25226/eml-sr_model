"""
feynman_eml_sr_fable_test1.py
======================================================
Feynman 方程式ベンチマーク — eml-sr_fable 【1.test 高速条件】

指示書の条件:
  - BEAM_WIDTH     = 1000
  - N_SAMPLES      = 750
  - MAX_COMPLEXITY = 6

再現性:
  - データ生成シード: 42 + CSV 行番号（cursor 実験と同一）
  - アルゴリズム内部のサブサンプルは決定的（等間隔抽出）で乱数不使用

使い方:
  python3 feynman_eml_sr_fable_test1.py            # 全 99 式
  python3 feynman_eml_sr_fable_test1.py 10         # 先頭 10 式のみ
  python3 feynman_eml_sr_fable_test1.py --smoke    # スモーク 3 式
======================================================
"""

import os
import sys

from feynman_fable_common import PROJECT_ROOT, run_benchmark

# ===== 1.test 実験設定 =====
N_SAMPLES = 750

SEARCHER_PARAMS = dict(
    max_complexity=6,        # 指示書の条件（ビームサーチの構造部分に適用）
    complexity_penalty=0.08,
    beam_width=1000,         # 指示書の条件
    time_budget_s=110.0,     # 1 式あたりの探索時間上限
    subsample_size=256,
    early_exit_threshold=1e-9,
    refinement_top_k=8,
    snap_constants=True,
    powerlaw_stage=True,
    ratio_search=True,
    affine_scaling=True,
    max_boost_terms=6,
    verbose=False,
)

RESULTS_PATH = os.path.join(PROJECT_ROOT, "results", "eml_sr_fable_feynman_test1_results.json")
REPORT_PATH  = os.path.join(PROJECT_ROOT, "texts", "eml_sr_fable_feynman_test1_report.md")

SMOKE_IDS = ["I.12.1", "I.12.5", "I.14.3"]  # first_AI ok 式の CSV 順先頭 3 式

TITLE = "EML-SR Fable — Feynman Equations 1.test レポート (BEAM=1000, N=750, CPLX=6)"

DESCRIPTION = [
    "新アルゴリズム **eml-sr_fable** による Feynman 方程式ベンチマーク（1.test 高速条件）。",
    "",
    "eml-sr_fable は EML 演算子ビームサーチの枠組みを維持したまま、次の 3 段パイプラインで探索する:",
    "",
    "1. **Stage A: べき単項式ソルバー（閉形式）** — 変換ターゲット "
    "$t(y) \\in \\{y, \\log y, 1/y, 1/y^2, y^2\\}$ に対し log 空間最小二乗＋"
    "直交最小二乗（OLS）辞書追跡＋バックフィット＋後方剪定で単項式和 "
    "$\\sum_k c_k \\prod_i x_i^{a_{ki}}$ を数ミリ秒で回収する。",
    "2. **Stage B: 改良 EML ビームサーチ** — 全候補をアフィンスケーリング "
    "$\\min_{a,b}\\mathrm{RMSE}(y, af+b)$ で採点（定数を無料化）、アフィン同値クラスごとの"
    "ビーム占有制限、決定的サブサンプル評価、早期終了、定数スナップ、式あたり時間予算を導入。",
    "3. **Stage C: 乗法分解** — 単項式ホワイトナー $m(x)$ で割った比 $y/m(x)$ を"
    "ビームサーチし、$m(x) \\times g(x)$ として合成する。",
    "",
    "評価条件・データ生成・乱数シードは cursor 実験と同一（RMSE < 1e-4 で完全回収）。",
    "MAX_COMPLEXITY=6 はビームサーチの構造部分に適用され、Stage A/C が組み立てる最終式の"
    "ノード数（複雑度欄）はこれを超える場合がある。",
]


def main():
    limit = None
    smoke = "--smoke" in sys.argv
    for arg in sys.argv[1:]:
        if arg != "--smoke":
            try:
                limit = int(arg)
            except ValueError:
                pass

    results_path = RESULTS_PATH
    report_path  = REPORT_PATH
    if smoke:
        results_path = results_path.replace(".json", "_smoke.json")
        report_path  = report_path.replace(".md", "_smoke.md")

    print("=" * 62)
    print("  EML-SR Fable: Feynman Equations  [1.test]")
    print(f"  BEAM_WIDTH={SEARCHER_PARAMS['beam_width']}  N_SAMPLES={N_SAMPLES}  "
          f"MAX_COMPLEXITY={SEARCHER_PARAMS['max_complexity']}")
    print("=" * 62)

    run_benchmark(
        searcher_params=SEARCHER_PARAMS,
        n_samples=N_SAMPLES,
        results_path=results_path,
        report_path=report_path,
        title=TITLE,
        description_lines=DESCRIPTION,
        limit=limit,
        smoke_ids=SMOKE_IDS if smoke else None,
    )


if __name__ == "__main__":
    main()
