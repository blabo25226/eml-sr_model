"""
feynman_eml_sr_fable_test1_v4.py
======================================================
Feynman 方程式ベンチマーク — eml-sr_fable 【1.test_v4 誤差付き】

pointout_cursor.md の指摘対応版 (v3 アルゴリズム) のテスト。
探索条件は 1.test と同一 + ホールドアウト評価を導入:
  - BEAM_WIDTH=1000, N_SAMPLES=750, MAX_COMPLEXITY=6, 110s/式
  - 訓練: seed=42+行番号 / テスト: seed=4242+行番号 の新規 250 点
  - 合否判定は「訓練で選んだ最良候補のテスト RMSE」(過適合を検出)

v3 のアルゴリズム変更:
  1. 変数集合の分離 (整数べき・trig・差分特徴は負値変数でも使用可)
  2. 有理関数ステージ (y·Q=P の線形化、I.16.6/I.18.4 型)
  3. Logit 変換 (ロジスティック族の線形化)
  4. 検証分割 (80/20) による項採択・剪定 (ノイズ過適合の抑止)
  5. 候補選択の節約的タイブレーク (訓練RMSE 5%帯内で最小複雑度)

使い方:
  python3 feynman_eml_sr_fable_test1_v4.py            # 全 99 式
  python3 feynman_eml_sr_fable_test1_v4.py --smoke    # スモーク 3 式
======================================================
"""

import os
import sys

from feynman_fable_common import PROJECT_ROOT, run_benchmark

# ===== 1.test_v2 実験設定（1.test と同一） =====
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
    affine_scaling=True,
    max_boost_terms=6,
    verbose=False,
)

RESULTS_PATH = os.path.join(PROJECT_ROOT, "results", "eml_sr_fable_feynman_test1_v4_results.json")
REPORT_PATH  = os.path.join(PROJECT_ROOT, "texts", "eml_sr_fable_feynman_test1_v4_report.md")

FABLE_V1_RESULTS_PATH = os.path.join(
    PROJECT_ROOT, "results", "eml_sr_fable_feynman_test1_results.json")
FABLE_V2_RESULTS_PATH = os.path.join(
    PROJECT_ROOT, "results", "eml_sr_fable_feynman_test1_v2_results.json")

SMOKE_IDS = ["I.12.1", "I.12.5", "I.14.3"]

TITLE = "EML-SR Fable — Feynman Equations 1.test_v4 (1%ノイズ) レポート (BEAM=1000, N=750, CPLX=6)"

DESCRIPTION = [
    "**pointout_cursor.md の指摘対応版 (v3)** のテスト。探索条件は 1.test/1.test_v2 と同一だが、",
    "**ホールドアウト評価**（テスト点 250、seed=4242+行番号、クリーンなターゲットとの RMSE で合否判定）を導入。",
    "",
    "### v3 のアルゴリズム変更（汎用化・ノイズ頑健化）",
    "",
    "1. **変数集合の分離**: 整数べき・sin/cos・差分特徴は負値/符号混在の変数でも使用可能に",
    "（Feynman は全変数正値だが一般データは違う）。",
    "2. **有理関数ステージ**: y·Q(x)=P(x) を y を拡張入力列として辞書追跡で線形化",
    "（有理式 I.16.6, I.18.4 型。一般データの P/Q 形に対応）。",
    "3. **Logit 変換** t=ln((1−y)/y): ロジスティック/シグモイド族の線形化。",
    "4. **検証分割（80/20 決定的）による項採択・剪定**: 訓練 RMSE だけで項を増やさない。",
    "5. **候補選択の節約的タイブレーク**: 訓練 RMSE 最良から 5% 帯内で最小複雑度の候補を選択。",
    "",
    "",
    "### v4 のノイズ条件",
    "",
    "- 訓練ターゲットに y += N(0, (0.01·std(y))²) のガウスノイズを注入（seed=777+行番号）",
    "- 合否判定: 訓練で選んだ最良候補の **クリーンなテストターゲットに対する RMSE** が",
    "  ok < max(1e-4, 0.15σ) / partial < max(1e-2, 0.5σ)（σ はノイズの絶対水準）。",
    "  構造を正しく回収できていればノイズ床を大きく下回ることを利用した判定。",
    "- 早期終了閾値をノイズ床 (0.9×1%) に設定し、構造回収後の無駄な探索を打ち切る。",
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
    print("  EML-SR Fable: Feynman Equations  [1.test_v4 noise=1%]")
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
        extra_baselines={"fable_v1": FABLE_V1_RESULTS_PATH,
                         "fable_v2": FABLE_V2_RESULTS_PATH},
        n_test=250,
        noise_rel=NOISE_REL,
    )


if __name__ == "__main__":
    main()
