# eml-sr_fable 作業計画書 — 高精度 EML シンボリック回帰アルゴリズム

## 1. Context（背景と目的）

`plan_eml-sr_fable.md` の指示に基づき、既存の `eml-sr_model_cursor`（Rust + PyO3、EML演算子ベースのビームサーチSR）を土台に、**精度を大幅に高めた新アルゴリズム `eml-sr_fable`** を構築する。

### 現状分析（cursor 版の実測）
- Feynman 99式ベンチマーク: **完全回収 13式 (13.1%)**、総実行時間 **819分**
- 失敗の主要因（レポート・コード精査による分類）:
  1. **べき単項式** $c \cdot \prod x_i^{a_i}$ 型（例: $q_1 q_2/(4\pi\epsilon r^2)$）が Feynman の**約3割**を占めるのに、定数 $c$ と累乗の組合せが complexity=6 に収まらない
  2. **スケール定数の欠如**: `Times(v0,v1)` は当たるが $\frac{3}{2} pr V$ の $3/2$ が乗らない（定数ノード追加で complexity 超過）
  3. **早期終了なし**: `find_candidates` は厳密解発見後も complexity 6 まで全探索（`I.12.1` は解発見後も計1354秒）
  4. **アフィン等価な候補の重複**: `f`, `Neg(f)`, `Times(C,f)`, `Plus(C,f)` が別候補としてビームを浪費
  5. Lorentz 因子 $1/\sqrt{1-v^2/c^2}$ や指数型 $e^{-mgx/k_bT}$ は complexity ≥ 7〜8 が必要で構造的に到達不能

## 2. 新アルゴリズム eml-sr_fable の仕組み

**EML演算子 $\mathrm{EML}(x,y)=e^x-\ln(y)$ を含む演算子集合・式表現・ビームサーチの枠組みは維持**したまま、3段パイプラインに再構成する。

### Stage A: べき単項式ソルバー + 残差ブースティング（閉形式・ミリ秒）
- 変換ターゲット $t(y) \in \{y,\ \log y,\ 1/y,\ 1/y^2,\ y^2\}$ それぞれに対し、
  $\log|t(y)| = \log c + \sum_i a_i \log x_i$ を**最小二乗（正規方程式）**で解く
- 指数 $a_i$ を有理数（分母≤2、必要に応じ3）に丸め → 係数 $c$ を再フィット → 元スケール $y$ での RMSE を厳密検証
- **残差ブースティング**: $y \approx \sum_{k=1}^{K} c_k \prod_i x_i^{a_{ki}}$（K≤3）。残差に単項式フィットを繰り返す
  - これで $x_1y_1+x_2y_2+x_3y_3$ (I.11.19) や $\frac{1}{2}m(v^2+u^2+w^2)$ (I.13.4)、$1/y^2 = 1/m_0^2 - v^2/(m_0^2c^2)$ 経由の Lorentz 因子 (I.10.7 等) が閉形式で解ける
- 得られた式は既存の式表現（Times/Divide/Square/Cube/Sqrt/Pow ノード）の木として組み立てる

### Stage B: 改良 EML ビームサーチ
cursor の BFS を以下で強化:
1. **アフィンスケーリング評価（Keijzer 線形スケーリング）**: 各候補構造 $f$ を $\min_{a,b}\mathrm{RMSE}(y,\ a f+b)$ で採点（閉形式2パラメータ最小二乗）。定数の LM 最適化なしで「定数×構造+オフセット」を無料で獲得 → 実効表現力が complexity+3 相当向上。最終式は $a\approx1$/$b\approx0$ なら省いて組み立て
2. **標準化フィンガープリント**: 予測ベクトルを平均0・ノルム1に標準化してから量子化 → アフィン等価候補（`Neg(f)` 等）がビーム上で1つに縮約され、ビーム実効幅が拡大＆高速化
3. **早期終了の復活**: Pareto front に RMSE < 1e-9 が現れたら残りレベルを打ち切り（`find_candidates` でも有効化）
4. **サブサンプル評価**: 探索中は固定シードで選んだ ≤256 点で採点、最終上位候補のみ全 750 点で再評価・再最適化
5. **LM の適用限定**: 内部パラメータ（非線形の内側の C）を持つ候補のみ LM（反復15）。末尾で上位 K=8 をマルチスタート LM（反復80）
6. **定数スナップ**: 最適化後の定数を整数・単純分数・$\pi$ 倍数へ丸めて RMSE が悪化しなければ採用（1e-4 閾値の完全回収を後押し）
7. **式ごとの時間予算**: レベル間で壁時計チェック（test1 既定 ~120s/式）し超過なら打ち切り

