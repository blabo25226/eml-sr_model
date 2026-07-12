## 2026/07/11 (eml-sr_fable v6: 純 EML モード — Stage A/C なしで EML の組み合わせだけ)

### AI（エージェント・Claude Fable）が行ったこと
- ユーザーの問い「ステージ A/C がメインで EML を組み合わせる原初の仕組みが空洞化している。原初の eml-sr を工夫して精度を上げられないか」を実測で確認 → **採用式に EML 演算子が現れる式は 0/99** だった
- ユーザー指示「Stage A/C を完全に削除して EML の組み合わせだけで精度の高いアルゴリズム」に対応する **純 EML モード (`pure_eml=True`)** を新設 (main から新ブランチ、既存 fable は無変更)
  - `OperatorRegistry::pure_eml()`: 関数プリミティブは EML(x,y)=e^x−ln(y) のみ、接着は Plus/Times/Neg/定数 (exp/log/べき/除算はすべて EML 合成)
  - `engine/pure_eml.rs`: EML 基底ブースティング f = c0 + Σ cₖ·EML(Pₖ,Qₖ) (log 拡張アフィン引数、多スタート LM + 検証分割受理) + 純 EML 文法ビーム (先読み採点・挙動多様性・幅再拡張) + 共通後段
  - 構造バリアント分離 (純単項式/アフィン指数/ガウス族 x+x²/混在) と入れ子 EML ユニット c·e^{Σb·ln x + a1·∏x^c} を追加 (n0·e^{−mgx/kbT} 族)
  - config/python に pure_eml / pure_basis_stage / lookahead_scoring / behavior_cap を追加、通常経路は不変。単体テスト 7 件追加で計 37 件全通過
- **本番結果 (Feynman 99式 + 1%ノイズ、EML+算術接着のみ): 31/99 (ok+partial 51/99)、187分**
  - 原初系 first_AI 9/99・cursor 13/99 (いずれもクリーン) の **2.4〜3.4 倍**、しかもノイズ下
  - **採用式 79/99 に EML(…) が明示的に出現 (0→79)、純文法違反ゼロ**
  - アブレーション: 基底ブースティングの寄与は Feynman30 で 4→7、一般で 6→7
  - I.40.1 は入れ子 EML ユニットでクリーン誤差 1.5e-1→1.8e-3 (フル fable v5 の failed を上回る)
  - 失敗43式中14式は三角関数 (実数 EML では原理的に表現不能) → 純 EML の理論上限 ~85/99
- レポート `texts/eml_sr_pure_feynman_test1_report.md` に考察追記、新ブランチ・PR #3 (ドラフト)

### ユーザーが行ったこと
- 「Stage A/C がメイン化し EML を組み合わせる原初の仕組みが意味を失っている。原初の eml-sr を工夫して精度を上げられないか」と問題提起
- 続けて「Stage A/C を完全に削除して EML を組み合わせるだけで精度の高いアルゴリズムを作って欲しい」と指示 → 純 EML モードの計画書を承諾

---

## 2026/07/10 (eml-sr_fable 解説書 manual_eml-sr_fable.md の作成)

### AI（エージェント・Claude Fable）が行ったこと
- `texts/manual_eml-sr_fable.md` を新規作成 (v5 時点の実装に基づく総合マニュアル)
  - 新しい PC 環境でのセットアップ・実行方法 (前提ソフト、clone→maturin ビルド→動作確認、ベンチ実行、トラブルシューティング)
  - Python / Rust 両方からの使い方 (Searcher 全パラメータ表・高速/高精度の推奨設定・節約的候補選択の実例コード・Rust の rlib 利用例)
  - 用途 (法則発見・経験式・飽和応答・スパース変数特定・代理式化)
  - 対応データ (実証済みの範囲と、非対応・前処理が必要なもの)
  - 詳しい仕組み (数式込み): 変換×単項式和の Stage A (WLS・貪欲 log-フィット・OLS 辞書追跡・検証分割)、有理 A2、乗法分解 C、EML ビーム B (アフィンスケーリング・LM)、後処理 (残差ブースティング・原空間ポリッシュ・1SE 則) と過学習防止機構 9 点
  - 強み・弱み: 従来 eml-sr (9/13→82 式・819→26 分の実測比較)、PySR (進化的・柔軟性で勝るが再現性・物理型の厳密回収で fable 優位)、Operon (GP 高速・近似力 vs 厳密回収狙い) — 直接対決ベンチ未実施の旨も明記
  - 精度・実績: Feynman (83/99・82/99・ノイズ付き 71-73/99)、一般ベンチ 19/20、拡張 28 式 27/28、変数特定 10/10、外れ値 20/28、過学習 0 件、発見式の実例
