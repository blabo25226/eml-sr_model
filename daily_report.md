## 2026/07/04 (eml-sr_fable 開発)

### AI（エージェント・Claude Fable）が行ったこと
- `eml-sr_fable/plan_eml-sr_fable.md` の指示書を読み込み、cursor 版のコード・レポート・失敗パターンを精査して**作業計画書を作成** → ユーザー承諾後にコーディング開始
- **`eml-sr_model_cursor/` を `eml-sr_fable/` へフォーク**し、クレート名・Python モジュール名を `eml_sr_fable` にリネーム。計画書を `eml-sr_fable/WORK_PLAN.md` として保存
- **Stage A: べき単項式ソルバーを新規実装**（`src/engine/powerlaw.rs`）
  - 変換ターゲット {y, log y, 1/y, 1/y², y²} × (log 空間最小二乗 + OLS 辞書追跡 + バックフィッティング + 後方剪定)
  - 開発中に発見した問題を順次修正: OMP の妥協列問題 → OLS 化、係数二重適用バグ、定数シード戦略（あり/なし両方を試行）、整数指数辞書の追加（x1y1+x2y2+x3y3 型に決定的）
- **Stage B: ビームサーチを改良**（`src/engine/bfs.rs` 全面改修）
  - アフィンスケーリング採点（min_{a,b} RMSE(y, a·f+b) 閉形式）、アフィン同値クラスのビーム占有制限、早期終了、決定的サブサンプル評価、式あたり時間予算、定数スナップ（整数・分数・π倍数, `optimizer.rs`）
- **Stage C: 乗法分解を新規実装**（`src/engine/fable.rs`）: 単項式ホワイトナーで割った比のビームサーチ → 積として合成。3 段を統合する `run_fable` パイプラインを構築
- **バグ修正**: `to_python()` の演算子優先順位（Divide/Inv/Neg、cursor から継承）、最適化定数が `p_{0}` のまま出力される問題（数値埋め込みに変更）
- `cargo test` 全通過（powerlaw 単体テスト 5 件追加）、maturin で wheel ビルド
- テストスクリプト作成: `src/feynman_eml_sr_fable_test1.py`（BEAM=1000, N=750, CPLX=6, 時間予算110s/式）、`src/feynman_eml_sr_fable_test2.py`（緩和条件 BEAM=2000, N=1500, CPLX=8, 600s/式, 高性能PC用）。シードは cursor と同一の 42+行番号で固定
- スモーク 3 式（I.12.1/I.12.5/I.14.3）: **3/3 回収、計 0.2 秒**（cursor は同 3 式で約 58 分）
- **全 99 式 1.test 本番ベンチマーク完了**（コンテナ再起動により 1 回中断 → 再実行）
  - **結果: 完全回収 63式 (63.6%)、部分回収 10式、合計 73式 (73.7%)、失敗 21式、スキップ 5式**
  - **総実行時間 19.0 分**（cursor: 819 分の約 1/43）、1 式あたり中央値 1.1 秒
  - **first_AI 比 delta_ok = +54、cursor 比 delta_ok = +50、回帰ゼロ**（lost_ok_ids = ∅）
  - 出力: `results/eml_sr_fable_feynman_test1_results.json`, `texts/eml_sr_fable_feynman_test1_report.md`, `results/feynman_fable_test1.log`
- 成果物を git commit → push（ブランチ `claude/eml-sr-fable-algorithm-7w8j49`）、PR #1 を作成

### ユーザーが行ったこと
- eml-sr_fable の作業指示書を提示し、作業計画書を承諾
- PR #1 を draft からレビュー可能状態に変更

---

## 2026/07/04

### AI（エージェント）が行ったこと
- **全99式 本番ベンチマーク完了**（`eml-sr_model_cursor`, release ビルド, MAX_COMPLEXITY=6, BEAM_WIDTH=1000, PENALTY=0.08, 約819分）
  - **結果:** 完全回収 **13式 (13.1%)**、部分回収 3式、合計 16式 (16.2%)、スキップ5式、失敗78式
  - **first_AI 比:** delta_ok = **+4**、回帰なし（lost_ok_ids = ∅、既存9式すべて維持）
  - **新規回収4式:** `I.34.27`, `I.39.1`, `II.27.16`, `III.12.43`
  - 新演算子 `Square` が `II.27.16`（$q^2$ 依存）・`II.27.18`（$\epsilon E^2$）の回収に直接寄与
  - 出力: `results/eml_sr_model_cursor_feynman_results.json`, `texts/eml_sr_model_cursor_feynman_report.md`, `results/feynman_cursor_full.log`
