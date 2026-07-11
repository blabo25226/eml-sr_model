"""
pure_eml_test1.py
======================================================
【pure_eml.test1】純 EML モード — Feynman 99 式 + 1% ガウシアンノイズ

Stage A/A2/C を一切使わず、EML(x,y)=e^x−ln(y) と算術接着 (+,×,Neg,定数)
だけで構成する pure_eml パイプラインの本番評価。

  - EML 基底ブースティング f = c0 + Σ cₖ·EML(Pₖ,Qₖ) (log 拡張アフィン引数)
  - 純 EML 文法ビームサーチ (先読み採点・挙動多様性・幅再拡張)
  - MAX_COMPLEXITY=10 (演算子 4 種で分岐係数が小さいため深く探索)
  - その他の条件は 1.test 系と同一 (BEAM=1000, N=750, 110s/式,
    1% ノイズ seed=777+行番号, ホールドアウト 250 点, σ 比例閾値)

参照値: 原初系 first_AI 9/99・cursor 13/99 (クリーン)、
フル fable v5 71/99 (同ノイズ条件)。

使い方:
  python3 pure_eml_test1.py            # 全 99 式
  python3 pure_eml_test1.py --smoke    # スモーク 3 式
======================================================
"""

import os
import sys

from feynman_fable_common import PROJECT_ROOT, run_benchmark

N_SAMPLES = 750
NOISE_REL = 1e-2

SEARCHER_PARAMS = dict(
    max_complexity=10,
    complexity_penalty=0.08,
    beam_width=1000,
    time_budget_s=110.0,
    subsample_size=256,
    early_exit_threshold=9e-3,
    refinement_top_k=8,
    snap_constants=True,
    powerlaw_stage=False,
    ratio_search=False,
    rational_stage=False,
    affine_scaling=True,
    max_boost_terms=6,
    verbose=False,
    pure_eml=True,
)

RESULTS_PATH = os.path.join(PROJECT_ROOT, "results", "eml_sr_pure_feynman_test1_results.json")
REPORT_PATH  = os.path.join(PROJECT_ROOT, "texts", "eml_sr_pure_feynman_test1_report.md")

FABLE_V5_RESULTS_PATH = os.path.join(
    PROJECT_ROOT, "results", "eml_sr_fable_feynman_test1_v5_results.json")

SMOKE_IDS = ["I.12.1", "I.12.5", "I.14.3"]

TITLE = "EML-SR Pure — Feynman Equations pure_eml.test1 (1%ノイズ, EML+算術接着のみ)"

DESCRIPTION = [
    "**純 EML モード**: Stage A/A2/C を一切使わず、関数プリミティブは",
    "EML(x,y)=e^x−ln(y) のみ (接着は +,×,Neg,定数)。",
    "",
    "- **EML 基底ブースティング**: f = c0 + Σ cₖ·EML(Pₖ,Qₖ)、",
    "  Pₖ = a0+Σaᵢxᵢ+Σbᵢ·ln xᵢ (ln も EML 合成)。e^{Pₖ} が単項式×指数クラスを、",
    "  −ln Qₖ が対数クラスを覆う。多スタート LM + 検証分割受理。",
    "- **純 EML 文法ビーム**: 演算子 4 種 (EML/Plus/Times/Neg) で複雑度 10 まで。",
    "  ラッパー先読み採点・挙動多様性キャップ・残余時間の幅再拡張。",
    "- sin/cos 等の振動構造は実数 EML では原理的に表現不能 (既知の限界)。",
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
    print("  EML-SR Pure: Feynman Equations  [pure_eml.test1 noise=1%]")
    print(f"  BEAM_WIDTH={SEARCHER_PARAMS['beam_width']}  N_SAMPLES={N_SAMPLES}  "
          f"MAX_COMPLEXITY={SEARCHER_PARAMS['max_complexity']}  PURE_EML=True")
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
        extra_baselines={"fable_v5_noise": FABLE_V5_RESULTS_PATH},
        n_test=250,
        noise_rel=NOISE_REL,
    )


if __name__ == "__main__":
    main()
