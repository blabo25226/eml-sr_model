# eml-sr_fable

数値データ $(X, y)$ から閉形式の数式 $y \approx f(x_1,\ldots,x_d)$ を自動発見する **シンボリック回帰 (SR)** エンジンである。Rust で実装し、Python から `eml_sr_fable` として利用できる。

- **バージョン:** v5（2026-07-09 時点）
- **クレート:** `eml_sr_fable` 0.2.0（MIT）
- **ブランチ:** `claude/eml-sr-fable-algorithm-7w8j49`

詳細な解説・数式・ベンチマーク結果は [`manual_eml-sr_fable.md`](manual_eml-sr_fable.md) を参照。

---

## 特徴

ニューラルネットは使わず、**閉形式ソルバー（線形代数）+ ビームサーチ + Levenberg-Marquardt 定数最適化** による決定論的パイプラインで動作する。同じデータと設定では常に同じ結果が得られる。

v5 で追加・強化された主な機能:

| 機能 | 内容 |
|------|------|
| 4 段パイプライン | Stage A（変換×単項式和）→ Stage A2（有理関数）→ Stage C（乗法分解）→ Stage B（EML ビーム） |
| ノイズ・過学習対策 | 変換空間 WLS、80/20 検証分割、1SE 則ランキング、ノイズ床早期終了 |
| 外れ値対策 | Huber-IRLS リフィット |
| 演算子拡張 | `abs` / `sigmoid` / `min` / `max`、連続指数の LM 最適化 |
| 原空間 LM ポリッシュ | 変換空間 OLS のバイアスを除去する最終段 |

フォーク元 `eml-sr_model_cursor`（Feynman 99 式で 13/99、819 分）に対し、同一高速条件で **82/99（26 分）** を達成している（§8 参照）。

---

## セットアップ

### 前提

- Python 3.8+
- Rust（[rustup](https://rustup.rs/)）
- `pip install maturin numpy`

Windows では Visual Studio Build Tools（C++ デスクトップ開発）が必要。

### ビルドとインストール

リポジトリルートから:

```bash
cd eml-sr_fable
maturin develop --release --features python,full-math
```

動作確認:

```bash
python -c "import eml_sr_fable; print(eml_sr_fable.__file__)"
```

wheel として配布する場合:

```bash
maturin build --release --features python,full-math -o dist/
pip install dist/eml_sr_fable-*.whl
```

---

## クイックスタート（Python）

```python
import numpy as np
import eml_sr_fable

rng = np.random.default_rng(0)
X = rng.uniform(0.5, 3.0, size=(500, 2))
y = X[:, 0] ** 2 + 3.0 * X[:, 1]

searcher = eml_sr_fable.Searcher(
    max_complexity=6,
    beam_width=1000,
    time_budget_s=60.0,
    rational_stage=True,
    verbose=False,
)

candidates = searcher.find_candidates(X.tolist(), y.tolist())
best = candidates[0]
print(best.formula)
print(best.to_python())
preds = best.predict(X.tolist())
```

入力は **2 次元リストまたは NumPy 配列**（行 = サンプル、列 = 変数）。変数は `v_{0}`, `v_{1}`, … で表される。

### 主な API

| メソッド | 説明 |
|---------|------|
| `Searcher.find_candidates(X, y)` | 誤差×複雑度の Pareto front を返す |
| `Searcher.find_function(xs, ys)` | 1 変数、最良 1 件 |
| `Searcher.find_multivariate(X, y)` / `.fit(X, y)` | 多変数、最良 1 件 |
| `Searcher.recognize_constant(value)` | スカラー定数の閉形式同定 |
| `SearchResult.to_python()` / `.to_latex()` | 出力形式の変換 |
| `SearchResult.predict(X)` / `.eval_batch(X)` | 多変数の数値評価（2 次元入力） |
| `SearchResult.eval(x)` | 1 変数の数値評価（スカラーまたは 1 次元） |

ノイズがあるデータでは `early_exit_threshold=9e-3` 程度への引き上げを推奨する。候補選択は「最良誤差の 5% 帯内で最小複雑度」を推奨（`example_python.py` 参照）。

---

## デモの実行

```bash
cd eml-sr_fable
python example_python.py
```

定数同定・1 変数・多変数 Pareto front・`fit`/`predict`・EML 演算子・`eval_batch` を順に試す。

---

## ベンチマーク（抜粋）

Feynman 物理方程式 99 式（ホールドアウト評価、RMSE $< 10^{-4}$ で完全回収）:

| テスト | 条件 | 完全回収 |
|--------|------|---------|
| 1.test_v3 | 高速・クリーン | 82/99 |
| 2.test | 緩和・クリーン | 83/99 |
| 1.test_v5 | 高速・1% ノイズ | 71/99 |
| 2.test_v2 | 緩和・1% ノイズ | 73/99 |

一般合成 20 式: クリーン 19/20、1% ノイズ 18/20。演算子拡張 28 式: クリーン 27/28。

ベンチスクリプトはリポジトリの `src/` にあり、**`src/` から実行**する:

```bash
cd src
python feynman_eml_sr_fable_test1_v3.py --smoke
```

別 PC での長時間実行手順は `texts/20260707_手順書.md` を参照。

---

## ドキュメント

| ファイル | 内容 |
|---------|------|
| [`manual_eml-sr_fable.md`](manual_eml-sr_fable.md) | 解説書（セットアップ・API・アルゴリズム・比較・実績） |
| [`example_python.py`](example_python.py) | Python デモ |
| [`texts/eml_sr_fable_v5_report.md`](../texts/eml_sr_fable_v5_report.md) | v5 総合レポート |

---

## ライセンス

MIT（`LICENSE` 参照）