- レポート雛形の記述を Phase F 後の実設定（complexity=6, 単一Paramシード, LM 二段構成）に合わせて修正し、保存済み JSON から再生成
- 目標達成: 最低基準（≥10, 回帰なし）および目標（≥12）を満たした。全成果を git commit → push（ユーザー許可済み）

### ユーザーが行ったこと
- 外出前に「99式計算完了後、レポート等を済ませて git push」を指示・許可（今回限りの push 承認）

---

## 2026/07/03

### AI（エージェント）が行ったこと
- `sample_code/gh3_allfunc_moreestimate_sr.py` を参考に、新モデル `eml-sr_model_first_AI` 用の Feynman 全式分析スクリプト `feynman_eml_sr_model_first_AI.py` を作成
- 実験設定: N_SAMPLES=750, BEAM_WIDTH=1000, MAX_COMPLEXITY=6（従来 eml-sr 実験と同一）
- 先頭3式での動作確認を実施（スクリプト・レポート生成の正常動作を確認）
- 全99式の本番実行を完了（約176分）
  - **結果:** 完全回収 9式 (9.1%)、部分回収 1式、合計 10式 (10.1%) — 従来 eml-sr と同一回収率
  - **出力:** `results/eml_sr_model_first_AI_feynman_results.json`, `texts/eml_sr_model_first_AI_feynman_report.md`
- 演算子拡張の検討を実施し、`WORK_PLAN.md` §3.3 に反映（Square/Cube/制限付き Pow を必須、Abs/Tanh 等は見送り）
- **速度低下の原因究明**: cursor 版が同一設定でも first_AI 比 ~200x 遅い（Level 5 で 37分）現象をコード精査。主因は (1) 複数 Param シード（$0.5,1.0,3.0$）が異なるフィンガープリントとなり重複除去されず候補が指数増殖、(2) BFS 内 LM 反復が 20→80 で4倍。`WORK_PLAN.md` に **Phase F: 速度改善計画** を追記
- **Phase F 実装（承諾後）**: `config.rs` の `param_seed_inits=[1.0]`（1個に）、`optimizer_max_iters=25`（BFS用）、`refine_max_iters=80`（新設・末尾refinement用）。`bfs.rs` の refinement pass を `refine_max_iters` 使用に変更。release ビルドで `maturin develop` 成功
- **単発検証（I.12.1）**: Level 5 が 2246秒→34秒（~65x高速化）、全体 252秒（4.2分）で `Times(v_0,v_1)` を RMSE 0 回収（目標<5分達成）
- **3式スモーク成功**: `I.12.1`,`I.12.5`,`I.14.3` を全て回収（3/3, RMSE≤3e-15, 回帰なし delta_ok=0）。ただし3変数 `I.14.3` は Level 6 が重く 1323秒（22分）を要した
- スモーク比較指標を部分実行対応に修正（`compare_with_baseline` に `attempted_ids` を追加、未実行式の lost 誤検出を防止）

### ユーザーが行ったこと
- `sample_code/` の従来 eml-sr 分析と `data/` の Feynman データを用い、新モデル `eml-sr_model_first_AI` で同等分析を依頼
- 精度改良版 `eml-sr_model_cursor` の作成を依頼（目標: Feynman 回収 9式超）。まず作業計画書の作成のみ（コーディング禁止）
- 作業計画の方針を指示: MAX_COMPLEXITY=7 可、定数は $0.5$, $3.1$ 等の数値近似で合格、cursor 専用設定許容、特定式の優先なし（全体回収最大化）
- **作業計画を承諾** → `eml-sr_model_cursor` の実装を着手
- Phase 0〜D 完了: first_AI を fork、Square/Cube/制限付き Pow、複数 Param シード、真の LM、complexity=7、refinement pass、beam=1200 等を実装
- `cargo test` 成功、`maturin develop --features python,full-math` 成功
- Phase E 着手: `src/feynman_eml_sr_model_cursor.py` を作成（first_AI 比較指標付き）
- スモークテスト 15式を実行開始 → 1式目に数時間かかるため **停止**
- 方針変更: `--smoke` モード追加（first_AI 同一設定 MAX_COMPLEXITY=6, BEAM_WIDTH=1000, penalty=0.1、baseline ok 9式のみ）で再実行開始
- スモーク再変更: 実行中のスモークをキャンセル。本番・スモーク共通で MAX_COMPLEXITY=6, BEAM_WIDTH=1000, COMPLEXITY_PENALTY=0.08。スモークは first_AI ok 式の CSV 順先頭 3 式（`I.12.1`, `I.12.5`, `I.14.3`）


