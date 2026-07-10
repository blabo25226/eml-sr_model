"""
eml_sr_fable_test3.py
======================================================
【test3】v4 演算子拡張 (abs / sigmoid / min / max / 連続指数 Pow) の
検証ベンチマーク — ノイズなし。

28 式 = A. 新関数の直接テスト (9) + B. 組合せ・改造 (9)
      + C. スパース性 (10: a 変数中 b<a 変数のみ使用、変数特定を計測)。

計算量は 1.test と同一 (BEAM=1000, N=750, CPLX=6, 110s/式)、
ホールドアウト 250 点 (seed 分離) のテスト RMSE で合否判定。

使い方:
  python3 eml_sr_fable_test3.py            # 全 28 式
  python3 eml_sr_fable_test3.py --smoke    # スモーク 3 式
======================================================
"""

import os
import sys

from feynman_fable_common import PROJECT_ROOT
from eml_sr_fable_test3_common import run_suite

RESULTS_PATH = os.path.join(PROJECT_ROOT, "results", "eml_sr_fable_test3_results.json")
REPORT_PATH  = os.path.join(PROJECT_ROOT, "texts", "eml_sr_fable_test3_report.md")

SMOKE_IDS = ["A2_abs_affine", "B3_sigmoid_prod", "C1_b2_a5_sigmoid"]

TITLE = "EML-SR Fable — test3: v4 演算子拡張ベンチ (BEAM=1000, N=750, CPLX=6)"

DESCRIPTION = [
    "v4 で追加した演算子 (Abs / Sigmoid / Min / Max) と既存の連続指数 Pow を、",
    "新関数の直接テスト (A, 9式)・組合せ改造 (B, 9式)・スパース性 (C, 10式) の",
    "計 28 式で検証。探索条件は 1.test と同一、ホールドアウト 250 点で合否判定。",
    "",
    "C はデータが a 変数、真の f は b<a 変数のみ使用 ((b,a) = (2,5), (3,7),",
    "(4,20), (5,15), (6,20), (2,10), (3,12), (2,8), (4,10), (5,25))。",
    "合否とは別に「変数特定」(発見式の変数集合 == 真の使用集合) を集計する。",
]


def main():
    smoke = "--smoke" in sys.argv
    results_path, report_path = RESULTS_PATH, REPORT_PATH
    if smoke:
        results_path = results_path.replace(".json", "_smoke.json")
        report_path  = report_path.replace(".md", "_smoke.md")

    run_suite(
        noise_rel=0.0,
        results_path=results_path,
        report_path=report_path,
        title=TITLE,
        description_lines=DESCRIPTION,
        smoke_ids=SMOKE_IDS if smoke else None,
    )


if __name__ == "__main__":
    main()