- マニュアル内の Python クイックスタート例を実際に実行して出力を検証 (記載値は実測)。to_python() の変数名置換の注意・recognize_constant の記述を実挙動に合わせて修正
- commit/push、PR #1 のレポート欄にマニュアルを追記

### ユーザーが行ったこと
- 必須 7 項目 (実行方法・Python/Rust 利用・用途・対応データ・数式込みの仕組み・PySR/operon 比較・実績) を指定して解説書の作成を依頼

---

## 2026/07/09 (eml-sr_fable v5: 実データ指向 — ノイズ頑健化・過学習防止・外れ値頑健化)

### AI（エージェント・Claude Fable）が行ったこと
- ユーザー指示 (①誤差なしテスト廃止・すべてノイズ付き評価、②過学習防止機構の明示的強化 (真の目的は精度でなく正しい構造)、③実データに強い追加アルゴリズムの検討) を反映した v5 計画書を作成 → 承諾後に着手
- 既存の過学習防止機構 7 点 (検証分割の項採択・後退剪定、節約的タイブレーク、複雑度ペナルティ、定数スナップ、ノイズ床早期終了、ホールドアウト評価) を整理してユーザーに報告
- **v5 アルゴリズム変更** (cargo test 30件全通過、コミット 1d7e7cb / 15bc8f9 ほか):
  - P1: 変換空間 WLS (w=1/|T'(y)|)、原空間 LM ポリッシュ (リテラル→Param→LM→戻し、変換バイアス・有理 EIV 除去)、連続指数の検証分割採点+丸め優先
  - P2: SigmoidDiff・TanhVar 特徴、AsinSqrt (y=sin²(m))・Atanh (y=tanh(m)) 変換
  - P3: 残差ブースティング (検証誤差 10% 改善で受理)
  - P4: 検証誤差 1標準誤差則ランキング (Stage A + 最終段)、相殺和ガード (Σrms(項)>30·rms(和) を破棄)、トレース比例リッジ
  - P5: Huber-IRLS 外れ値頑健リフィット。見送り項目 (full TLS・欠損値・x依存σ・次元解析) は理由付きで記録
- **v4 wheel でノイズ付きベースラインを先に取得** → v5 と同一条件比較:
  - Feynman 99式+1%ノイズ: **70 → 71 ok (ok+partial 84 → 86)**。+I.41.16, +II.13.34、−III.4.33 (正しい構造のまま閾値 1.64× の際どい判定落ち)
  - 一般ベンチ 20式+ノイズ: **16 → 18 ok (+2)**
  - 12式サブセット+ノイズ: 8 → 8 (式単位で完全一致、回帰ゼロ)
  - test3_v2 (28式+ノイズ): 25 → 25 だが **B2 = z·sigmoid(x−y) を回収 (B 8/9→9/9)**。C10 は正しい構造 (係数 0.1% ずれ) で閾値 2.7% 超過の判定落ち
  - **test4 新設 (1%ノイズ+5%外れ値 10σ): 19 → 20 ok**。B2・C8 復帰 (Huber-IRLS の効果)
  - **スパース性の変数特定 10/10 をノイズ・外れ値下とも維持**
- **過学習検査**: 訓練≪テスト乖離 (10×超) は v4・v5 とも 0 件、test/train 比中央値 0.05 — 既存機構が有効に働いており、v5 ガード群は保険の強化
- **重要な知見**: Lorentz 族の partial 残留の真因は係数の非効率ではなく**識別可能性の限界** (1% ノイズ床では真の構造と 2 次近似が統計的に区別できない)。対策はデータ点数増加側にあることをレポートに明記
- 総合レポート `texts/eml_sr_fable_v5_report.md`、commit/push、PR #1 更新

### ユーザーが行ったこと
- 「さらに精度を上げる余地」「Is One Layer Enough? 論文の応用可否」の 2 問を質問 (後者はニューラルネット不使用のため直接応用不可と回答)
- 質問1 の改善計画書の作成を指示、その後 3 条件 (誤差ありテストのみ・過学習防止・実データ頑健化の検討) を追加して改訂版を承諾

---

## 2026/07/09 (eml-sr_fable v4: 演算子拡張 abs/sigmoid/min/max/連続Pow と test3/test3_v2)

### AI（エージェント・Claude Fable）が行ったこと
- ユーザー提示の演算子拡張提案書に基づき、採否判断付きの作業計画書を作成 → C（スパース性）を 10 式へ拡張する指示を反映した改訂版を承諾後に着手
  - 連続指数 Pow は**既に実装済みであることを確認**（ビームは Pow(x, Param) を許可、LM が指数位置の Param も最適化）→ 新実装せず検証テストのみ追加
- **v4 アルゴリズム変更**（cargo test 19件全通過、コミット 0935ad9）:
  - 演算子 `Abs`・`Sigmoid`（単項）、`Min`・`Max`（二項可換・実部比較）を追加
  - Stage A 特徴に `Sigmoid(xi)`（1変数）と `AbsDiff(i,j)`（ペア |xi−xj|）を追加
  - 正値変数ゼロのとき分数指数の辞書設定をスキップ（|x−y|·z 型の回収退行を修正）
  - 貪欲フィットの指数候補を切片込みで採点 + 定数項の自動挿入 + **連続指数の三分探索リファイン**（±2 範囲 40 反復。Hill 式の指数 −2.5 到達に必要だった）
- **回帰確認（演算子フラグ化の要否判定）**: Feynman 回帰サブセット 12 式で v3 比 delta_ok=0・喪失なし（4.8分）、一般ベンチ **19/20 維持**（0.7分）→ 回帰なし、フラグ化不要と判断し常時有効
- **test3 / test3_v2 を新設**（`eml_sr_fable_test3_common.py` + ランナー2本、28式 = A 新関数直接 9 + B 組合せ改造 9 + C スパース性 10、変数特定指標付き）
- **test3（ノイズなし、1.test と同一条件）: 27/28 回収（A 9/9, B 8/9, C 10/10）、3.9 分**
  - Min/Max はビームが厳密構造を直接発見（`Min(v2, v0·v1)` 複雑度5・RMSE 0）、連続指数 x^1.7 は 4e-15 精度で到達、Hill 式は等価閉形式で回収
  - **スパース性 C は 10/10 回収・変数特定 10/10**（最難 (5,25) の 25 変数も 12.9 秒、無関係変数の混入ゼロ）
  - 唯一の partial は B2 = z·sigmoid(x−y)（Sigmoid 特徴が1変数のみのため。対策候補: SigmoidDiff ペア特徴）
- **test3_v2（+1%ノイズ）: 25/28 回収（A 7/9, B 8/9, C 10/10）、16.2 分**
  - ノイズで喪失したのは A7 (x^1.7)・A8 (Hill) の連続指数 2 式のみ（指数の尤度面がノイズ床で平坦化 — test1_v4 の変換系ノイズ増幅と同型）
  - **離散構造 Min/Max/Abs はノイズに頑健**（構造は厳密なまま係数だけ揺れる）、**スパース性は回収・変数特定とも 10/10 を完全維持**
- レポート2本（A/B/C 別集計・変数特定詳細・考察付き）、daily_report、commit/push、PR #1 更新

### ユーザーが行ったこと
- 演算子拡張提案書を提示し、abs/連続Pow/sigmoid/min/max の追加、必要ならアルゴリズム改善、test3（test1 相当の計算量）と test3_v2（+ガウス誤差）の新設、新関数・組合せ改造・スパース性（(a,b)=(2,5),(3,7),(4,20),(5,15),(6,20) など）のテストを指示
- 作業計画書の C セクションを **10 式**でテストするよう指示 → 改訂版を承諾

---

## 2026/07/08 (2.test / 2.test_v2 結果の考察)

### AI（エージェント・Claude Fable）が行ったこと
- ユーザーが高性能 PC で実行し push した 2.test / 2.test_v2 の結果を git pull で取得し、集計・考察を実施
  - **2.test（BEAM=2000, N=1500, CPLX=8, ノイズなし）: 完全回収 83/99 (83.8%)、部分込み 90/99、48.6 分**
    - 1.test_v3 比 +I.32.17（共鳴・数値近似）、+II.36.38（6変数項）、−III.4.33（partial 降格、閾値超過10%）
    - 残る失敗 4 式（I.30.3, II.35.21, III.8.54, III.9.52）は多変数積の入れ子 trig / tanh 到達性の構造限界
  - **2.test_v2（同+1%ノイズ）: 完全回収 73/99 (73.7%)、部分込み 87/99、187.4 分**
    - ノイズ喪失 13 式の分類: Lorentz 族（1/y² 変換のノイズ増幅、全て partial 残留）、指数対数族（log 空間の等分散性崩れ）、有理式（errors-in-variables）
    - **興味深い現象**: I.37.4（干渉項）等 3 式はノイズ下でのみ完全回収 — 検証分割+ノイズ床早期終了が junk を抑え真の構造が勝ちやすくなる「ノイズの正則化効果」を確認。I.37.4 は真の構造 I1+I2+2√(I1I2)cosδ を係数誤差 0.1% で発見
- 両レポートに考察セクション（§8）を追記、改善候補（変換空間の重み付き最小二乗・有理ステージの TLS 化・3変数積特徴・ノイズ水準の自動推定）を整理
- commit / push、PR #1 の結果表を更新

### ユーザーが行ったこと
- 高性能 PC で 2.test / 2.test_v2 を実行し、結果（JSON・レポート・ログ）を push。考察を依頼

---

## 2026/07/10 (eml-sr_fable v5 ドキュメント更新)

### AI（エージェント）が行ったこと
- `manual_eml-sr_fable.md` (v5) に基づき `eml-sr_fable/README.md` を本格的な日本語 README に更新（特徴・セットアップ・API・ベンチ抜粋）
- `eml-sr_fable/example_python.py` を v5 API（`eml_sr_fable`、`rational_stage`、節約的タイブレーク、`eval`/`eval_batch`）に合わせて全面改修し動作確認

### ユーザーが行ったこと
- v5 進化に合わせた README / example_python の編集を依頼

---

## 2026/07/07 (test2 別PC実行手順書)

### AI（エージェント）が行ったこと
- 別 PC で `feynman_eml_sr_fable_test2.py` / `test2_v2.py` を実行する手順書を作成: `texts/20260707_手順書.md`（git clone・ブランチ切替・maturin ビルド・スモーク/本番実行・出力ファイルの説明・トラブルシューティング）
- 手順書に追記: 計算後のターミナル出力を `results/feynman_fable_test2.log` / `test2_v2.log` へコピペする手順、git add / commit / push

### ユーザーが行ったこと
- 別 PC 実行手順の整理・手順書作成を依頼
- 手順書へ .log コピペと git push 手順の追記を指示

---

## 2026/07/05 (eml-sr_fable v3: 汎用化・ノイズ頑健化)

### AI（エージェント・Claude Fable）が行ったこと
- ユーザーが push した `eml-sr_fable/pointout_cursor.md`（Feynman 特化・誤差未テストの指摘）を精査し、採否判断付きの作業計画書を作成 → 承諾後に着手
  - 採用: 負値変数対応・有理関数ステージ・Logit 変換・検証分割・ホールドアウト評価・ノイズベンチ・非 Feynman ベンチ
  - 不採用(理由付き): Stage A/C の格下げ、Box-Cox 変換、暗黙関数ステージ、辞書のデータ駆動生成 等
- **v3 アルゴリズム変更**（eml-sr_fable 直接編集、cargo test 14件全通過）:
  - 変数集合の分離（整数べき・trig・差分特徴は負値/符号混在変数でも使用可）
  - 有理関数ステージ（y·Q=P を y を拡張入力列とする辞書追跡で線形化、分母項の強制初手を小基底では全列試行）
  - Logit 変換 t=ln((1−y)/y)（ロジスティック族）
  - 検証分割（80/20 決定的）による項採択・剪定、候補選択の節約的タイブレーク
  - 整数辞書を ±2（素特徴付き）と ±3 の 2 本立てに（±3 単独追加で I.40.1 の選択退行が出たため修正）
- **テスト体系を4本+一般ベンチに拡張**: test1_v3（ホールドアウト250点）/ test1_v4（+1%ノイズ）/ test2 更新（v3+ホールドアウト500点）/ test2_v2（+1%ノイズ、ユーザーPC用）/ 非Feynman合成20式ベンチ
- **ベンチマーク実行結果**:
  - 非 Feynman 一般ベンチ: **19/20 回収**（0.6分。有理式4/4・ロジスティック3/3・負値域・符号混在を含む。唯一の失敗 sin(x+x²) は complexity 6 の構造限界）
  - **1.test_v3: 完全回収 82/99 (82.8%)**、ホールドアウトのテスト RMSE 基準。訓練=評価だった v2 (80式) を上回り**過適合していないことを確認**。+I.16.6/+I.18.4（有理式）、v1 からの回帰ゼロ、26.0分
  - **1.test_v4 (1%ノイズ): 完全回収 70/99 (70.7%)**、部分回収込み 84/99。喪失の大半は Lorentz 因子型の partial 降格（1/y² 変換によるノイズ増幅、原因特定済み）。65.5分
- レポート3本（v3/v4/一般ベンチ、指摘への採否と分析付き）、commit/push、PR #1 更新

### ユーザーが行ったこと
- pointout_cursor.md を push し、指摘の批判的検討とアルゴリズム修正、テスト4本の作成、1.test_v3/v4 の実行を指示
- v3 作業計画書（Feynman特化への対処の対応表を含む改訂版）を承諾

---

## 2026/07/05 (eml-sr_fable v2 改善)

### AI（エージェント・Claude Fable）が行ったこと
- 1.test 失敗 21 式の原因分析を実施（ユーザー指摘の「Plus 演算子だらけの失敗式」= 辞書にない構造を junk な単項式和で近似していたことを特定）。作業計画書 v2 を作成しユーザー承諾後に着手
- **eml-sr_fable 本体を直接改善（v2）**:
  - 特徴量拡張辞書（単項式×{sin, cos, sin(2x), cos(2x), ln, (xi−xj)², cos(xi−xj), cos/sin(xi·xj), cos/sin(2xy)}、±0.5 指数込み）を第2パスとして追加
  - 整数辞書の active 変数上限 3→5、素の1変数特徴量列を整数辞書に同乗
  - Log 変換の過剰ガード除去（指数則で y が数十桁にわたるケース、I.40.1 の直接原因）
  - 後方剪定の強化（削除許容 1.25 倍 + 未剪定フォールバック）→ junk な Plus 連鎖を抑制
  - Log1p 変換（e^m−1 型）、比ターゲットへの Stage A 適用、混合符号ホワイトナー + 候補5プローブ
  - 辞書 ≤2万列時の多スタート OLS（初期相関上位4列を強制初手）
  - Tanh 演算子登録（2.test 向け）、単体テスト4件追加（計9件全通過）
- `src/feynman_eml_sr_fable_test1_v2.py` を新規作成（1.test と同一条件・同一シード、出力は v1 と分離、fable_v1 ベースライン比較付き）
- **全 99 式 1.test_v2 本番実行完了**:
  - **結果: 完全回収 80式 (80.8%)、部分回収 6式、合計 86式 (86.9%)、失敗 8式、スキップ 5式**
  - **v1 比 delta_ok = +17（新規: I.8.14, I.12.11, I.13.12, I.29.16, I.34.14, I.40.1, I.41.16, I.44.4, I.50.26, I.6.2b, II.2.42, II.21.32, II.6.15a/b, III.14.14, III.15.12, III.17.37）、回帰ゼロ**
  - cursor 比 +67、first_AI 比 +71。総実行時間 27.1 分、1 式中央値 2.9 秒
  - 残る失敗 8 式は有理式・Dirichlet 核・sinc²・6変数項など complexity 6 での構造的困難
  - 出力: `results/eml_sr_fable_feynman_test1_v2_results.json`, `texts/eml_sr_fable_feynman_test1_v2_report.md`, `results/feynman_fable_test1_v2.log`
- 成果物を git commit → push、PR #1 を更新

### ユーザーが行ったこと
- 1.test 結果を確認し、失敗式（特に Plus 演算子が多い出力）の原因分析と修正・1.test_v2 の実行を指示。eml-sr_fable 本体の直接編集を許可
- v2 作業計画書を承諾

---

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


