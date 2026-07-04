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


