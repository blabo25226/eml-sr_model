"""
feynman_eml_sr_fable_test1_v2.py
======================================================
Feynman 方程式ベンチマーク — eml-sr_fable 【1.test_v2】

1.test の失敗 21 式の原因分析に基づくモデル改善後の再テスト。
探索条件・データ生成・乱数シードは 1.test と完全に同一:
  - BEAM_WIDTH     = 1000
  - N_SAMPLES      = 750
  - MAX_COMPLEXITY = 6
  - データ生成シード: 42 + CSV 行番号

v2 でのモデル改善（eml-sr_fable 本体を直接編集）:
  1. 特徴量拡張辞書: 単項式 × {sin, cos, sin2x, cos2x, ln, (xi-xj)²,
     cos(xi-xj), cos(xi·xj), sin(xi·xj)}（基本辞書で未解決のときのみ実行）
  2. 整数指数辞書の active 変数上限 3 → 5（G·m1·m2/r 型の 4 変数項）
  3. 後方剪定の強化（削除許容 1.25 倍 + 未剪定フォールバック）
     → junk な Plus 連鎖の抑制
  4. Log1p 変換（e^m − 1 型の線形化）
  5. 比ターゲットへの Stage A 適用 + 混合符号ホワイトナー + 候補 5 つのプローブ
  6. Tanh 演算子の登録（2.test 向け）

出力（v1 と分離）:
  - results/eml_sr_fable_feynman_test1_v2_results.json
  - texts/eml_sr_fable_feynman_test1_v2_report.md
比較対象: first_AI / cursor / fable_v1 (1.test)

使い方:
  python3 feynman_eml_sr_fable_test1_v2.py            # 全 99 式
  python3 feynman_eml_sr_fable_test1_v2.py 10         # 先頭 10 式のみ
  python3 feynman_eml_sr_fable_test1_v2.py --smoke    # スモーク 3 式
======================================================
"""

import os
import sys

from feynman_fable_common import PROJECT_ROOT, run_benchmark

# ===== 1.test_v2 実験設定（1.test と同一） =====
N_SAMPLES = 750

SEARCHER_PARAMS = dict(
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
    affine_scaling=True,
    max_boost_terms=6,
    verbose=False,
)

RESULTS_PATH = os.path.join(PROJECT_ROOT, "results", "eml_sr_fable_feynman_test1_v2_results.json")
REPORT_PATH  = os.path.join(PROJECT_ROOT, "texts", "eml_sr_fable_feynman_test1_v2_report.md")

FABLE_V1_RESULTS_PATH = os.path.join(
    PROJECT_ROOT, "results", "eml_sr_fable_feynman_test1_results.json")

SMOKE_IDS = ["I.12.1", "I.12.5", "I.14.3"]

TITLE = "EML-SR Fable — Feynman Equations 1.test_v2 レポート (BEAM=1000, N=750, CPLX=6)"

DESCRIPTION = [
    "**1.test 失敗 21 式の原因分析に基づくモデル改善（v2）** の再テスト。探索条件・シードは 1.test と同一。",
    "",
    "### v1 失敗の原因分析",
    "",
    "1. **辞書の欠落（主因）**: 真の式が「単項式×sin/cos/ln」の和（I.12.11, I.37.4, I.44.4 等）や",
    "「変数差・変数積の三角」（I.8.14, I.29.16, III.15.12 等）なのに Stage A の辞書が純単項式のみで、",
    "junk な単項式和（Plus 連鎖）で近似するしかなかった。",
    "2. **整数辞書の変数上限 3**: I.13.12 / II.2.42 の 4 変数項が辞書に入らず貪欲選択が迷走。",
    "3. **比探索の限界**: I.40.1 等は比が exp(単項式) 型なのに比にはビームサーチしか適用しておらず",
    "complexity 6 で届かない。",
    "4. **剪定が甘い**: わずかに寄与する junk 項を全て残すため Plus 連鎖が肥大化（ユーザー指摘）。",
    "",
    "### v2 の修正",
    "",
    "1. **特徴量拡張辞書**: 単項式 × {sin(x), cos(x), sin(2x), cos(2x), ln(x), (xi−xj)²,",
    "cos(xi−xj), cos(xi·xj), sin(xi·xj)} を第 2 パスの OLS 辞書として追加",
    "（サイズ上限 15 万列、指数集合と active 上限を自動選択）。",
    "2. **整数指数辞書の active 上限 3 → 5**。素の 1 変数特徴量列も整数辞書に同乗。",
    "3. **後方剪定の強化**: 「削除で RMSE が 1.25 倍以上悪化しない限り削除」に変更",
    "（厳密解の真項は削除で桁違いに悪化するため安全）。精度低下時は未剪定版もフォールバックとして併存。",
    "4. **Log1p 変換** t=ln(1+y) を追加（e^m − 1 型の線形化）。",
    "5. **比ターゲットへの Stage A 適用**: ホワイトナーを |y| ベースの混合符号対応にし、",
    "候補 5 つを閉形式プローブしてからビームサーチ。",
    "6. **Tanh 演算子の登録**（complexity 8 の 2.test で II.35.21 到達用）。",
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
    print("  EML-SR Fable: Feynman Equations  [1.test_v2]")
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
        extra_baselines={"fable_v1": FABLE_V1_RESULTS_PATH},
    )


if __name__ == "__main__":
    main()
