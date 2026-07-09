"""
feynman_eml_sr_fable_test1_v5.py
======================================================
Feynman 方程式ベンチマーク — eml-sr_fable 【1.test_v5 誤差付き】

v5 アルゴリズム (実データ指向: ノイズ頑健化・過学習防止・構造拡張) の
ノイズ付き評価。条件は 1.test_v4 と完全同一 (比較のため):
  - BEAM_WIDTH=1000, N_SAMPLES=750, MAX_COMPLEXITY=6, 110s/式
  - 訓練: seed=42+行番号 / テスト: seed=4242+行番号 の新規 250 点
  - ノイズ: y += N(0, (0.01·std(y))²), seed=777+行番号
  - 合否判定はクリーンなテストターゲットへの RMSE (σ 比例閾値)

v5 のアルゴリズム変更:
  P1 ノイズ頑健化: 変換空間 WLS / 原空間 LM ポリッシュ /
     連続指数の検証分割採点+丸め優先
  P2 構造拡張: SigmoidDiff・TanhVar 特徴 / AsinSqrt・Atanh 変換
  P3 残差ブースティング (検証10%改善で受理)
  P4 過学習防止: 検証誤差 1SE ランキング / 相殺和ガード / リッジ強化
  P5 Huber-IRLS 外れ値頑健リフィット

使い方:
  python3 feynman_eml_sr_fable_test1_v5.py            # 全 99 式
  python3 feynman_eml_sr_fable_test1_v5.py --smoke    # スモーク 3 式
======================================================
"""

import os
import sys

from feynman_fable_common import PROJECT_ROOT, run_benchmark

N_SAMPLES = 750
NOISE_REL = 1e-2  # 訓練ターゲットへの相対ノイズ (1%)

SEARCHER_PARAMS = dict(
    max_complexity=6,
    complexity_penalty=0.08,
    beam_width=1000,
    time_budget_s=110.0,
    subsample_size=256,
    early_exit_threshold=9e-3,  # ノイズ床 (0.9×noise_rel) で早期終了
    refinement_top_k=8,
    snap_constants=True,
    powerlaw_stage=True,
    ratio_search=True,
    rational_stage=True,
    affine_scaling=True,
    max_boost_terms=6,
    verbose=False,
)

RESULTS_PATH = os.path.join(PROJECT_ROOT, "results", "eml_sr_fable_feynman_test1_v5_results.json")
REPORT_PATH  = os.path.join(PROJECT_ROOT, "texts", "eml_sr_fable_feynman_test1_v5_report.md")

FABLE_V4_RESULTS_PATH = os.path.join(
    PROJECT_ROOT, "results", "eml_sr_fable_feynman_test1_v4_results.json")
FABLE_V3_RESULTS_PATH = os.path.join(
    PROJECT_ROOT, "results", "eml_sr_fable_feynman_test1_v3_results.json")

SMOKE_IDS = ["I.12.1", "I.12.5", "I.14.3"]

TITLE = "EML-SR Fable — Feynman Equations 1.test_v5 (1%ノイズ, v5 アルゴリズム)"

DESCRIPTION = [
    "**v5 アルゴリズム (実データ指向)** のノイズ付き評価。条件は 1.test_v4 と完全同一",
    "(BEAM=1000, N=750, CPLX=6, 1% ガウスノイズ, ホールドアウト 250 点) で、",
    "v4 結果 (70/99) との差分がアルゴリズム改善の効果。",
    "",
    "### v5 のアルゴリズム変更",
    "",
    "- **P1 ノイズ頑健化**: 変換空間の重み付き最小二乗 (w=1/|T'(y)|、Lorentz 族の",
    "  1/y² ノイズ増幅対策)、原空間 LM ポリッシュ (変換バイアス・有理 EIV の除去)、",
    "  連続指数の検証分割採点と丸め優先。",
    "- **P2 構造拡張**: SigmoidDiff(i,j)・TanhVar(i) 特徴、AsinSqrt (y=sin²(m))・",
    "  Atanh (y=tanh(m)) 変換。",
    "- **P3 残差ブースティング**: 惜しい候補の残差に Stage A を再実行 (検証10%改善で受理)。",
    "- **P4 過学習防止**: 検証誤差ベースの 1標準誤差則ランキング、相殺和ガード",
    "  (Σrms(項) > 30·rms(和) を破棄)、トレース比例リッジ。",
    "- **P5 実データ頑健化**: Huber-IRLS 外れ値頑健リフィット (検証改善時のみ採用)。",
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
    print("  EML-SR Fable: Feynman Equations  [1.test_v5 noise=1%]")
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
        extra_baselines={"fable_v3_clean": FABLE_V3_RESULTS_PATH,
                         "fable_v4_noise": FABLE_V4_RESULTS_PATH},
        n_test=250,
        noise_rel=NOISE_REL,
    )


if __name__ == "__main__":
    main()
