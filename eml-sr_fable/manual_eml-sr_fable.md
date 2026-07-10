# eml-sr_fable 解説書 (マニュアル)

**対象バージョン:** v5 (2026-07-09 時点、ブランチ `claude/eml-sr-fable-algorithm-7w8j49`)
**エンジン:** Rust クレート `eml_sr_fable` 0.2.0 (Python バインディング付き、MIT ライセンス)

---

## 目次

1. [eml-sr_fable とは](#1-eml-sr_fable-とは)
2. [新しい PC 環境でのセットアップと実行方法](#2-新しい-pc-環境でのセットアップと実行方法)
3. [Python / Rust からの使い方](#3-python--rust-からの使い方)
4. [何に使えるか](#4-何に使えるか)
5. [対応しているデータ](#5-対応しているデータ)
6. [詳しい仕組み (数式込み)](#6-詳しい仕組み-数式込み)
7. [強み・弱み — 従来 eml-sr / PySR / Operon との比較](#7-強み弱み--従来-eml-sr--pysr--operon-との比較)
8. [精度と実績](#8-精度と実績)
9. [参考: リポジトリ内の関連ファイル](#9-参考-リポジトリ内の関連ファイル)

---

## 1. eml-sr_fable とは

**eml-sr_fable** は、数値データ $(X, y)$ から人間が読める**閉形式の数式** $y \approx f(x_1, \dots, x_d)$ を自動発見する**シンボリック回帰 (symbolic regression, SR)** エンジンである。

従来モデル `eml-sr_model_cursor` をフォークして開発され、EML 演算子

$$\mathrm{EML}(x, y) = \exp(x) - \ln(y)$$

を含む演算子体系の枠組みを維持したまま、探索アルゴリズムを全面的に刷新した。ニューラルネットワークは使わず、**閉形式ソルバー (線形代数) + ビームサーチ + Levenberg-Marquardt 定数最適化**による決定論的なパイプラインで動く。同じデータと設定に対しては常に同じ結果を返す。

### 開発の経緯 (v1 → v5)

| 版 | 主な内容 | Feynman 99式 (同一高速条件) |
|----|---------|------------------------------|
| v1 | 3段パイプライン (閉形式 Stage A + 乗法分解 Stage C + ビーム Stage B) の新設 | 63/99 (19分) |
| v2 | 失敗分析に基づく改善 (特徴量拡張辞書・バックフィッティング・剪定) | 80/99 (27分) |
| v3 | 汎用化・ノイズ頑健化 (負値変数・有理ステージ・Logit 変換・検証分割・ホールドアウト評価) | 82/99 (26分) |
| v4 | 演算子拡張 (Abs / Sigmoid / Min / Max、連続指数の検証) + スパース性テスト | 82 相当を維持 + 拡張28式 27/28 |
| v5 | 実データ指向 (変換空間 WLS・原空間 LM ポリッシュ・過学習ガード・Huber-IRLS 外れ値対策) | ノイズ付きで 71/99 (v4 比 +1)、一般ベンチ +2 |

参考: フォーク元の `eml-sr_model_cursor` は同一データで **13/99 (819分)** だった。

---

## 2. 新しい PC 環境でのセットアップと実行方法

GPU は不要 (CPU のみ)。Linux / macOS / Windows で動作する。

### 2.1 前提ソフトウェア

| ソフトウェア | バージョン目安 | 入手 |
|-------------|--------------|------|
| Git | 任意 | — |
| Python | 3.8 以上 | — |
| Rust (cargo) | 最新安定版 | https://rustup.rs (`curl ... | sh` 一発) |
| maturin | 1.x | `pip install maturin` |

**Windows のみ:** Visual Studio Build Tools の「C++ によるデスクトップ開発」ワークロードが必要 (Rust のリンクに使う)。

### 2.2 取得 → ビルド → 動作確認

```bash
# 1. リポジトリ取得とブランチ切替 (main には未マージ)
git clone https://github.com/blabo25226/eml-sr_model.git
cd eml-sr_model
git fetch origin
git checkout claude/eml-sr-fable-algorithm-7w8j49

# 2. Python 仮想環境
python3 -m venv .venv
source .venv/bin/activate        # Windows: .\.venv\Scripts\Activate.ps1
pip install --upgrade pip
pip install maturin numpy pandas

# 3. Rust エンジンをビルドして Python にインストール
cd eml-sr_fable
maturin develop --release --features python,full-math
cd ..

# 4. 動作確認
python -c "import eml_sr_fable; print(eml_sr_fable.__file__)"
```

パスが表示されれば成功。wheel ファイルとして配布したい場合は `maturin develop` の代わりに:

```bash
maturin build --release --features python,full-math -o dist/
pip install dist/eml_sr_fable-*.whl
```

(任意) Rust 単体テスト 30 件の実行:

```bash
cd eml-sr_fable && cargo test --release --features full-math
```

### 2.3 ベンチマークスクリプトの実行

**必ず `src/` ディレクトリから実行する** (共通モジュール `feynman_fable_common.py` が同ディレクトリにあるため)。

```bash
cd src

# Feynman 99式 (高速条件・ホールドアウト評価)
python feynman_eml_sr_fable_test1_v3.py --smoke   # まず3式で動作確認
python feynman_eml_sr_fable_test1_v3.py           # 本番 (約30分)

# Feynman 99式 + 1% ガウスノイズ (v5 評価)
python feynman_eml_sr_fable_test1_v5.py           # 約80分

# 緩和条件 (BEAM=2000, N=1500, CPLX=8) — 高性能 PC 向け、十数時間
python feynman_eml_sr_fable_test2.py

# 非 Feynman 一般ベンチ 20式 / 演算子拡張 28式 / 外れ値ベンチ
python general_eml_sr_fable_test_v3.py
python eml_sr_fable_test3.py
python eml_sr_fable_test4.py
```

結果は `results/*.json` (機械可読、1式ごとに逐次保存) と `texts/*_report.md` (レポート) に出力される。長時間実行は `nohup python ... > ../results/xxx.log 2>&1 &` を推奨。

### 2.4 トラブルシューティング

| 症状 | 対処 |
|------|------|
| `ModuleNotFoundError: eml_sr_fable` | venv を activate した状態で `maturin develop --release --features python,full-math` を再実行 |
| `ModuleNotFoundError: feynman_fable_common` | `src/` から実行しているか確認 |
| Rust ビルド失敗 (Windows) | VS Build Tools をインストールしターミナル再起動 |
| `FeynmanEquations.csv` がない | ブランチ確認: `git checkout claude/eml-sr-fable-algorithm-7w8j49` |
| 1式が時間上限で打ち切られる | `time_budget_s` の設計どおり (正常動作) |

より詳しい別 PC 実行手順は `texts/20260707_手順書.md` を参照。

---

## 3. Python / Rust からの使い方

### 3.1 Python — クイックスタート

```python
import numpy as np
import eml_sr_fable

# データ生成 (例: y = x0^2 + 3*x1、真の式は未知という想定)
rng = np.random.default_rng(0)
X = rng.uniform(0.5, 3.0, size=(500, 2))     # 500 サンプル × 2 変数
y = X[:, 0]**2 + 3.0 * X[:, 1]

searcher = eml_sr_fable.Searcher(
    max_complexity=6,
    beam_width=1000,
    time_budget_s=60.0,
    verbose=False,
)

# 候補の Pareto front (誤差 × 複雑度) を取得
candidates = searcher.find_candidates(X.tolist(), y.tolist())
best = candidates[0]
print(best.formula)       # Plus(Times(2.9999999999999996, v_{1}), Times(1.0000000000000002, Square(v_{0})))
print(best.to_python())   # ((2.99...)*(v_{1}))+((1.00...)*(((v_{0})**2)))
print(best.complexity)    # 8
preds = best.predict(X.tolist())   # 予測値のリスト (式の評価はこちらを使う)
```

入力は **Python の 2 重リストまたは 2 次元 NumPy 配列** (行 = サンプル、列 = 変数)、ターゲットは 1 次元。`v_{i}` は第 i 列 (0 始まり) の変数を指す。`to_python()` の文字列を自分のコードに移す場合は `v_{0}` 等を自分の変数名 (例: `X[:,0]`) に置換して使う。数値評価そのものは置換不要の `predict()` / `eval_batch()` が確実である。

### 3.2 `Searcher` のパラメータ一覧

| パラメータ | 既定値 | 意味 |
|-----------|--------|------|
| `max_complexity` | 7 | ビームサーチが組み立てる式の最大ノード数。大きいほど複雑な式に届くが指数的に遅くなる |
| `complexity_penalty` | 0.08 | ビーム採点の複雑度ペナルティ (誤差 + penalty×複雑度) |
| `beam_width` | 1200 | ビーム幅 (各複雑度レベルで保持する候補数) |
| `time_budget_s` | 0 (無制限) | 1 回の探索の時間予算 (秒)。**実用では必ず設定を推奨** |
| `subsample_size` | 256 | ビーム採点・LM に使う決定論的サブサンプル数 |
| `early_exit_threshold` | 1e-9 | 訓練誤差が `threshold×std(y)` を下回ったら探索を打ち切る。**ノイズがあるデータでは 9e-3 程度に上げる** (ノイズ床以下を追わない) |
| `refinement_top_k` | 8 | 末尾 LM リファインを掛ける候補数 |
| `snap_constants` | True | 悪化しない場合のみ係数を整数・π 等へ丸める |
| `powerlaw_stage` | True | Stage A (閉形式ソルバー) の有効化 |
| `ratio_search` | True | Stage C (乗法分解) の有効化 |
| `rational_stage` | True | Stage A2 (有理関数) の有効化 |
| `affine_scaling` | True | ビーム候補のアフィンスケーリング採点 |
| `max_boost_terms` | 6 | Stage A の項数上限 |
| `verbose` | True | 進捗ログ |

**推奨設定 2 例:**

```python
# 高速 (Feynman 1.test 相当: 1式あたり最大110秒)
fast = dict(max_complexity=6, complexity_penalty=0.08, beam_width=1000,
            time_budget_s=110.0, subsample_size=256, early_exit_threshold=1e-9,
            refinement_top_k=8, snap_constants=True, powerlaw_stage=True,
            ratio_search=True, rational_stage=True, affine_scaling=True,
            max_boost_terms=6, verbose=False)

# 高精度 (2.test 相当: 1式あたり最大10分)
thorough = dict(fast, max_complexity=8, beam_width=2000,
                time_budget_s=600.0, max_boost_terms=8)

# ノイズがあるデータではどちらの場合も early_exit_threshold=9e-3 を推奨
```

### 3.3 候補からの選択 (節約的タイブレーク)

`find_candidates` は誤差×複雑度の Pareto front を返す。ノイズがあるデータでは「最良誤差の候補」より「誤差がほぼ同じで最も単純な候補」を選ぶのが安全である (プロジェクトのベンチはすべてこの方式):

```python
scored = []
for c in candidates:
    p = np.asarray(c.predict(X.tolist()), dtype=float)
    rmse = float(np.sqrt(np.mean((y - p)**2))) if np.all(np.isfinite(p)) else np.inf
    scored.append((c, rmse))
best_rmse = min(r for _, r in scored)
band = [(c, r) for c, r in scored if r <= best_rmse * 1.05]   # 5% 帯
best, _ = min(band, key=lambda cr: (cr[0].complexity, cr[1]))  # 帯内で最小複雑度
```

### 3.4 その他の Python API

```python
r = searcher.find_function(xs, ys)        # 1変数版 (最良1件を返す)
r = searcher.find_multivariate(X, y)      # 多変数版 (最良1件)
r = searcher.fit(X, y)                    # find_multivariate の別名 (sklearn 風)
r = searcher.recognize_constant(3.14159)  # スカラー定数の閉形式同定
r.formula      # 正準表記 (プレフィックス記法)
r.to_python()  # NumPy 式文字列
r.to_latex()   # LaTeX 文字列
r.error        # 探索時誤差
r.complexity   # ノード数
r.eval(x) / r.eval_batch(X) / r.predict(X)
```

例外: 入力不正は `EmlDimensionError`、候補ゼロは `EmlComplexityError`。

### 3.5 Rust からの使い方

`eml_sr_fable` は `cdylib` と同時に `rlib` としてもビルドされるので、Rust プロジェクトから直接使える。

```toml
# Cargo.toml
[dependencies]
eml_sr_fable = { path = "../eml-sr_model/eml-sr_fable", features = ["full-math"] }
```

```rust
use eml_sr_fable::{Searcher, SearchConfig};

fn main() {
    let mut config = SearchConfig::fable_default();
    config.max_complexity = 6;
    config.time_budget_s = 60.0;
    config.verbose = false;

    // 行 = サンプル、列 = 変数
    let inputs: Vec<Vec<f64>> = (0..500)
        .map(|i| {
            let x0 = 0.5 + 2.5 * ((i * 7919 % 1000) as f64 / 1000.0);
            let x1 = 0.5 + 2.5 * ((i * 104729 % 1000) as f64 / 1000.0);
            vec![x0, x1]
        })
        .collect();
    let ys: Vec<f64> = inputs.iter().map(|r| r[0] * r[0] + 3.0 * r[1]).collect();

    let searcher = Searcher::new(config);
    let results = searcher.find_candidates(&inputs, &ys).unwrap();
    for r in &results {
        println!("{}  (error={:.3e}, complexity={})", r.formula(), r.error(), r.complexity());
    }
}
```

CLI バイナリ (`cargo run --bin eml-sr-fable`) も同梱されている。Rust 側 API は Python と同じ `find_function` / `find_multivariate` / `find_candidates` / `recognize_constant` を持つ。

---

## 4. 何に使えるか

1. **実験・観測データからの法則発見** — 物理・化学・生物の測定データから支配方程式そのものを回収する。Feynman 物理方程式 99 本のベンチで実証済み (§8)
2. **工学の経験式・整定式の自動導出** — 実測データに合う可読な近似式 (べき乗則・有理式・飽和則) を作る。回帰係数だけでなく**式の形**が出るので、外挿性・単位の見通しがよい
3. **飽和・応答曲線のモデリング** — Hill 式 $E x^n/(x^n + K^n)$、ロジスティック $1/(1+e^{-m})$、tanh 飽和、指数減衰など、専用の変換 (§6) で閉形式回収できる
4. **スパースな多変数データからの変数特定** — 例: 25 変数のうち実際に効いている 5 変数だけを使う式を、無関係変数の混入なしで発見 (変数特定 10/10 実証)。特徴量スクリーニングとしても使える
5. **ブラックボックスモデルの代理式化** — NN や勾配ブースティングの予測を可読な数式で近似し、解釈・監査・軽量デプロイに使う
6. **定数の閉形式同定** — `recognize_constant(value)` によるスカラー定数の記号的同定 (補助機能)

---

## 5. 対応しているデータ

### 対応している (実証済み)

| 項目 | 内容 |
|------|------|
| 変数の数 | 1〜25 変数で実証 (Feynman は最大 9、スパース性テストで 25) |
| サンプル数 | 数百〜数千。ベンチは 750〜1500 点。目安として変数数の 30 倍以上を推奨 |
| 値域 | 正値・負値・符号混在いずれも可 (v3 で対応)。値のスケールは 10^-50〜10^+50 のような広レンジも可 (I.40.1 で実証) |
| ノイズ | ガウシアンノイズ (1% 水準で系統的に実証)。変換空間 WLS・検証分割・ノイズ床早期終了で対策済み。ノイズがある場合は `early_exit_threshold=9e-3` 程度に設定する |
| 外れ値 | 混入外れ値 (5%・10σ で実証)。Huber-IRLS リフィットが自動で作動 |
| 無関係変数 | 使われない変数が大量に混ざっていてよい (25 変数中 5 変数のみ使用、で実証) |

### 対応していない / 前処理が必要

| 項目 | 対処 |
|------|------|
| 欠損値 (NaN) | 事前に行を削除するか補完する (エンジンは有限値を要求) |
| カテゴリ変数 | ダミー変数化などの数値化が必要 (式の形はカテゴリを想定していない) |
| 複数ターゲット | 1 ターゲットずつ実行する |
| 時系列の逐次構造 | ラグ特徴を列として与えれば可能だが、微分方程式・再帰構造の直接発見は非対応 |
| 陰関数 $g(x, y) = 0$ | 非対応 (陽形式 $y = f(x)$ のみ) |
| 単位・次元情報 | 使わない (将来課題として記録) |

---

## 6. 詳しい仕組み (数式込み)

eml-sr_fable は **4 つの探索ステージ + 共通後処理**からなる。安い順に走り、どこかで解ければ以降を省略する。全体を通した設計原理は「**物理・工学の式の大半は、適切な変換の下で少数の単項式の和になる**」という観察である。

### 6.0 記法

データ $\{(\mathbf{x}^{(i)}, y_i)\}_{i=1}^n$、$\mathbf{x} \in \mathbb{R}^d$。単項式を

$$m_k(\mathbf{x}) = \prod_{j=1}^{d} x_j^{a_{kj}}$$

と書く ($a_{kj}$ は整数・半整数、または連続値)。

### 6.1 Stage A — 閉形式ソルバー (変換 × 単項式和)

ターゲットに可逆変換 $T$ を適用し、変換後が単項式和になる構造を**線形代数だけで**解く:

$$T(y) \approx t(\mathbf{x}) = c_0 + \sum_{k} c_k \, m_k(\mathbf{x}) \cdot \phi_k(\mathbf{x})$$

$\phi_k$ は任意の特徴因子 (無しも可)。使用する変換は

$$T \in \left\{\, y,\ \ln y,\ \tfrac{1}{y},\ \tfrac{1}{y^2},\ y^2,\ \ln(1+y),\ \ln\tfrac{1-y}{y},\ \arcsin\sqrt{y},\ \operatorname{atanh} y \,\right\}$$

それぞれ指数則 ($\ln$)、Lorentz 型 ($1/y, 1/y^2$)、ロジスティック族 (logit)、遷移確率 $y=\sin^2 m$ ($\arcsin\sqrt{y}$)、飽和則 $y=\tanh m$ (atanh) などを線形化する。

**(a) 重み付き最小二乗 (WLS)** — $y$ のノイズ $\sigma$ は変換後に $\sigma\,|T'(y_i)|$ に増幅されるため、行重み

$$w_i = \frac{1}{|T'(y_i)|} \qquad (\text{例: } T=\ln y \Rightarrow w_i = |y_i|,\quad T = 1/y^2 \Rightarrow w_i = |y_i|^3/2)$$

で等分散性を回復する (設計行列・ターゲットを $\sqrt{w_i}$ でスケーリング)。

**(b) 貪欲 log-空間ブースティング** — 残差 $r$ に対し $\ln|r| \approx \ln|c| + \sum_j a_j \ln x_j$ の線形回帰で指数ベクトルを推定 → 有理数 (分母 1..4) に丸めた候補を切片込みで採点 → 残差を更新して繰り返す。項が 1 変数のみのときは指数を**三分探索**で連続値へリファインする。このとき 80/20 検証分割で採点し、**検証誤差が 2% 以上改善しない限り丸め指数を優先**する (ノイズで指数がドリフトするのを防ぐ)。

**(c) OLS 辞書追跡 (orthogonal least squares pursuit)** — 標準指数集合 (分数 $\{\pm 3, \pm 2, \pm 1, \pm 0.5\}$ / 整数 $\{\pm 3..\}, \{\pm 2..\}$) から生成した単項式辞書、および**特徴量拡張辞書**

$$\phi \in \{\sin x_i,\ \cos x_i,\ \sin 2x_i,\ \cos 2x_i,\ \ln x_i,\ \sigma(x_i),\ \tanh x_i,\ (x_i - x_j)^2,\ |x_i - x_j|,\ \sigma(x_i - x_j),\ \cos(x_i \pm x_j),\ \sin/\cos(x_i x_j), \dots\}$$

(σ はシグモイド) に対して、既選択列に直交化した相関で 1 列ずつ選ぶ。単純な OMP は共線辞書で「妥協列」(例: $x^2 + 3y$ に対する $x\sqrt{y}$) に捕まるため、以下で防ぐ:

- **バックフィッティング**: 選択済みの各列を、他列の残差を最も説明する列と巡回交換 (6 パス)
- **多スタート**: 相関上位 4 列 + 定数シードを強制初手にした複数回の追跡
- **検証ゲート**: 項の追加は 80/20 分割の検証誤差が改善する場合のみ受理
- **後退剪定**: 完成後、各項を外して検証誤差が悪化しない項を削除
- **Huber-IRLS**: 剪定後の係数を Huber 重み ($c = 1.345 \cdot \mathrm{MAD}$) の反復再重み付け最小二乗で頑健化し、検証誤差が改善する場合のみ採用 (外れ値対策)
- **相殺和ガード**: $\sum_k \mathrm{rms}(c_k m_k) > 30 \cdot \mathrm{rms}(\sum_k c_k m_k)$ となる候補 (巨大係数の相殺 = 過学習 junk) を破棄

### 6.2 Stage A2 — 有理関数ステージ

$$y \approx \frac{P(\mathbf{x})}{Q(\mathbf{x})}$$

をピボット項 $q_0$ (係数 1 に正規化) を選んで **$y\,Q = P$ と線形化**する:

$$y \cdot q_0(\mathbf{x}) = \sum_j p_j\, b_j(\mathbf{x}) - \sum_k q_k\, \bigl(y \cdot b_k(\mathbf{x})\bigr)$$

$y$ を拡張入力列として扱えば通常の辞書追跡に帰着する。分母列は初期相関で不利なため、小さい基底では**全分母列を強制初手**として試す。

### 6.3 Stage C — 乗法分解 (比の探索)

$$y = m(\mathbf{x}) \cdot g(\mathbf{x})$$

と分解し、log-空間フィットから単項式ホワイトナー $m$ の候補を 5 通り生成 ($|y|$ ベース・分母無視・自信あり丸め・単位指数のみ 等)、比 $y/m$ に対して Stage A → ビームサーチを掛ける。$q \sin^2\theta / \varepsilon$ のような「単項式 × 非単項式」の積構造を回収する。

### 6.4 Stage B — EML ビームサーチ

演算子集合 (四則、EML、exp/log/sqrt/square/cube/pow、三角・逆三角、tanh、**abs・sigmoid・min・max**、調整可能定数 Param) の上で、複雑度 $k = 1, 2, \dots$ の式を幅 `beam_width` のビームで構成的に列挙する。採点は**アフィンスケーリング**:

$$\mathrm{score}(f) = \min_{a, b} \ \mathrm{RMSE}(y,\ a f(\mathbf{x}) + b) + \lambda \cdot \mathrm{complexity}(f)$$

これにより定数倍・オフセットをビームが探す必要がなくなる。連続指数は $\mathrm{Pow}(x, C)$ ($C$=Param) が候補に含まれ、**Levenberg-Marquardt** が式中の全 Param (指数位置も) を最適化する:

$$(J^\top J + \mu I)\,\delta = J^\top r$$

上位 `refinement_top_k` 件は全データで多スタート LM + 定数スナップ (悪化しない場合のみ整数・$\pi$ 倍数へ丸め) を受ける。重複除去は数値フィンガープリント、多様性はアフィン同値クラスの上限で確保する。

### 6.5 共通後処理 (パイプライン最終段)

1. **残差ブースティング** — 最良候補 $f_1$ の誤差が「惜しい」帯域 ($10^{-4}\,\mathrm{std} < \mathrm{err} < 0.2\,\mathrm{std}$) なら、残差 $y - f_1$ に Stage A を再実行し $f_1 + g$ を候補化。**検証誤差 10% 以上改善**する場合のみ受理 (ノイズ追随の防止)
2. **原空間 LM ポリッシュ** — 上位 8 候補の数値リテラルを Param 化し、**生の $y$** に対して LM 再最適化。変換空間 OLS のバイアス ($\ln$ 空間の最適 ≠ 原空間の最適) と有理線形化の errors-in-variables バイアスを除去
3. **検証誤差 1標準誤差則ランキング** — 全候補を held-out 20% の誤差でランキングし、最良から 1SE ($\approx 1/\sqrt{2 n_{\mathrm{val}}}$、5〜6%) 帯内では**最小複雑度を優先**。訓練誤差だけで勝つ過学習候補はここで落ちる
4. **Pareto front** — 複雑度ごとの最良を、誤差が単調改善する列として返す

### 6.6 過学習防止機構のまとめ

(1) 検証分割による項採択、(2) 検証分割による後退剪定、(3) 1SE 則ランキング、(4) 相殺和ガード、(5) 複雑度ペナルティ、(6) 定数スナップ、(7) ノイズ床早期終了、(8) 節約的タイブレーク (利用側)、(9) ホールドアウト評価 (評価側)。実測では訓練≪テスト乖離 (10 倍超) は **0 件** (§8)。

---

## 7. 強み・弱み — 従来 eml-sr / PySR / Operon との比較

### 7.1 従来 eml-sr (first_AI / cursor) との比較 — 実測

| モデル | 手法 | Feynman 99式 (同一条件) | 時間 |
|--------|------|--------------------------|------|
| eml-sr_model_first_AI | 純粋ビームサーチ | 9/99 | 176分 |
| eml-sr_model_cursor | ビーム強化 (Square/Cube/Pow・LM) | 13/99 | 819分 |
| **eml-sr_fable (v3)** | **閉形式 4 段 + ビーム** | **82/99** | **26分** |

差の本質: cursor まではすべてを**ビームサーチだけ**で解こうとした。式空間は複雑度に対して指数的なので、複雑な式には届かずに時間を使い切る。fable は「物理式の大半は変換すれば単項式和」という事前知識を**閉形式ステージ (線形代数、ミリ秒〜秒)** として前置し、ビームは閉形式で表せない構造 (深い合成・EML 的な非分離形) にだけ使う。この分業が精度 6 倍・速度 30 倍の源泉である。

### 7.2 PySR との比較

[PySR](https://github.com/MilesCranmer/PySR) は Julia 製 SymbolicRegression.jl をバックエンドとする、現在最も広く使われる SR ライブラリ。**遺伝的プログラミング (進化的探索) + 焼きなまし + 定数最適化**の確率的手法。

| 観点 | eml-sr_fable | PySR |
|------|--------------|------|
| 探索方式 | 決定論的 (閉形式 + ビーム) | 確率的 (進化的 + 焼きなまし) |
| 再現性 | **同一入力 → 常に同一出力** | 実行ごとに結果が揺れる (シード固定でも並列で非決定的になり得る) |
| 物理型の式 (べき・有理・飽和) | **閉形式ステージが秒で厳密回収** | 世代を重ねて到達 (時間がかかる、係数は近似) |
| ノイズ・過学習対策 | WLS・検証分割・1SE 則・Huber を**機構として内蔵** | model_selection・parsimony で調整 (統計的検証は利用者側) |
| 演算子の拡張性 | Rust 側にコード追加が必要 | **Julia の任意関数を演算子にできる (強い)** |
| カスタム損失・制約 | RMSE 固定 | **任意損失・次元制約・演算子の入れ子制約が可能** |
| スケール | CPU 単機 (rayon 並列) | **マルチコア・クラスタ・(一部) GPU** |
| エコシステム | 本リポジトリのみ | 論文・コミュニティ・scikit-learn 互換 |

**要約:** 「変換すれば線形になる」クラスの式 (物理・工学の大半) では fable が速く・正確で・再現可能。式の形が完全に自由・独自演算子や独自損失が要る場合は PySR が柔軟。

### 7.3 Operon との比較

[Operon](https://github.com/heal-research/operon) は C++ 製の高速 GP フレームワーク (NSGA-II による誤差×複雑度の多目的最適化、SRBench 上位常連)。

| 観点 | eml-sr_fable | Operon |
|------|--------------|--------|
| 探索方式 | 決定論的パイプライン | 遺伝的プログラミング (木構造進化) |
| 速度の性格 | 閉形式で解ける式は**ミリ秒〜秒**、ビーム部は複雑度上限に強く依存 | 評価スループット自体が非常に速い (ベクトル化・低レベル最適化) |
| 厳密回収 vs 近似 | **構造の厳密一致を狙う** (RMSE < 1e-4 判定で 82/99) | 汎化誤差の最小化が主眼 (厳密一致は保証しない) |
| Pareto front | 複雑度ごとの最良列を返す | NSGA-II のフロント (同様) |
| 定数最適化 | LM (解析的でない Param 全対応) | LM / 非線形最小二乗を内蔵 |

**要約:** 「予測精度のよい式を進化させる」のが Operon、「生成過程の式そのものを当てにいく」のが fable。真の式が存在するデータ (科学データ) では fable の設計が有利に働き、真の式が存在しない・複雑すぎるデータでは GP 系の近似力が勝る。

*注: PySR / Operon との比較は設計上の対比であり、同一データでの直接対決ベンチは本プロジェクトでは実施していない。定量比較は将来課題である。*

### 7.4 eml-sr_fable の弱み (自己評価)

1. **複雑度の上限** — ビームは実用上 CPLX 6〜8 まで。深い入れ子 (Dirichlet 核 $\sin^2(n\theta/2)/\sin^2(\theta/2)$、3変数積の入れ子 trig) は構造クラスとして届かない (2.test の残存失敗 4 式)
2. **ノイズ床での識別可能性** — 1% ノイズでは真の構造とその 2 次近似が統計的に区別できないケースがある (Lorentz 族)。これはアルゴリズムでなく情報量の限界で、データ点数を増やす側の問題
3. **固定された構造クラス** — 変換・特徴のリストは手設計。リストにない構造 (例: パラメータが sin の内部に入る形) は Stage A で拾えない
4. **拡張の敷居** — 演算子・特徴の追加は Rust コードの変更とリビルドが必要
5. **単一ターゲット・実数値・RMSE のみ** — 多目的・カスタム損失・分類は非対応
6. **エコシステア不在** — ドキュメント・コミュニティ・パッケージ配布は本リポジトリの範囲のみ

---

## 8. 精度と実績

すべて本リポジトリ内に生データ (`results/*.json`) とレポート (`texts/*_report.md`) がある。判定は原則ホールドアウト (訓練と別シードのテスト点) に対する RMSE で、`ok` は RMSE < 1e-4 (ノイズ付きは < max(1e-4, 0.15σ))。

### 8.1 Feynman 物理方程式 99 式

| テスト | 条件 | 完全回収 | 部分込み | 時間 |
|--------|------|---------|---------|------|
| 2.test | BEAM=2000, N=1500, CPLX=8, クリーン | **83/99 (83.8%)** | 90/99 | 48.6分 |
| 1.test_v3 | BEAM=1000, N=750, CPLX=6, クリーン | **82/99 (82.8%)** | 87/99 | 26.0分 |
| 2.test_v2 | 緩和条件 + 1% ノイズ | **73/99 (73.7%)** | 87/99 | 187分 |
| 1.test_v5 | 高速条件 + 1% ノイズ (v5) | **71/99 (71.7%)** | 86/99 | ~80分 |
| (参考) cursor | 高速条件クリーン, 訓練=評価 | 13/99 | 16/99 | 819分 |
| (参考) first_AI | 同上 | 9/99 | 10/99 | 176分 |

### 8.2 非 Feynman 一般ベンチ (合成 20 式: 負値域・符号混在・有理・ロジスティック)

- クリーン: **19/20** (唯一の失敗は $\sin(x + x^2)$ の複雑度限界)
- 1% ノイズ: **18/20** (v4 では 16/20 → v5 で +2)

### 8.3 演算子拡張ベンチ (28 式 = 新関数 9 + 組合せ 9 + スパース性 10)

- クリーン (test3): **27/28**。`Min(v2, v0·v1)` を複雑度 5・RMSE 0 で厳密発見、連続指数 $x^{1.7}$ を誤差 4e-15 で回収
- 1% ノイズ (test3_v2): **25/28**。離散構造 (Min/Max/Abs) はノイズに頑健
- **スパース性の変数特定: 10/10** — 25 変数中 5 変数のみ使う式でも無関係変数の混入ゼロ。ノイズ・外れ値下でも 10/10 を維持

### 8.4 外れ値頑健性 (test4: 1% ノイズ + 5% 外れ値 10σ)

- v4 (Huber なし): 19/28 → **v5 (Huber-IRLS): 20/28**

### 8.5 過学習の実測

- 訓練≪テスト乖離 (テスト RMSE > 10×訓練 RMSE) の件数: **0 件** (94 式判定、v4・v5 とも)
- test/train 比の中央値 0.05 — クリーンなテストへの誤差が訓練ノイズ床を大きく下回る = 構造を正しく回収している証拠
- 特筆事例: 「partial 判定」の C10 (25 変数スパース) でも発見式は `1.0004·(x2·x6·x10·x17·x21)` と構造・変数とも完全に正しい (係数 0.1% ずれで閾値を 2.7% 超過しただけ)

### 8.6 興味深い現象

- **ノイズの正則化効果**: 干渉項 I.37.4 ($I_1 + I_2 + 2\sqrt{I_1 I_2}\cos\delta$) など 3 式は**ノイズ下でのみ**完全回収された。検証分割 + ノイズ床早期終了が junk を抑え、真の構造が勝ちやすくなるため
- 発見式の例 (すべて実測):
  - I.40.1: `Exp(−(m·g·x)/(kb·T) + Log(n0))` — $y$ が 50 桁を張る指数則
  - I.26.2: `ArcSin(n·Sin(θ2))` — スネルの法則
  - Hill 式: 指数 2.5 を含む有理閉形式を RMSE 1.5e-15 で回収

---

## 9. 参考: リポジトリ内の関連ファイル

| 種類 | パス |
|------|------|
| エンジン本体 (Rust) | `eml-sr_fable/src/` (`engine/fable.rs` がパイプライン、`engine/powerlaw.rs` が Stage A/A2、`engine/bfs.rs` がビーム) |
| Python バインディング | `eml-sr_fable/src/python.rs` |
| ベンチ共通ロジック | `src/feynman_fable_common.py` |
| ベンチスクリプト | `src/feynman_eml_sr_fable_test1_v3.py` / `test1_v4.py` / `test1_v5.py` / `test2.py` / `test2_v2.py`、`src/general_eml_sr_fable_test_v3.py` / `_v5n.py`、`src/eml_sr_fable_test3.py` / `test3_v2.py` / `test4.py` |
| 総合レポート | `texts/eml_sr_fable_v5_report.md` (v5)、各 `texts/eml_sr_fable_*_report.md` |
| 別 PC 実行手順書 | `texts/20260707_手順書.md` |
| 作業ログ | `daily_report.md` |

---

*本マニュアルは 2026-07-09 時点の実装 (v5) に基づく。*
