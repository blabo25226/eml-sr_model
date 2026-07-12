"""
pure_eml_test3_v2.py
======================================================
【pure_eml.test3_v2】test3_v2 と同一の 28 式 (A9 新関数 + B9 組合せ +
C10 スパース性) + 1% ガウシアンノイズを、**純 EML モード** (pure_eml=True、
Stage A/A2/C なし、EML+算術接着のみ) で実行する。

フル版 test3_v2 (v5: 25/28) との比較用。純 EML は sin/cos/abs/min/max 等の
振動・非平滑構造を実数 EML では原理的に表現できないため、A/B の該当式は
落ちる想定。C (スパース性・単項式/和) は EML 基底で回収できる見込み。

  - 探索条件: pure_eml=True, MAX_COMPLEXITY=10, BEAM=1000, 110s/式,
    Stage A/A2/C は off, early_exit=9e-3 (ノイズ)
  - ノイズ・判定規約は test3_v2 と同一 (共通モジュールを流用)

使い方:
  python3 pure_eml_test3_v2.py            # 全 28 式
  python3 pure_eml_test3_v2.py --smoke    # スモーク 3 式
======================================================
"""

import os
import sys

from feynman_fable_common import PROJECT_ROOT
from eml_sr_fable_test3_common import run_suite

NOISE_REL = 0.01

RESULTS_PATH = os.path.join(PROJECT_ROOT, "results", "eml_sr_pure_test3_v2_results.json")
REPORT_PATH  = os.path.join(PROJECT_ROOT, "texts", "eml_sr_pure_test3_v2_report.md")

SMOKE_IDS = ["A2_abs_affine", "B3_sigmoid_prod", "C1_b2_a5_sigmoid"]

# 純 EML モード用のオーバーライド (BASE_SEARCHER_PARAMS を上書き)
PURE_OVERRIDE = dict(
    pure_eml=True,
    max_complexity=10,       # 演算子4種なので深めに
    powerlaw_stage=False,    # Stage A off
    ratio_search=False,      # Stage C off
    rational_stage=False,    # Stage A2 off
)

TITLE = "EML-SR Pure — test3_v2: v4 演算子拡張ベンチ + 1%ノイズ (純 EML モード)"

DESCRIPTION = [
    "**純 EML モード** (pure_eml=True、Stage A/A2/C なし、関数プリミティブは",
    "EML(x,y)=e^x−ln(y) のみ) で test3_v2 と同一の 28 式 (A9 + B9 + C10) を実行。",
    "ノイズ・判定規約はフル版 test3_v2 と同一。フル版 (v5: 25/28) との比較用。",
    "",
    "純 EML は sin/cos/abs/min/max 等の振動・非平滑構造を実数 EML では",
    "原理的に表現できないため、A/B の該当式は落ちる想定。C (スパース性・",
    "単項式/和) は EML 基底ブースティングで回収できる見込み。",
]


def main():
    smoke = "--smoke" in sys.argv
    results_path, report_path = RESULTS_PATH, REPORT_PATH
    if smoke:
        results_path = results_path.replace(".json", "_smoke.json")
        report_path  = report_path.replace(".md", "_smoke.md")

    run_suite(
        noise_rel=NOISE_REL,
        results_path=results_path,
        report_path=report_path,
        title=TITLE,
        description_lines=DESCRIPTION,
        searcher_params=PURE_OVERRIDE,
        smoke_ids=SMOKE_IDS if smoke else None,
    )


if __name__ == "__main__":
    main()
