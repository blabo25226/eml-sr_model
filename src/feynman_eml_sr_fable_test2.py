"""
feynman_eml_sr_fable_test2.py
======================================================
Feynman 方程式ベンチマーク — eml-sr_fable 【2.test 緩和条件・v3 アルゴリズム】

高性能 PC でのローカル実行用に条件を緩和した設定:
  - BEAM_WIDTH     = 2000  (1.test: 1000)
  - N_SAMPLES      = 1500  (1.test: 750)
  - MAX_COMPLEXITY = 8     (1.test: 6)
  - time_budget_s  = 600   (1.test: 110)
  - max_boost_terms= 8     (1.test: 6)

v3 アルゴリズム（負値変数対応・有理関数ステージ・Logit 変換・検証分割）を
そのまま適用する。ホールドアウト評価（テスト 500 点、seed=4242+行番号）で
合否判定する。

再現性:
  - データ生成シード: 42 + CSV 行番号（1.test / cursor 実験と同一）
  - テスト点: seed = 4242 + CSV 行番号
  - アルゴリズム内部のサブサンプルは決定的（等間隔抽出）で乱数不使用

使い方:
  python3 feynman_eml_sr_fable_test2.py            # 全 99 式
  python3 feynman_eml_sr_fable_test2.py 10         # 先頭 10 式のみ
  python3 feynman_eml_sr_fable_test2.py --smoke    # スモーク 3 式
======================================================
"""

import os
import sys

from feynman_fable_common import PROJECT_ROOT, run_benchmark

# ===== 2.test 実験設定（緩和） =====
N_SAMPLES = 1500

SEARCHER_PARAMS = dict(
    max_complexity=8,        # 緩和: Lorentz 因子等の深い構造に到達可能
    complexity_penalty=0.08,
    beam_width=2000,         # 緩和
    time_budget_s=600.0,     # 式あたり最大 10 分
    subsample_size=384,
    early_exit_threshold=1e-9,
    refinement_top_k=12,
    snap_constants=True,
    powerlaw_stage=True,
    ratio_search=True,
    affine_scaling=True,
    max_boost_terms=8,
    verbose=False,
)

RESULTS_PATH = os.path.join(PROJECT_ROOT, "results", "eml_sr_fable_feynman_test2_results.json")
REPORT_PATH  = os.path.join(PROJECT_ROOT, "texts", "eml_sr_fable_feynman_test2_report.md")

SMOKE_IDS = ["I.12.1", "I.12.5", "I.14.3"]

TITLE = "EML-SR Fable — Feynman Equations 2.test レポート (BEAM=2000, N=1500, CPLX=8)"

DESCRIPTION = [
    "新アルゴリズム **eml-sr_fable** による Feynman 方程式ベンチマーク（2.test 緩和条件・高性能 PC 向け）。",
    "",
    "アルゴリズムの構成は 1.test と同一（Stage A べき単項式ソルバー → Stage C 乗法分解 → "
    "Stage B 改良 EML ビームサーチ）。探索資源のみを拡大している:",
    "",
    "- MAX_COMPLEXITY 6 → 8: $\\sin$ / $\\exp$ を含む深い構造"
    "（例: $q(E_f + Bv\\sin\\theta)$）へ構造的に到達可能",
    "- BEAM_WIDTH 1000 → 2000, サブサンプル 256 → 384, 時間予算 110s → 600s",
    "- Stage A の最大項数 6 → 8",
    "",
    "評価条件・データ生成・乱数シードは 1.test と同一（RMSE < 1e-4 で完全回収）。",
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
    print("  EML-SR Fable: Feynman Equations  [2.test]")
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
        n_test=500,
    )


if __name__ == "__main__":
    main()
