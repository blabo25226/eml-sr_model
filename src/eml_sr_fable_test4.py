"""
eml_sr_fable_test4.py
======================================================
【test4】外れ値頑健性ベンチ — test3 と同一の 28 式に
**1% ガウシアンノイズ + 5% の外れ値 (振幅 10σ)** を注入。

  - ノイズ: y += N(0, (0.01·std(y))²), seed=777+行番号
  - 外れ値: 訓練点の 5% に ±10σ を加算, seed=555+行番号
  - 合否閾値はガウシアン σ 基準のまま (外れ値では緩めない)
  - 合否判定はクリーンなホールドアウト 250 点のテスト RMSE

実データの混入外れ値 (センサー異常・記録ミス) への頑健性を測る。
v4/v5 エンジンの比較には --out-suffix を使う。

使い方:
  python3 eml_sr_fable_test4.py --out-suffix v4base   # v4 wheel で基準値
  python3 eml_sr_fable_test4.py --out-suffix v5       # v5 wheel で比較
  python3 eml_sr_fable_test4.py --smoke               # スモーク 3 式
======================================================
"""

import os
import sys

from feynman_fable_common import PROJECT_ROOT
from eml_sr_fable_test3_common import run_suite

NOISE_REL     = 0.01
OUTLIER_FRAC  = 0.05
OUTLIER_SCALE = 10.0

RESULTS_PATH = os.path.join(PROJECT_ROOT, "results", "eml_sr_fable_test4_results.json")
REPORT_PATH  = os.path.join(PROJECT_ROOT, "texts", "eml_sr_fable_test4_report.md")

SMOKE_IDS = ["A2_abs_affine", "B3_sigmoid_prod", "C1_b2_a5_sigmoid"]

TITLE = "EML-SR Fable — test4: 外れ値頑健性ベンチ (1%ノイズ + 5%外れ値 10σ)"

DESCRIPTION = [
    "test3 と同一の 28 式 (A9 + B9 + C10) に対し、1% ガウシアンノイズに加えて",
    "**訓練点の 5% に ±10σ の外れ値** (seed=555+行番号) を注入。",
    "実データの混入外れ値への頑健性を測る。合否判定はクリーンな",
    "ホールドアウト 250 点、閾値はガウシアン σ 基準のまま。",
]


def main():
    smoke = "--smoke" in sys.argv
    suffix = None
    if "--out-suffix" in sys.argv:
        suffix = sys.argv[sys.argv.index("--out-suffix") + 1]
    results_path, report_path = RESULTS_PATH, REPORT_PATH
    if suffix:
        results_path = results_path.replace(".json", f"_{suffix}.json")
        report_path  = report_path.replace(".md", f"_{suffix}.md")
    if smoke:
        results_path = results_path.replace(".json", "_smoke.json")
        report_path  = report_path.replace(".md", "_smoke.md")

    run_suite(
        noise_rel=NOISE_REL,
        results_path=results_path,
        report_path=report_path,
        title=TITLE,
        description_lines=DESCRIPTION,
        smoke_ids=SMOKE_IDS if smoke else None,
        outlier_frac=OUTLIER_FRAC,
        outlier_scale=OUTLIER_SCALE,
    )


if __name__ == "__main__":
    main()