### Stage C: 乗法分解（ratio search）
- Stage A の単項式フィットが「高相関だが不十分」（例 $R^2>0.5$）な場合、比 $y' = y / (c\prod x^{a})$ を新ターゲットとして Stage B を実行
- 比は通常少数の無次元量（$v/c$ 等）にのみ依存するため complexity 6 で $\sin$, $\exp$, $\mathrm{EML}$ 等の補正構造に到達可能（例: $n_0 e^{-mgx/k_bT}$, $I_0\,\mathrm{sinc}$ 型）
- 最終式 = 単項式 × 補正構造。通常ターゲット $y$ への Stage B も並行実施し、全ステージの Pareto front を統合して返す

### 注記（仕様上の明示事項）
- **MAX_COMPLEXITY=6 はビームサーチの構造部分に適用**する。アフィンスケーリング係数や Stage A/C で組み上がる最終式のノード数は 6 を超えうる（魔改造の範囲、レポートには実ノード数を記載）
- 乱数はデータ生成（seed=42+行番号、cursor と同一）・サブサンプル選択（固定シード）とも完全固定し再現性を保証
- `to_python()` / `.formula` は **最適化済み定数を数値で埋め込んで出力**するよう改良（現状は `p_{0}` のまま表示され検証しづらい）

## 3. 実装ステップ

### 3.1 クレートのフォーク
- `eml-sr_model_cursor/` を `eml-sr_fable/` へコピー、`Cargo.toml`/`pyproject.toml` を `eml_sr_fable`（crate名・Pythonモジュール名 `eml_sr_fable`）へリネーム
- この計画書を `eml-sr_fable/WORK_PLAN.md` として保存

### 3.2 Rust コア変更（主要ファイル）
| ファイル | 変更 |
|---|---|
| `src/engine/powerlaw.rs` (新規) | Stage A: log-空間最小二乗・指数丸め・残差ブースティング・式木組み立て |
| `src/engine/bfs.rs` | アフィン採点、早期終了、サブサンプル、時間予算、定数スナップ、Stage 統合の `run_fable` |
| `src/core/signature.rs` | 標準化フィンガープリント（データ点ベース or プローブ点の標準化） |
| `src/engine/optimizer.rs` | LM 反復数の段階化・スナップ後再検証 |
| `src/config.rs` | `subsample_size`, `time_budget_s`, `early_exit_threshold`, `snap_constants` 等を追加 |
| `src/python.rs` | 新 config の公開、`Searcher.find_candidates` が fable パイプラインを呼ぶよう変更、数値定数の文字列出力 |

既存の `cargo test` を維持しつつ powerlaw/アフィン採点の単体テストを追加。

### 3.3 テストスクリプト（別ファイル・シード固定）
- `src/feynman_eml_sr_fable_test1.py`: **BEAM_WIDTH=1000, N_SAMPLES=750, MAX_COMPLEXITY=6**、データseed=42+idx。評価は cursor と同一（RMSE<1e-4 で ok、<1e-2 で partial、first_AI/cursor 比較指標付き）。結果→`results/eml_sr_fable_feynman_test1_results.json`、レポート→`texts/eml_sr_fable_feynman_test1_report.md`
- `src/feynman_eml_sr_fable_test2.py`: 条件緩和版（MAX_COMPLEXITY=8, BEAM_WIDTH=2000, 時間予算拡大, N_SAMPLES=1500）。ユーザーが高性能PCで実行する用

### 3.4 実行と成果物
1. `maturin develop --release` でビルド → `cargo test` → 3式スモーク
2. **1.test を本環境で完全実行**（99式、目標: 総計 ≤ 2時間 / 式あたり時間予算で上限保証）
3. レポート生成（結果・変更点・アルゴリズム説明を含む）→ `texts/`
4. `daily_report.md` に時系列で作業記録を追記
5. git add/commit/push（ブランチ `claude/eml-sr-fable-algorithm-7w8j49`）→ draft PR 作成

## 4. 検証方法
- `cargo test`（既存+新規単体テスト）
- スモーク3式（`I.12.1`, `I.12.5`, `I.14.3`）で回帰なしを確認
- 単項式代表（`I.12.2`）と Lorentz 型（`I.10.7`）を単発検証し Stage A/C の効果を直接確認
- 全99式 test1 実行 → ok 数を cursor(13) と比較。**目標: 完全回収 ≥ 30式**、回帰（lost_ok_ids）ゼロ、総時間 ≤ 約2時間

## 5. 期待効果（見積り）
| 改善 | 対象 | 期待増分 |
|---|---|---|
| Stage A 単項式+ブースト | 純単項式 ~25-30式 + 多項式型 | +20〜25 |
| アフィンスケーリング | 定数係数付き構造 | +3〜6 |
| Stage C 比探索 | 指数減衰・Lorentz・sinc 型 | +3〜6 |
| 早期終了+サブサンプル+縮約 | 全体 | 819分 → **1〜2時間** |
