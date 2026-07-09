"""
eml_sr_fable_test3_v2.py
======================================================
【test3_v2】test3 と同一の 28 式 + 訓練ターゲットに 1% ガウシアンノイズ。

  - y += N(0, (0.01·std(y))²), seed=777+行番号 (既存ノイズ規約と同一)
  - early_exit_threshold=9e-3 (ノイズ床でも早期終了できる水準)
  - 合否閾値は σ 比例: ok < max(1e-4, 0.15σ), partial < max(1e-2, 0.5σ)
  - 合否判定はクリーンなホールドアウト 250 点のテスト RMSE

使い方:
  python3 eml_sr_fable_test3_v2.py            # 全 28 式
  python3 eml_sr_fable_test3_v2.py --smoke    # スモーク 3 式
======================================================
"""

import os
import sys

from feynman_fable_common import PROJECT_ROOT
from eml_sr_fable_test3_common import run_suite

NOISE_REL = 0.01

RESULTS_PATH = os.path.join(PROJECT_ROOT, "results", "eml_sr_fable_test3_v2_results.json")
REPORT_PATH  = os.path.join(PROJECT_ROOT, "texts", "eml_sr_fable_test3_v2_report.md")

SMOKE_IDS = ["A2_abs_affine", "B3_sigmoid_prod", "C1_b2_a5_sigmoid"]

TITLE = "EML-SR Fable — test3_v2: v4 演算子拡張ベンチ + 1% ガウシアンノイズ"

DESCRIPTION = [
    "test3 と同一の 28 式 (A9 + B9 + C10) に対し、訓練ターゲットへ",
    "**1% ガウシアンノイズ** y += N(0, (0.01·std(y))²) (seed=777+行番号) を注入。",
    "合否判定はクリーンなホールドアウト 250 点のテスト RMSE、閾値は σ 比例",
    "(ok < max(1e-4, 0.15σ), partial < max(1e-2, 0.5σ))。",
    "",
    "ノイズ下でも新演算子構造 (abs/sigmoid/min/max/連続指数) と",
    "スパース性の変数特定が維持されるかを検証する。",
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
        smoke_ids=SMOKE_IDS if smoke else None,
    )


if __name__ == "__main__":
    main()
