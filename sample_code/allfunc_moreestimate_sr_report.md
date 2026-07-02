# EML-SR Feynman Equations 全式推定 (高精度設定) レポート

**生成日時:** 2026-05-22 03:54:59  
**ブランチ:** `20260521_claude_allfunc_estimate`  

---

## 1. 実験概要

### 目的

前回実験 (BEAM_WIDTH=300, N_SAMPLES=200) よりも大幅に高いパラメータ (BEAM_WIDTH=1000, N_SAMPLES=750) を用い、より多くの Feynman 方程式をデータのみから回収できるかを検証する。また、推定に失敗した数式についても eml-sr が出力した候補数式を記録し、数学的な関係を考察する材料とする。

### 実験設定

| パラメータ | 値 | 前回比 |
|-----------|-----|--------|
| N_SAMPLES | 750 | 200 → 750 (+275%) |
| BEAM_WIDTH | 1000 | 300 → 1000 (+233%) |
| MAX_COMPLEXITY | 6 | 変更なし |
| RMSE 閾値 (ok) | 0.0001 | 変更なし |
| 合計実行時間 | 7576s (126.3min) | — |

---

## 2. 総合結果サマリー

| 指標 | 値 |
|-----|-----|
| 対象方程式数 | 99 |
| ✅ 回収成功 (RMSE < 1e-4) | 9 (9.1%) |
| 🟡 部分的回収 (RMSE < 1e-2) | 1 (1.0%) |
| ❌ 失敗 | 84 |
| ⏭️ スキップ | 5 |
| **成功+部分合計** | **10 (10.1%)** |

---

## 3. 各方程式の詳細結果

| # | 式ID | 変数数 | ステータス | RMSE | 複雑度 | eml-sr 出力 | 時間(s) |
|---|------|--------|------------|------|--------|-------------|---------|
| 1 | `I.6.2a` | 1 | 🟡 partial | 6.013e-03 | 6 | `EML(ArcCos(ArcTan(Exp(v_{0}))), Pi)` | 83.8 |
| 2 | `I.6.2` | 2 | ❌ failed | 2.158e-02 | 6 | `Divide(ArcTan(1), ArcSin(Exp(v_{1})))` | 102.1 |
| 3 | `I.6.2b` | 3 | ❌ failed | 3.720e-02 | 6 | `Divide(Cos(ArcTan(v_{0})), ArcSin(Pi))` | 212.0 |
| 4 | `I.8.14` | 4 | ❌ failed | 9.848e-01 | 6 | `Plus(Sqrt(v_{1}), ArcCos(Log(v_{1})))` | 131.8 |
| 5 | `I.9.18` | 9 | ❌ failed | 9.910e-02 | 6 | `Times(v_{1}, Divide(v_{0}, ArcSin(Pi)))` | 240.4 |
| 6 | `I.10.7` | 3 | ❌ failed | 8.658e-02 | 5 | `Divide(v_{0}, ArcTan(ArcTan(v_{2})))` | 71.4 |
| 7 | `I.11.19` | 6 | ❌ failed | 7.507e+00 | 6 | `Plus(Exp(E), Times(v_{1}, v_{4}))` | 42.6 |
| 8 | `I.12.1` | 2 | ✅ ok | 0.000e+00 | 3 | `Times(v_{0}, v_{1})` | 16.6 |
| 9 | `I.12.2` | 4 | ❌ failed | 4.236e-02 | 6 | `Divide(Inv(v_{2}), EML(v_{3}, v_{0}))` | 150.3 |
| 10 | `I.12.4` | 3 | ❌ failed | 1.549e-02 | 6 | `Divide(Tan(E), EML(v_{2}, v_{0}))` | 130.7 |
| 11 | `I.12.5` | 2 | ✅ ok | 0.000e+00 | 3 | `Times(v_{0}, v_{1})` | 14.7 |
| 12 | `I.12.11` | 5 | ❌ failed | 1.816e+01 | 6 | `Times(v_{0}, Cos(Log(Neg(v_{4}))))` | 36.1 |
| 13 | `I.13.4` | 4 | ❌ failed | 1.937e+01 | 6 | `Times(v_{0}, EML(E, Inv(v_{3})))` | 28.2 |
| 14 | `I.13.12` | 5 | ❌ failed | 6.314e+00 | 5 | `Times(v_{1}, Subtract(v_{2}, v_{3}))` | 39.1 |
| 15 | `I.14.3` | 3 | ✅ ok | 3.254e-15 | 5 | `Times(v_{1}, Times(v_{0}, v_{2}))` | 24.5 |
| 16 | `I.14.4` | 2 | ❌ failed | 1.590e+00 | 6 | `Times(v_{0}, Sqrt(EML(v_{1}, Pi)))` | 16.3 |
| 17 | `I.15.3x` | 4 | ❌ failed | 2.101e-01 | 5 | `Subtract(v_{0}, Times(v_{1}, v_{3}))` | 47.6 |
| 18 | `I.15.3t` | 4 | ❌ failed | 1.011e-01 | 6 | `EML(Log(v_{3}), ArcTan(Sqrt(v_{0})))` | 92.8 |
| 19 | `I.15.1` | 3 | ❌ failed | 1.952e-01 | 6 | `Times(v_{1}, Plus(v_{0}, Inv(v_{2})))` | 26.2 |
| 20 | `I.16.6` | 3 | ❌ failed | 3.590e-01 | 5 | `Times(v_{0}, Sin(ArcTan(v_{1})))` | 85.0 |
| 21 | `I.18.4` | 4 | ❌ failed | 2.667e-01 | 6 | `Times(ArcTan(1), Plus(v_{2}, v_{3}))` | 99.1 |
| 22 | `I.18.12` | 2 | ⏭️ skipped | N/A | None | `N/A` | 0.0 |
| 23 | `I.18.14` | 3 | ⏭️ skipped | N/A | None | `N/A` | 0.0 |
| 24 | `I.24.6` | 4 | ❌ failed | 7.933e+00 | 6 | `Times(v_{0}, EML(v_{3}, Tan(v_{2})))` | 28.7 |
| 25 | `I.25.13` | 2 | ✅ ok | 1.180e-16 | 3 | `Divide(v_{0}, v_{1})` | 111.9 |
| 26 | `I.26.2` | 2 | ✅ ok | 4.258e-17 | 5 | `ArcSin(Times(v_{0}, Sin(v_{1})))` | 107.9 |
| 27 | `I.27.6` | 3 | ❌ failed | 1.557e-01 | 6 | `Plus(I, Sqrt(Divide(v_{1}, v_{2})))` | 142.6 |
| 28 | `I.29.4` | 2 | ✅ ok | 1.846e-16 | 3 | `Divide(v_{0}, v_{1})` | 77.6 |
| 29 | `I.29.16` | 4 | ❌ failed | 1.662e+00 | 6 | `Divide(Plus(v_{0}, v_{1}), Sqrt(E))` | 54.3 |
| 30 | `I.30.3` | 3 | ❌ failed | 1.944e+00 | 5 | `Times(v_{0}, Exp(Cos(v_{1})))` | 22.3 |
| 31 | `I.30.5` | 3 | ✅ ok | 2.671e-17 | 6 | `ArcSin(Divide(v_{0}, Times(v_{1}, v_{2})))` | 131.7 |
| 32 | `I.32.5` | 4 | ❌ failed | 5.779e-01 | 5 | `Tan(Divide(Log(v_{0}), v_{3}))` | 153.9 |
| 33 | `I.32.17` | 6 | ❌ failed | 4.520e+00 | 6 | `Divide(Exp(v_{4}), Subtract(v_{5}, E))` | 41.9 |
| 34 | `I.34.8` | 4 | ❌ failed | 7.028e+00 | 6 | `Times(v_{1}, Times(v_{0}, Log(v_{2})))` | 29.0 |
| 35 | `I.34.1` | 3 | ❌ failed | 4.994e-01 | 6 | `Divide(Times(Pi, v_{2}), Sqrt(v_{0}))` | 36.5 |
| 36 | `I.34.14` | 3 | ❌ failed | 3.916e-01 | 6 | `Divide(Times(Pi, v_{2}), Sqrt(v_{0}))` | 46.2 |
| 37 | `I.34.27` | 2 | ❌ failed | 1.340e-01 | 6 | `Divide(Log(v_{0}), Divide(Pi, v_{1}))` | 118.8 |
| 38 | `I.37.4` | 3 | ❌ failed | 1.751e+00 | 6 | `EML(Exp(Cos(v_{2})), Cos(v_{2}))` | 20.8 |
| 39 | `I.38.12` | 3 | ⏭️ skipped | N/A | None | `N/A` | 0.0 |
| 40 | `I.39.1` | 2 | ❌ failed | 8.287e-02 | 6 | `Times(v_{1}, Times(v_{0}, Tan(1)))` | 15.0 |
| 41 | `I.39.11` | 3 | ❌ failed | 1.547e+00 | 6 | `Divide(Times(v_{1}, v_{2}), Sqrt(v_{0}))` | 20.7 |
| 42 | `I.39.22` | 4 | ❌ failed | 6.956e+00 | 6 | `Times(v_{3}, Times(v_{0}, Log(v_{1})))` | 30.7 |
| 43 | `I.40.1` | 6 | ❌ failed | 4.804e-01 | 5 | `Divide(Divide(v_{0}, v_{4}), v_{1})` | 195.0 |
| 44 | `I.41.16` | 5 | ❌ failed | 1.896e+00 | 6 | `Times(v_{0}, Tan(Sqrt(ArcSin(v_{4}))))` | 36.8 |
| 45 | `I.43.16` | 4 | ❌ failed | 7.492e+00 | 6 | `Times(v_{2}, Times(v_{0}, Log(v_{1})))` | 30.3 |
| 46 | `I.43.31` | 3 | ✅ ok | 0.000e+00 | 5 | `Times(v_{1}, Times(v_{0}, v_{2}))` | 21.5 |
| 47 | `I.43.43` | 4 | ❌ failed | 1.008e+00 | 6 | `Times(Log(v_{1}), Divide(v_{3}, v_{2}))` | 111.2 |
| 48 | `I.44.4` | 5 | ❌ failed | 1.475e+01 | 6 | `Times(Exp(E), Subtract(v_{4}, v_{3}))` | 37.1 |
| 49 | `I.47.23` | 3 | ❌ failed | 2.802e-01 | 6 | `Divide(Plus(v_{0}, v_{1}), ArcSin(v_{2}))` | 146.2 |
| 50 | `I.48.2` | 3 | ❌ failed | 4.399e+00 | 5 | `Times(v_{0}, Times(v_{2}, v_{2}))` | 21.5 |
| 51 | `I.50.26` | 4 | ❌ failed | 1.790e+00 | 6 | `Subtract(Plus(v_{2}, v_{3}), Tan(1))` | 44.5 |
| 52 | `II.2.42` | 5 | ❌ failed | 3.924e+00 | 5 | `Times(v_{3}, Subtract(v_{2}, v_{1}))` | 37.8 |
| 53 | `II.3.24` | 2 | ❌ failed | 1.092e-02 | 6 | `Divide(Log(Sqrt(v_{0})), Exp(v_{1}))` | 113.1 |
| 54 | `II.4.23` | 3 | ❌ failed | 1.985e-02 | 6 | `Divide(Inv(v_{2}), Times(Pi, v_{1}))` | 131.9 |
| 55 | `II.6.11` | 4 | ❌ failed | 1.516e-02 | 6 | `Tan(Divide(Cos(v_{2}), ArcSin(E)))` | 163.8 |
| 56 | `II.6.15a` | 6 | ❌ failed | 1.897e-01 | 6 | `Divide(Inv(v_{0}), Log(Tan(v_{2})))` | 187.1 |
| 57 | `II.6.15b` | 4 | ❌ failed | 2.526e-02 | 6 | `Divide(Cos(v_{2}), Exp(Exp(v_{3})))` | 182.8 |
| 58 | `II.8.7` | 3 | ❌ failed | 6.515e-02 | 6 | `Times(v_{0}, Divide(Sqrt(I), v_{2}))` | 139.2 |
| 59 | `II.8.31` | 2 | ❌ failed | 1.621e+00 | 6 | `Times(v_{0}, Sqrt(EML(v_{1}, Pi)))` | 16.5 |
| 60 | `II.10.9` | 3 | ❌ failed | 7.008e-02 | 6 | `Divide(Divide(v_{0}, v_{1}), ArcSin(v_{2}))` | 139.8 |
| 61 | `II.11.3` | 5 | ❌ failed | 7.792e-02 | 6 | `Divide(Divide(v_{0}, v_{3}), ArcSin(v_{2}))` | 188.2 |
| 62 | `II.11.17` | 6 | ❌ failed | 1.042e+00 | 6 | `Times(v_{0}, ArcCos(Tan(Log(v_{3}))))` | 151.5 |
| 63 | `II.11.20` | 5 | ❌ failed | 4.623e+00 | 6 | `Times(v_{0}, Divide(v_{1}, Sqrt(v_{4})))` | 35.8 |
| 64 | `II.11.27` | 4 | ❌ failed | 2.364e-01 | 6 | `Times(E, ArcSin(Times(v_{0}, v_{1})))` | 174.0 |
| 65 | `II.11.28` | 2 | ❌ failed | 3.790e-02 | 6 | `Plus(I, Exp(Times(v_{0}, v_{1})))` | 121.3 |
| 66 | `II.13.17` | 4 | ❌ failed | 1.804e-02 | 6 | `Divide(Divide(I, v_{1}), Sqrt(v_{3}))` | 165.5 |
| 67 | `II.13.23` | 3 | ❌ failed | 1.006e-01 | 5 | `Divide(v_{0}, ArcTan(ArcTan(v_{2})))` | 72.5 |
| 68 | `II.13.34` | 3 | ❌ failed | 2.088e-01 | 6 | `Times(v_{1}, Plus(v_{0}, Inv(v_{2})))` | 25.3 |
| 69 | `II.15.4` | 3 | ❌ failed | 3.293e+00 | 6 | `Divide(Cos(v_{2}), EML(0, Pi))` | 22.1 |
| 70 | `II.15.5` | 3 | ❌ failed | 3.419e+00 | 5 | `Divide(v_{0}, Exp(Cos(v_{2})))` | 22.0 |
| 71 | `II.21.32` | 5 | ❌ failed | 2.919e-02 | 6 | `Divide(Inv(v_{1}), Times(Pi, v_{2}))` | 186.2 |
| 72 | `II.24.17` | 3 | ❌ failed | 9.598e-02 | 6 | `Subtract(Divide(v_{0}, v_{1}), Inv(v_{0}))` | 79.2 |
| 73 | `II.27.16` | 3 | ❌ failed | 5.369e+01 | 6 | `Times(v_{1}, Exp(Exp(ArcTan(v_{2}))))` | 21.5 |
| 74 | `II.27.18` | 2 | ✅ ok | 0.000e+00 | 5 | `Times(v_{0}, Times(v_{1}, v_{1}))` | 15.4 |
| 75 | `II.34.2a` | 3 | ❌ failed | 2.541e-01 | 6 | `Times(Sin(E), Divide(v_{1}, v_{2}))` | 142.4 |
| 76 | `II.34.2` | 3 | ❌ failed | 2.995e+00 | 6 | `Times(v_{2}, Times(v_{1}, Sqrt(v_{0})))` | 22.7 |
| 77 | `II.34.11` | 4 | ❌ failed | 2.941e+00 | 6 | `Divide(Times(v_{0}, v_{1}), Sqrt(v_{3}))` | 34.6 |
| 78 | `II.34.29a` | 3 | ❌ failed | 1.263e-01 | 5 | `Divide(v_{0}, Times(Pi, v_{2}))` | 140.6 |
| 79 | `II.34.29b` | 5 | ❌ failed | 1.756e+02 | 6 | `Times(v_{3}, Times(v_{2}, Exp(Pi)))` | 38.2 |
| 80 | `II.35.18` | 5 | ❌ failed | 2.181e-01 | 6 | `ArcSin(Divide(v_{0}, Plus(v_{3}, v_{4})))` | 198.7 |
| 81 | `II.35.21` | 5 | ❌ failed | 2.115e+00 | 6 | `Times(v_{0}, Subtract(v_{1}, Sqrt(I)))` | 36.6 |
| 82 | `II.36.38` | 8 | ❌ failed | 8.275e-01 | 6 | `EML(Inv(v_{2}), Divide(v_{3}, v_{0}))` | 197.5 |
| 83 | `II.38.3` | 4 | ❌ failed | 7.515e+00 | 6 | `Times(v_{3}, Times(v_{1}, Log(v_{0})))` | 33.2 |
| 84 | `II.38.14` | 2 | ❌ failed | 4.713e-02 | 6 | `Sin(Divide(v_{0}, Times(E, v_{1})))` | 117.9 |
| 85 | `III.4.32` | 4 | ❌ failed | 6.901e+00 | 6 | `Times(v_{3}, Plus(v_{2}, Sin(v_{0})))` | 29.3 |
| 86 | `III.4.33` | 4 | ❌ failed | 3.690e-01 | 6 | `Subtract(Times(v_{2}, v_{3}), Log(Pi))` | 30.5 |
| 87 | `III.7.38` | 3 | ❌ failed | 2.922e+01 | 5 | `Times(v_{0}, Times(v_{0}, v_{1}))` | 21.5 |
| 88 | `III.8.54` | 3 | ❌ failed | 3.512e-01 | 6 | `EML(Sqrt(Tan(ArcCos(v_{0}))), E)` | 150.2 |
| 89 | `III.9.52` | 6 | ❌ failed | 1.265e+01 | 6 | `Times(v_{1}, Exp(EML(1, v_{3})))` | 42.8 |
| 90 | `III.10.19` | 3 | ⏭️ skipped | N/A | None | `N/A` | 0.0 |
| 91 | `III.12.43` | 2 | ❌ failed | 1.773e-01 | 6 | `Divide(Log(v_{1}), Divide(Pi, v_{0}))` | 118.1 |
| 92 | `III.13.18` | 4 | ❌ failed | 3.597e+02 | 6 | `Times(v_{0}, Times(v_{2}, Exp(v_{1})))` | 29.1 |
| 93 | `III.14.14` | 5 | ❌ failed | 5.294e+00 | 5 | `Times(v_{1}, Times(v_{0}, v_{2}))` | 37.0 |
| 94 | `III.15.12` | 3 | ❌ failed | 4.569e+00 | 6 | `EML(Sqrt(v_{0}), Sin(Log(v_{2})))` | 21.6 |
| 95 | `III.15.14` | 3 | ❌ failed | 1.113e-02 | 6 | `Times(v_{0}, Divide(Sqrt(I), v_{2}))` | 136.2 |
| 96 | `III.15.27` | 3 | ❌ failed | 1.480e+00 | 6 | `Divide(Exp(E), Times(v_{1}, v_{2}))` | 21.8 |
| 97 | `III.17.37` | 3 | ❌ failed | 2.745e+00 | 6 | `Times(Cos(v_{2}), Sqrt(Exp(v_{1})))` | 21.2 |
| 98 | `III.19.51` | 4 | ⏭️ skipped | N/A | None | `N/A` | 0.0 |
| 99 | `III.21.20` | 4 | ❌ failed | 6.997e+00 | 6 | `Times(v_{2}, Subtract(Sin(v_{0}), v_{1}))` | 29.4 |

---

## 4. 回収成功した方程式の詳細

### `I.12.1`

- **変数:** mu, Nn
- **eml-sr 発見式:** `Times(v_{0}, v_{1})`
- **Python 表現:** `(v_{0})*(v_{1})`
- **RMSE:** 0.000e+00
- **複雑度:** 3
- **実行時間:** 16.57s

### `I.12.5`

- **変数:** q2, Ef
- **eml-sr 発見式:** `Times(v_{0}, v_{1})`
- **Python 表現:** `(v_{0})*(v_{1})`
- **RMSE:** 0.000e+00
- **複雑度:** 3
- **実行時間:** 14.72s

### `I.14.3`

- **変数:** m, g, z
- **eml-sr 発見式:** `Times(v_{1}, Times(v_{0}, v_{2}))`
- **Python 表現:** `(v_{1})*((v_{0})*(v_{2}))`
- **RMSE:** 3.254e-15
- **複雑度:** 5
- **実行時間:** 24.54s

### `I.25.13`

- **変数:** q, C
- **eml-sr 発見式:** `Divide(v_{0}, v_{1})`
- **Python 表現:** `(v_{0}/v_{1})`
- **RMSE:** 1.180e-16
- **複雑度:** 3
- **実行時間:** 111.85s

### `I.26.2`

- **変数:** n, theta2
- **eml-sr 発見式:** `ArcSin(Times(v_{0}, Sin(v_{1})))`
- **Python 表現:** `np.arcsin((v_{0})*(np.sin(v_{1})))`
- **RMSE:** 4.258e-17
- **複雑度:** 5
- **実行時間:** 107.94s

### `I.29.4`

- **変数:** omega, c
- **eml-sr 発見式:** `Divide(v_{0}, v_{1})`
- **Python 表現:** `(v_{0}/v_{1})`
- **RMSE:** 1.846e-16
- **複雑度:** 3
- **実行時間:** 77.64s

### `I.30.5`

- **変数:** lambd, d, n
- **eml-sr 発見式:** `ArcSin(Divide(v_{0}, Times(v_{1}, v_{2})))`
- **Python 表現:** `np.arcsin((v_{0}/(v_{1})*(v_{2})))`
- **RMSE:** 2.671e-17
- **複雑度:** 6
- **実行時間:** 131.69s

### `I.43.31`

- **変数:** mob, T, kb
- **eml-sr 発見式:** `Times(v_{1}, Times(v_{0}, v_{2}))`
- **Python 表現:** `(v_{1})*((v_{0})*(v_{2}))`
- **RMSE:** 0.000e+00
- **複雑度:** 5
- **実行時間:** 21.50s

### `II.27.18`

- **変数:** epsilon, Ef
- **eml-sr 発見式:** `Times(v_{0}, Times(v_{1}, v_{1}))`
- **Python 表現:** `(v_{0})*((v_{1})*(v_{1}))`
- **RMSE:** 0.000e+00
- **複雑度:** 5
- **実行時間:** 15.45s

---

## 5. 部分的回収の方程式

### `I.6.2a`
- **変数:** theta
- **eml-sr 発見式:** `EML(ArcCos(ArcTan(Exp(v_{0}))), Pi)`
- **RMSE:** 6.013e-03
- **複雑度:** 6

---

## 6. 失敗した方程式と eml-sr 出力数式

以下は推定に失敗（RMSE >= 1e-4）した方程式の一覧である。eml-sr が出力した数式（Pareto front の最良候補）を記録する。これらの出力数式から数学的変換により正しい数式を導ける可能性がある。

| # | 式ID | 変数 | RMSE | eml-sr 出力式 | eml-sr Python 表現 |
|---|------|------|------|--------------|-------------------|
| 2 | `I.6.2` | sigma, theta | 2.158e-02 | `Divide(ArcTan(1), ArcSin(Exp(v_{1})))` | `(np.arctan(0.7487)/np.arcsin(np.exp(v_{1})))` |
| 3 | `I.6.2b` | sigma, theta, theta1 | 3.720e-02 | `Divide(Cos(ArcTan(v_{0})), ArcSin(Pi))` | `(np.cos(np.arctan(v_{0}))/np.arcsin(1.6997))` |
| 4 | `I.8.14` | x1, x2, y1, y2 | 9.848e-01 | `Plus(Sqrt(v_{1}), ArcCos(Log(v_{1})))` | `(np.sqrt(v_{1}))+(np.arccos(np.log(v_{1})))` |
| 5 | `I.9.18` | m1, m2, G, x1, x2, y1, y2, z1, z2 | 9.910e-02 | `Times(v_{1}, Divide(v_{0}, ArcSin(Pi)))` | `(v_{1})*((v_{0}/np.arcsin(10.9756)))` |
| 6 | `I.10.7` | m_0, v, c | 8.658e-02 | `Divide(v_{0}, ArcTan(ArcTan(v_{2})))` | `(v_{0}/np.arctan(np.arctan(v_{2})))` |
| 7 | `I.11.19` | x1, x2, x3, y1, y2, y3 | 7.507e+00 | `Plus(Exp(E), Times(v_{1}, v_{4}))` | `(np.exp(np.e))+((v_{1})*(v_{4}))` |
| 9 | `I.12.2` | q1, q2, epsilon, r | 4.236e-02 | `Divide(Inv(v_{2}), EML(v_{3}, v_{0}))` | `((1/v_{2})/(np.exp(v_{3}) - np.log(v_{0})))` |
| 10 | `I.12.4` | q1, epsilon, r | 1.549e-02 | `Divide(Tan(E), EML(v_{2}, v_{0}))` | `(np.tan(42380.2511)/(np.exp(v_{2}) - np.log(v_{0})))` |
| 12 | `I.12.11` | q, Ef, B, v, theta | 1.816e+01 | `Times(v_{0}, Cos(Log(Neg(v_{4}))))` | `(v_{0})*(np.cos(np.log((-v_{4}))))` |
| 13 | `I.13.4` | m, v, u, w | 1.937e+01 | `Times(v_{0}, EML(E, Inv(v_{3})))` | `(v_{0})*((np.exp(np.e) - np.log((1/v_{3}))))` |
| 14 | `I.13.12` | m1, m2, r1, r2, G | 6.314e+00 | `Times(v_{1}, Subtract(v_{2}, v_{3}))` | `(v_{1})*((v_{2})-(v_{3}))` |
| 16 | `I.14.4` | k_spring, x | 1.590e+00 | `Times(v_{0}, Sqrt(EML(v_{1}, Pi)))` | `(v_{0})*(np.sqrt((np.exp(v_{1}) - np.log(13.9731))))` |
| 17 | `I.15.3x` | x, u, c, t | 2.101e-01 | `Subtract(v_{0}, Times(v_{1}, v_{3}))` | `(v_{0})-((v_{1})*(v_{3}))` |
| 18 | `I.15.3t` | x, c, u, t | 1.011e-01 | `EML(Log(v_{3}), ArcTan(Sqrt(v_{0})))` | `(np.exp(np.log(v_{3})) - np.log(np.arctan(np.sqrt(v_{0}))))` |
| 19 | `I.15.1` | m_0, v, c | 1.952e-01 | `Times(v_{1}, Plus(v_{0}, Inv(v_{2})))` | `(v_{1})*((v_{0})+((1/v_{2})))` |
| 20 | `I.16.6` | c, v, u | 3.590e-01 | `Times(v_{0}, Sin(ArcTan(v_{1})))` | `(v_{0})*(np.sin(np.arctan(v_{1})))` |
| 21 | `I.18.4` | m1, m2, r1, r2 | 2.667e-01 | `Times(ArcTan(1), Plus(v_{2}, v_{3}))` | `(np.arctan(0.5802))*((v_{2})+(v_{3}))` |
| 24 | `I.24.6` | m, omega, omega_0, x | 7.933e+00 | `Times(v_{0}, EML(v_{3}, Tan(v_{2})))` | `(v_{0})*((np.exp(v_{3}) - np.log(np.tan(v_{2}))))` |
| 27 | `I.27.6` | d1, d2, n | 1.557e-01 | `Plus(I, Sqrt(Divide(v_{1}, v_{2})))` | `(-0.2091)+(np.sqrt((v_{1}/v_{2})))` |
| 29 | `I.29.16` | x1, x2, theta1, theta2 | 1.662e+00 | `Divide(Plus(v_{0}, v_{1}), Sqrt(E))` | `((v_{0})+(v_{1})/np.sqrt(2.7811))` |
| 30 | `I.30.3` | Int_0, theta, n | 1.944e+00 | `Times(v_{0}, Exp(Cos(v_{1})))` | `(v_{0})*(np.exp(np.cos(v_{1})))` |
| 32 | `I.32.5` | q, a, epsilon, c | 5.779e-01 | `Tan(Divide(Log(v_{0}), v_{3}))` | `np.tan((np.log(v_{0})/v_{3}))` |
| 33 | `I.32.17` | epsilon, c, Ef, r, omega, omega_0 | 4.520e+00 | `Divide(Exp(v_{4}), Subtract(v_{5}, E))` | `(np.exp(v_{4})/(v_{5})-(np.e))` |
| 34 | `I.34.8` | q, v, B, p | 7.028e+00 | `Times(v_{1}, Times(v_{0}, Log(v_{2})))` | `(v_{1})*((v_{0})*(np.log(v_{2})))` |
| 35 | `I.34.1` | c, v, omega_0 | 4.994e-01 | `Divide(Times(Pi, v_{2}), Sqrt(v_{0}))` | `((3.4315)*(v_{2})/np.sqrt(v_{0}))` |
| 36 | `I.34.14` | c, v, omega_0 | 3.916e-01 | `Divide(Times(Pi, v_{2}), Sqrt(v_{0}))` | `((3.1805)*(v_{2})/np.sqrt(v_{0}))` |
| 37 | `I.34.27` | omega, h | 1.340e-01 | `Divide(Log(v_{0}), Divide(Pi, v_{1}))` | `(np.log(v_{0})/(2.1675/v_{1}))` |
| 38 | `I.37.4` | I1, I2, delta | 1.751e+00 | `EML(Exp(Cos(v_{2})), Cos(v_{2}))` | `(np.exp(np.exp(np.cos(v_{2}))) - np.log(np.cos(v_{2})))` |
| 40 | `I.39.1` | pr, V | 8.287e-02 | `Times(v_{1}, Times(v_{0}, Tan(1)))` | `(v_{1})*((v_{0})*(np.tan(0.9853)))` |
| 41 | `I.39.11` | gamma, pr, V | 1.547e+00 | `Divide(Times(v_{1}, v_{2}), Sqrt(v_{0}))` | `((v_{1})*(v_{2})/np.sqrt(v_{0}))` |
| 42 | `I.39.22` | n, T, V, kb | 6.956e+00 | `Times(v_{3}, Times(v_{0}, Log(v_{1})))` | `(v_{3})*((v_{0})*(np.log(v_{1})))` |
| 43 | `I.40.1` | n_0, m, x, T, g, kb | 4.804e-01 | `Divide(Divide(v_{0}, v_{4}), v_{1})` | `((v_{0}/v_{4})/v_{1})` |
| 44 | `I.41.16` | omega, T, h, kb, c | 1.896e+00 | `Times(v_{0}, Tan(Sqrt(ArcSin(v_{4}))))` | `(v_{0})*(np.tan(np.sqrt(np.arcsin(v_{4}))))` |
| 45 | `I.43.16` | mu_drift, q, Volt, d | 7.492e+00 | `Times(v_{2}, Times(v_{0}, Log(v_{1})))` | `(v_{2})*((v_{0})*(np.log(v_{1})))` |
| 47 | `I.43.43` | gamma, kb, A, v | 1.008e+00 | `Times(Log(v_{1}), Divide(v_{3}, v_{2}))` | `(np.log(v_{1}))*((v_{3}/v_{2}))` |
| 48 | `I.44.4` | n, kb, T, V1, V2 | 1.475e+01 | `Times(Exp(E), Subtract(v_{4}, v_{3}))` | `(np.exp(np.e))*((v_{4})-(v_{3}))` |
| 49 | `I.47.23` | gamma, pr, rho | 2.802e-01 | `Divide(Plus(v_{0}, v_{1}), ArcSin(v_{2}))` | `((v_{0})+(v_{1})/np.arcsin(v_{2}))` |
| 50 | `I.48.2` | m, v, c | 4.399e+00 | `Times(v_{0}, Times(v_{2}, v_{2}))` | `(v_{0})*((v_{2})*(v_{2}))` |
| 51 | `I.50.26` | x1, omega, t, alpha | 1.790e+00 | `Subtract(Plus(v_{2}, v_{3}), Tan(1))` | `((v_{2})+(v_{3}))-(np.tan(214.7923))` |
| 52 | `II.2.42` | kappa, T1, T2, A, d | 3.924e+00 | `Times(v_{3}, Subtract(v_{2}, v_{1}))` | `(v_{3})*((v_{2})-(v_{1}))` |
| 53 | `II.3.24` | Pwr, r | 1.092e-02 | `Divide(Log(Sqrt(v_{0})), Exp(v_{1}))` | `(np.log(np.sqrt(v_{0}))/np.exp(v_{1}))` |
| 54 | `II.4.23` | q, epsilon, r | 1.985e-02 | `Divide(Inv(v_{2}), Times(Pi, v_{1}))` | `((1/v_{2})/(5.2707)*(v_{1}))` |
| 55 | `II.6.11` | epsilon, p_d, theta, r | 1.516e-02 | `Tan(Divide(Cos(v_{2}), ArcSin(E)))` | `np.tan((np.cos(v_{2})/np.arcsin(568.5703)))` |
| 56 | `II.6.15a` | epsilon, p_d, r, x, y, z | 1.897e-01 | `Divide(Inv(v_{0}), Log(Tan(v_{2})))` | `((1/v_{0})/np.log(np.tan(v_{2})))` |
| 57 | `II.6.15b` | epsilon, p_d, theta, r | 2.526e-02 | `Divide(Cos(v_{2}), Exp(Exp(v_{3})))` | `(np.cos(v_{2})/np.exp(np.exp(v_{3})))` |
| 58 | `II.8.7` | q, epsilon, d | 6.515e-02 | `Times(v_{0}, Divide(Sqrt(I), v_{2}))` | `(v_{0})*((np.sqrt(0.0060)/v_{2}))` |
| 59 | `II.8.31` | epsilon, Ef | 1.621e+00 | `Times(v_{0}, Sqrt(EML(v_{1}, Pi)))` | `(v_{0})*(np.sqrt((np.exp(v_{1}) - np.log(11.4920))))` |
| 60 | `II.10.9` | sigma_den, epsilon, chi | 7.008e-02 | `Divide(Divide(v_{0}, v_{1}), ArcSin(v_{2}))` | `((v_{0}/v_{1})/np.arcsin(v_{2}))` |
| 61 | `II.11.3` | q, Ef, m, omega_0, omega | 7.792e-02 | `Divide(Divide(v_{0}, v_{3}), ArcSin(v_{2}))` | `((v_{0}/v_{3})/np.arcsin(v_{2}))` |
| 62 | `II.11.17` | n_0, kb, T, theta, p_d, Ef | 1.042e+00 | `Times(v_{0}, ArcCos(Tan(Log(v_{3}))))` | `(v_{0})*(np.arccos(np.tan(np.log(v_{3}))))` |
| 63 | `II.11.20` | n_rho, p_d, Ef, kb, T | 4.623e+00 | `Times(v_{0}, Divide(v_{1}, Sqrt(v_{4})))` | `(v_{0})*((v_{1}/np.sqrt(v_{4})))` |
| 64 | `II.11.27` | n, alpha, epsilon, Ef | 2.364e-01 | `Times(E, ArcSin(Times(v_{0}, v_{1})))` | `(2.5998)*(np.arcsin((v_{0})*(v_{1})))` |
| 65 | `II.11.28` | n, alpha | 3.790e-02 | `Plus(I, Exp(Times(v_{0}, v_{1})))` | `(-0.0180)+(np.exp((v_{0})*(v_{1})))` |
| 66 | `II.13.17` | epsilon, c, I, r | 1.804e-02 | `Divide(Divide(I, v_{1}), Sqrt(v_{3}))` | `((0.0764/v_{1})/np.sqrt(v_{3}))` |
| 67 | `II.13.23` | rho_c_0, v, c | 1.006e-01 | `Divide(v_{0}, ArcTan(ArcTan(v_{2})))` | `(v_{0}/np.arctan(np.arctan(v_{2})))` |
| 68 | `II.13.34` | rho_c_0, v, c | 2.088e-01 | `Times(v_{1}, Plus(v_{0}, Inv(v_{2})))` | `(v_{1})*((v_{0})+((1/v_{2})))` |
| 69 | `II.15.4` | mom, B, theta | 3.293e+00 | `Divide(Cos(v_{2}), EML(0, Pi))` | `(np.cos(v_{2})/(np.exp(0) - np.log(np.pi)))` |
| 70 | `II.15.5` | p_d, Ef, theta | 3.419e+00 | `Divide(v_{0}, Exp(Cos(v_{2})))` | `(v_{0}/np.exp(np.cos(v_{2})))` |
| 71 | `II.21.32` | q, epsilon, r, v, c | 2.919e-02 | `Divide(Inv(v_{1}), Times(Pi, v_{2}))` | `((1/v_{1})/(3.1092)*(v_{2}))` |
| 72 | `II.24.17` | omega, c, d | 9.598e-02 | `Subtract(Divide(v_{0}, v_{1}), Inv(v_{0}))` | `((v_{0}/v_{1}))-((1/v_{0}))` |
| 73 | `II.27.16` | epsilon, c, Ef | 5.369e+01 | `Times(v_{1}, Exp(Exp(ArcTan(v_{2}))))` | `(v_{1})*(np.exp(np.exp(np.arctan(v_{2}))))` |
| 75 | `II.34.2a` | q, v, r | 2.541e-01 | `Times(Sin(E), Divide(v_{1}, v_{2}))` | `(np.sin(-1147.1993))*((v_{1}/v_{2}))` |
| 76 | `II.34.2` | q, v, r | 2.995e+00 | `Times(v_{2}, Times(v_{1}, Sqrt(v_{0})))` | `(v_{2})*((v_{1})*(np.sqrt(v_{0})))` |
| 77 | `II.34.11` | g_, q, B, m | 2.941e+00 | `Divide(Times(v_{0}, v_{1}), Sqrt(v_{3}))` | `((v_{0})*(v_{1})/np.sqrt(v_{3}))` |
| 78 | `II.34.29a` | q, h, m | 1.263e-01 | `Divide(v_{0}, Times(Pi, v_{2}))` | `(v_{0}/(4.1798)*(v_{2}))` |
| 79 | `II.34.29b` | g_, h, Jz, mom, B | 1.756e+02 | `Times(v_{3}, Times(v_{2}, Exp(Pi)))` | `(v_{3})*((v_{2})*(np.exp(np.pi)))` |
| 80 | `II.35.18` | n_0, kb, T, mom, B | 2.181e-01 | `ArcSin(Divide(v_{0}, Plus(v_{3}, v_{4})))` | `np.arcsin((v_{0}/(v_{3})+(v_{4})))` |
| 81 | `II.35.21` | n_rho, mom, B, kb, T | 2.115e+00 | `Times(v_{0}, Subtract(v_{1}, Sqrt(I)))` | `(v_{0})*((v_{1})-(np.sqrt(1j)))` |
| 82 | `II.36.38` | mom, H, kb, T, alpha, epsilon, c, M | 8.275e-01 | `EML(Inv(v_{2}), Divide(v_{3}, v_{0}))` | `(np.exp((1/v_{2})) - np.log((v_{3}/v_{0})))` |
| 83 | `II.38.3` | Y, A, d, x | 7.515e+00 | `Times(v_{3}, Times(v_{1}, Log(v_{0})))` | `(v_{3})*((v_{1})*(np.log(v_{0})))` |
| 84 | `II.38.14` | Y, sigma | 4.713e-02 | `Sin(Divide(v_{0}, Times(E, v_{1})))` | `np.sin((v_{0}/(2.8301)*(v_{1})))` |
| 85 | `III.4.32` | h, omega, kb, T | 6.901e+00 | `Times(v_{3}, Plus(v_{2}, Sin(v_{0})))` | `(v_{3})*((v_{2})+(np.sin(v_{0})))` |
| 86 | `III.4.33` | h, omega, kb, T | 3.690e-01 | `Subtract(Times(v_{2}, v_{3}), Log(Pi))` | `((v_{2})*(v_{3}))-(np.log(2.0003))` |
| 87 | `III.7.38` | mom, B, h | 2.922e+01 | `Times(v_{0}, Times(v_{0}, v_{1}))` | `(v_{0})*((v_{0})*(v_{1}))` |
| 88 | `III.8.54` | E_n, t, h | 3.512e-01 | `EML(Sqrt(Tan(ArcCos(v_{0}))), E)` | `(np.exp(np.sqrt(np.tan(np.arccos(v_{0})))) - np.log(2.7598))` |
| 89 | `III.9.52` | p_d, Ef, t, h, omega, omega_0 | 1.265e+01 | `Times(v_{1}, Exp(EML(1, v_{3})))` | `(v_{1})*(np.exp((np.exp(1) - np.log(v_{3}))))` |
| 91 | `III.12.43` | n, h | 1.773e-01 | `Divide(Log(v_{1}), Divide(Pi, v_{0}))` | `(np.log(v_{1})/(2.3155/v_{0}))` |
| 92 | `III.13.18` | E_n, d, k, h | 3.597e+02 | `Times(v_{0}, Times(v_{2}, Exp(v_{1})))` | `(v_{0})*((v_{2})*(np.exp(v_{1})))` |
| 93 | `III.14.14` | I_0, q, Volt, kb, T | 5.294e+00 | `Times(v_{1}, Times(v_{0}, v_{2}))` | `(v_{1})*((v_{0})*(v_{2}))` |
| 94 | `III.15.12` | U, k, d | 4.569e+00 | `EML(Sqrt(v_{0}), Sin(Log(v_{2})))` | `(np.exp(np.sqrt(v_{0})) - np.log(np.sin(np.log(v_{2}))))` |
| 95 | `III.15.14` | h, E_n, d | 1.113e-02 | `Times(v_{0}, Divide(Sqrt(I), v_{2}))` | `(v_{0})*((np.sqrt(0.0001)/v_{2}))` |
| 96 | `III.15.27` | alpha, n, d | 1.480e+00 | `Divide(Exp(E), Times(v_{1}, v_{2}))` | `(np.exp(2.8509)/(v_{1})*(v_{2}))` |
| 97 | `III.17.37` | beta, alpha, theta | 2.745e+00 | `Times(Cos(v_{2}), Sqrt(Exp(v_{1})))` | `(np.cos(v_{2}))*(np.sqrt(np.exp(v_{1})))` |
| 99 | `III.21.20` | rho_c_0, q, A_vec, m | 6.997e+00 | `Times(v_{2}, Subtract(Sin(v_{0}), v_{1}))` | `(v_{2})*((np.sin(v_{0}))-(v_{1}))` |

---

## 7. 失敗した方程式の Pareto Front 全候補

eml-sr は各実行で複数の候補（Pareto front）を返す。以下に失敗した各方程式の全候補を示す。複雑度が低い候補ほど一般化しやすく、最良の複雑度/精度バランスを持つ候補が「真の数式の部分構造」を含む可能性がある。

### `I.6.2`  — 正しい数式: `exp(-(theta/sigma)**2/2)/(sqrt(2*pi)*sigma)` （変数: sigma, theta）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.225e-01 | `0` | `0` |
| 2 | 4.743e-02 | `Sqrt(I)` | `np.sqrt(0.0086)` |
| 3 | 4.193e-02 | `Divide(I, E)` | `(0.3177/2.7183)` |
| 4 | 2.462e-02 | `Divide(Sin(E), v_{1})` | `(np.sin(2.9204)/v_{1})` |
| 5 | 2.283e-02 | `Divide(Exp(ArcSin(Pi)), v_{1})` | `(np.exp(np.arcsin(-148459.0924))/v_{1})` |
| 6 | 2.158e-02 | `Divide(ArcTan(1), ArcSin(Exp(v_{1})))` | `(np.arctan(0.7487)/np.arcsin(np.exp(v_{1})))` |

### `I.6.2b`  — 正しい数式: `exp(-((theta-theta1)/sigma)**2/2)/(sqrt(2*pi)*sigma)` （変数: sigma, theta, theta1）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 2.044e-01 | `0` | `0` |
| 2 | 6.710e-02 | `Tan(1)` | `np.tan(-349973.2555)` |
| 3 | 6.134e-02 | `Divide(I, E)` | `(0.5382/2.7183)` |
| 4 | 4.131e-02 | `Inv(Times(E, v_{0}))` | `(1/(2.8046)*(v_{0}))` |
| 5 | 3.776e-02 | `Divide(Sin(Inv(v_{0})), E)` | `(np.sin((1/v_{0}))/2.7014)` |
| 6 | 3.720e-02 | `Divide(Cos(ArcTan(v_{0})), ArcSin(Pi))` | `(np.cos(np.arctan(v_{0}))/np.arcsin(1.6997))` |

### `I.8.14`  — 正しい数式: `sqrt((x2-x1)**2+(y2-y1)**2)` （変数: x1, x2, y1, y2）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.194e+00 | `E` | `np.e` |
| 2 | 1.020e+00 | `Tan(1)` | `np.tan(1.1379)` |
| 3 | 1.017e+00 | `Exp(ArcSin(I))` | `np.exp(np.arcsin(820.6342))` |
| 4 | 9.982e-01 | `Plus(E, Cos(v_{1}))` | `(2.5770)+(np.cos(v_{1}))` |
| 5 | 9.877e-01 | `EML(0, Cos(Sin(v_{1})))` | `(np.exp(0.5543) - np.log(np.cos(np.sin(v_{1}))))` |
| 6 | 9.848e-01 | `Plus(Sqrt(v_{1}), ArcCos(Log(v_{1})))` | `(np.sqrt(v_{1}))+(np.arccos(np.log(v_{1})))` |

### `I.9.18`  — 正しい数式: `G*m1*m2/((x2-x1)**2+(y2-y1)**2+(z2-z1)**2)` （変数: m1, m2, G, x1, x2, y1, y2, z1, z2）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 3.195e-01 | `0` | `0` |
| 2 | 1.306e-01 | `Inv(v_{3})` | `(1/v_{3})` |
| 3 | 1.182e-01 | `Divide(v_{0}, Pi)` | `(v_{0}/5.3076)` |
| 4 | 1.174e-01 | `Tan(Divide(v_{0}, Pi))` | `np.tan((v_{0}/5.3076))` |
| 5 | 1.142e-01 | `Times(v_{0}, Divide(I, v_{3}))` | `(v_{0})*((0.6318/v_{3}))` |
| 6 | 9.910e-02 | `Times(v_{1}, Divide(v_{0}, ArcSin(Pi)))` | `(v_{1})*((v_{0}/np.arcsin(10.9756)))` |

### `I.10.7`  — 正しい数式: `m_0/sqrt(1-v**2/c**2)` （変数: m_0, v, c）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.852e-01 | `v_{0}` | `v_{0}` |
| 3 | 1.410e-01 | `Plus(I, v_{0})` | `(0.0929)+(v_{0})` |
| 4 | 1.128e-01 | `Plus(v_{0}, Inv(v_{2}))` | `(v_{0})+((1/v_{2}))` |
| 5 | 8.658e-02 | `Divide(v_{0}, ArcTan(ArcTan(v_{2})))` | `(v_{0}/np.arctan(np.arctan(v_{2})))` |

### `I.11.19`  — 正しい数式: `x1*y1+x2*y2+x3*y3` （変数: x1, x2, x3, y1, y2, y3）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 2.558e+01 | `v_{2}` | `v_{2}` |
| 2 | 9.847e+00 | `Exp(Pi)` | `np.exp(np.pi)` |
| 4 | 8.589e+00 | `Plus(v_{4}, Exp(Pi))` | `(v_{4})+(np.exp(np.pi))` |
| 5 | 8.170e+00 | `Times(ArcTan(v_{4}), Exp(Pi))` | `(np.arctan(v_{4}))*(np.exp(np.pi))` |
| 6 | 7.507e+00 | `Plus(Exp(E), Times(v_{1}, v_{4}))` | `(np.exp(np.e))+((v_{1})*(v_{4}))` |

### `I.12.2`  — 正しい数式: `q1*q2*r/(4*pi*epsilon*r**3)` （変数: q1, q2, epsilon, r）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.014e-01 | `0` | `0` |
| 2 | 8.545e-02 | `Sqrt(I)` | `np.sqrt(0.0041)` |
| 3 | 7.779e-02 | `Divide(I, v_{3})` | `(0.2492/v_{3})` |
| 4 | 6.367e-02 | `Exp(Subtract(I, v_{3}))` | `np.exp((-0.4983)-(v_{3}))` |
| 5 | 5.852e-02 | `Divide(Inv(v_{2}), Exp(v_{3}))` | `((1/v_{2})/np.exp(v_{3}))` |
| 6 | 4.236e-02 | `Divide(Inv(v_{2}), EML(v_{3}, v_{0}))` | `((1/v_{2})/(np.exp(v_{3}) - np.log(v_{0})))` |

### `I.12.4`  — 正しい数式: `q1*r/(4*pi*epsilon*r**3)` （変数: q1, epsilon, r）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 3.178e-02 | `0` | `0` |
| 2 | 2.579e-02 | `ArcSin(I)` | `np.arcsin(0.0144)` |
| 3 | 2.338e-02 | `Divide(I, v_{2})` | `(0.0856/v_{2})` |
| 4 | 1.886e-02 | `Divide(I, Exp(v_{2}))` | `(0.2628/np.exp(v_{2}))` |
| 5 | 1.706e-02 | `Divide(Divide(I, v_{2}), v_{2})` | `((0.0856/v_{2})/v_{2})` |
| 6 | 1.549e-02 | `Divide(Tan(E), EML(v_{2}, v_{0}))` | `(np.tan(42380.2511)/(np.exp(v_{2}) - np.log(v_{0})))` |

### `I.12.11`  — 正しい数式: `q*(Ef+B*v*sin(theta))` （変数: q, Ef, B, v, theta）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 2.723e+01 | `v_{0}` | `v_{0}` |
| 2 | 2.619e+01 | `Exp(E)` | `np.exp(np.e)` |
| 3 | 2.565e+01 | `Times(v_{0}, v_{1})` | `(v_{0})*(v_{1})` |
| 4 | 2.322e+01 | `Divide(Exp(Pi), v_{4})` | `(np.exp(np.pi)/v_{4})` |
| 5 | 1.881e+01 | `Times(Exp(Pi), Sin(v_{4}))` | `(np.exp(np.pi))*(np.sin(v_{4}))` |
| 6 | 1.816e+01 | `Times(v_{0}, Cos(Log(Neg(v_{4}))))` | `(v_{0})*(np.cos(np.log((-v_{4}))))` |

### `I.13.4`  — 正しい数式: `1/2*m*(v**2+u**2+w**2)` （変数: m, v, u, w）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 5.062e+01 | `v_{0}` | `v_{0}` |
| 2 | 3.251e+01 | `Exp(v_{0})` | `np.exp(v_{0})` |
| 3 | 2.864e+01 | `Tan(Cos(I))` | `np.tan(np.cos(1j))` |
| 4 | 2.012e+01 | `Times(v_{0}, Exp(E))` | `(v_{0})*(np.exp(np.e))` |
| 6 | 1.937e+01 | `Times(v_{0}, EML(E, Inv(v_{3})))` | `(v_{0})*((np.exp(np.e) - np.log((1/v_{3}))))` |

### `I.13.12`  — 正しい数式: `G*m1*m2*(1/r2-1/r1)` （変数: m1, m2, r1, r2, G）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 9.613e+00 | `v_{2}` | `v_{2}` |
| 2 | 9.367e+00 | `Sin(v_{3})` | `np.sin(v_{3})` |
| 3 | 8.589e+00 | `Subtract(v_{2}, v_{3})` | `(v_{2})-(v_{3})` |
| 5 | 6.314e+00 | `Times(v_{1}, Subtract(v_{2}, v_{3}))` | `(v_{1})*((v_{2})-(v_{3}))` |

### `I.14.4`  — 正しい数式: `1/2*k_spring*x**2` （変数: k_spring, x）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.746e+01 | `v_{1}` | `v_{1}` |
| 2 | 1.274e+01 | `Exp(E)` | `np.exp(np.e)` |
| 3 | 9.559e+00 | `Times(v_{1}, v_{1})` | `(v_{1})*(v_{1})` |
| 4 | 9.100e+00 | `Divide(Exp(v_{1}), Pi)` | `(np.exp(v_{1})/np.pi)` |
| 5 | 2.131e+00 | `Times(v_{0}, Sqrt(Exp(v_{1})))` | `(v_{0})*(np.sqrt(np.exp(v_{1})))` |
| 6 | 1.590e+00 | `Times(v_{0}, Sqrt(EML(v_{1}, Pi)))` | `(v_{0})*(np.sqrt((np.exp(v_{1}) - np.log(13.9731))))` |

### `I.15.3x`  — 正しい数式: `(x-u*t)/sqrt(1-u**2/c**2)` （変数: x, u, c, t）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 2.218e+00 | `v_{0}` | `v_{0}` |
| 3 | 6.139e-01 | `Subtract(v_{0}, E)` | `(v_{0})-(2.2815)` |
| 4 | 6.032e-01 | `Subtract(v_{0}, Tan(1))` | `(v_{0})-(np.tan(1.1486))` |
| 5 | 2.101e-01 | `Subtract(v_{0}, Times(v_{1}, v_{3}))` | `(v_{0})-((v_{1})*(v_{3}))` |

### `I.15.3t`  — 正しい数式: `(t-u*x/c**2)/sqrt(1-u**2/c**2)` （変数: x, c, u, t）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.102e-01 | `v_{3}` | `v_{3}` |
| 3 | 1.078e-01 | `Plus(I, v_{3})` | `(-0.0178)+(v_{3})` |
| 4 | 1.053e-01 | `Log(EML(v_{3}, v_{2}))` | `np.log((np.exp(v_{3}) - np.log(v_{2})))` |
| 5 | 1.013e-01 | `Plus(v_{3}, Times(I, v_{0}))` | `(v_{3})+((-0.0107)*(v_{0}))` |
| 6 | 1.011e-01 | `EML(Log(v_{3}), ArcTan(Sqrt(v_{0})))` | `(np.exp(np.log(v_{3})) - np.log(np.arctan(np.sqrt(v_...` |

### `I.15.1`  — 正しい数式: `m_0*v/sqrt(1-v**2/c**2)` （変数: m_0, v, c）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 2.128e+00 | `v_{0}` | `v_{0}` |
| 2 | 1.865e+00 | `Exp(v_{1})` | `np.exp(v_{1})` |
| 3 | 3.448e-01 | `Times(v_{0}, v_{1})` | `(v_{0})*(v_{1})` |
| 5 | 2.649e-01 | `Times(v_{0}, Plus(I, v_{1}))` | `(v_{0})*((0.0549)+(v_{1}))` |
| 6 | 1.952e-01 | `Times(v_{1}, Plus(v_{0}, Inv(v_{2})))` | `(v_{1})*((v_{0})+((1/v_{2})))` |

### `I.16.6`  — 正しい数式: `(u+v)/(1+u*v/c**2)` （変数: c, v, u）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 4.098e-01 | `v_{0}` | `v_{0}` |
| 3 | 3.762e-01 | `Plus(I, v_{0})` | `(-0.1259)+(v_{0})` |
| 4 | 3.692e-01 | `Divide(v_{0}, ArcSin(1))` | `(v_{0}/np.arcsin(0.8775))` |
| 5 | 3.590e-01 | `Times(v_{0}, Sin(ArcTan(v_{1})))` | `(v_{0})*(np.sin(np.arctan(v_{1})))` |

### `I.18.4`  — 正しい数式: `(m1*r1+m2*r2)/(m1+m2)` （変数: m1, m2, r1, r2）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 8.334e-01 | `v_{3}` | `v_{3}` |
| 3 | 7.116e-01 | `Exp(ArcTan(v_{3}))` | `np.exp(np.arctan(v_{3}))` |
| 4 | 3.097e-01 | `Sqrt(Times(v_{2}, v_{3}))` | `np.sqrt((v_{2})*(v_{3}))` |
| 6 | 2.667e-01 | `Times(ArcTan(1), Plus(v_{2}, v_{3}))` | `(np.arctan(0.5802))*((v_{2})+(v_{3}))` |

### `I.24.6`  — 正しい数式: `1/2*m*(omega**2+omega_0**2)*1/2*x**2` （変数: m, omega, omega_0, x）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 2.092e+01 | `Pi` | `np.pi` |
| 2 | 1.446e+01 | `Exp(E)` | `np.exp(np.e)` |
| 3 | 1.442e+01 | `EML(Pi, Pi)` | `(np.exp(np.pi) - np.log(np.pi))` |
| 4 | 8.562e+00 | `Times(v_{0}, Exp(v_{3}))` | `(v_{0})*(np.exp(v_{3}))` |
| 6 | 7.933e+00 | `Times(v_{0}, EML(v_{3}, Tan(v_{2})))` | `(v_{0})*((np.exp(v_{3}) - np.log(np.tan(v_{2}))))` |

### `I.27.6`  — 正しい数式: `1/(1/d1+n/d2)` （変数: d1, d2, n）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 4.319e-01 | `1` | `1` |
| 2 | 3.568e-01 | `Sqrt(I)` | `np.sqrt(0.5675)` |
| 3 | 2.836e-01 | `Divide(v_{1}, E)` | `(v_{1}/4.0078)` |
| 4 | 1.682e-01 | `ArcTan(Divide(v_{1}, v_{2}))` | `np.arctan((v_{1}/v_{2}))` |
| 5 | 1.663e-01 | `Divide(v_{1}, Plus(1, v_{2}))` | `(v_{1}/(1.2236)+(v_{2}))` |
| 6 | 1.557e-01 | `Plus(I, Sqrt(Divide(v_{1}, v_{2})))` | `(-0.2091)+(np.sqrt((v_{1}/v_{2})))` |

### `I.29.16`  — 正しい数式: `sqrt(x1**2+x2**2-2*x1*x2*cos(theta1-theta2))` （変数: x1, x2, theta1, theta2）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.834e+00 | `Pi` | `np.pi` |
| 3 | 1.735e+00 | `Exp(ArcTan(v_{1}))` | `np.exp(np.arctan(v_{1}))` |
| 4 | 1.719e+00 | `Exp(Sqrt(Sqrt(v_{1})))` | `np.exp(np.sqrt(np.sqrt(v_{1})))` |
| 5 | 1.671e+00 | `Plus(Sqrt(v_{0}), Sqrt(v_{1}))` | `(np.sqrt(v_{0}))+(np.sqrt(v_{1}))` |
| 6 | 1.662e+00 | `Divide(Plus(v_{0}, v_{1}), Sqrt(E))` | `((v_{0})+(v_{1})/np.sqrt(2.7811))` |

### `I.30.3`  — 正しい数式: `Int_0*sin(n*theta/2)**2/sin(theta/2)**2` （変数: Int_0, theta, n）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 2.409e+00 | `v_{0}` | `v_{0}` |
| 3 | 2.390e+00 | `Subtract(v_{0}, 1)` | `(v_{0})-(1)` |
| 4 | 2.170e+00 | `Divide(v_{0}, ArcTan(v_{1}))` | `(v_{0}/np.arctan(v_{1}))` |
| 5 | 1.944e+00 | `Times(v_{0}, Exp(Cos(v_{1})))` | `(v_{0})*(np.exp(np.cos(v_{1})))` |

### `I.32.5`  — 正しい数式: `q**2*a**2/(6*pi*epsilon*c**3)` （変数: q, a, epsilon, c）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 8.832e-01 | `0` | `0` |
| 2 | 7.817e-01 | `Inv(v_{3})` | `(1/v_{3})` |
| 3 | 7.704e-01 | `EML(0, v_{3})` | `(np.exp(0.2021) - np.log(v_{3}))` |
| 4 | 6.926e-01 | `Divide(v_{1}, Exp(v_{3}))` | `(v_{1}/np.exp(v_{3}))` |
| 5 | 5.779e-01 | `Tan(Divide(Log(v_{0}), v_{3}))` | `np.tan((np.log(v_{0})/v_{3}))` |

### `I.32.17`  — 正しい数式: `(1/2*epsilon*c*Ef**2)*(8*pi*r**2/3)*(omega**4/(omega**2-omega_0**2)**2)` （変数: epsilon, c, Ef, r, omega, omega_0）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 5.628e+00 | `Pi` | `np.pi` |
| 2 | 5.369e+00 | `Exp(v_{4})` | `np.exp(v_{4})` |
| 3 | 5.052e+00 | `EML(v_{4}, v_{5})` | `(np.exp(v_{4}) - np.log(v_{5}))` |
| 4 | 4.907e+00 | `EML(v_{4}, Tan(v_{5}))` | `(np.exp(v_{4}) - np.log(np.tan(v_{5})))` |
| 6 | 4.520e+00 | `Divide(Exp(v_{4}), Subtract(v_{5}, E))` | `(np.exp(v_{4})/(v_{5})-(np.e))` |

### `I.34.8`  — 正しい数式: `q*v*B/p` （変数: q, v, B, p）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.258e+01 | `v_{1}` | `v_{1}` |
| 2 | 1.105e+01 | `Exp(E)` | `np.exp(np.e)` |
| 3 | 8.409e+00 | `Times(v_{0}, v_{1})` | `(v_{0})*(v_{1})` |
| 5 | 7.795e+00 | `Plus(v_{0}, Times(v_{1}, v_{2}))` | `(v_{0})+((v_{1})*(v_{2}))` |
| 6 | 7.028e+00 | `Times(v_{1}, Times(v_{0}, Log(v_{2})))` | `(v_{1})*((v_{0})*(np.log(v_{2})))` |

### `I.34.1`  — 正しい数式: `omega_0/(1-v/c)` （変数: c, v, omega_0）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.477e+00 | `v_{2}` | `v_{2}` |
| 3 | 8.915e-01 | `Plus(v_{1}, v_{2})` | `(v_{1})+(v_{2})` |
| 4 | 6.940e-01 | `EML(Sqrt(v_{2}), v_{0})` | `(np.exp(np.sqrt(v_{2})) - np.log(v_{0}))` |
| 6 | 4.994e-01 | `Divide(Times(Pi, v_{2}), Sqrt(v_{0}))` | `((3.4315)*(v_{2})/np.sqrt(v_{0}))` |

### `I.34.14`  — 正しい数式: `(1+v/c)/sqrt(1-v**2/c**2)*omega_0` （変数: c, v, omega_0）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.130e+00 | `v_{2}` | `v_{2}` |
| 3 | 6.172e-01 | `Plus(1, v_{2})` | `(0.9602)+(v_{2})` |
| 4 | 5.006e-01 | `Times(v_{2}, ArcTan(E))` | `(v_{2})*(np.arctan(3.6290))` |
| 5 | 4.590e-01 | `Times(v_{2}, ArcTan(Exp(v_{1})))` | `(v_{2})*(np.arctan(np.exp(v_{1})))` |
| 6 | 3.916e-01 | `Divide(Times(Pi, v_{2}), Sqrt(v_{0}))` | `((3.1805)*(v_{2})/np.sqrt(v_{0}))` |

### `I.34.27`  — 正しい数式: `(h/(2*pi))*omega` （変数: omega, h）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 9.142e-01 | `1` | `1` |
| 2 | 6.687e-01 | `Sqrt(v_{0})` | `np.sqrt(v_{0})` |
| 3 | 6.137e-01 | `Divide(v_{0}, Pi)` | `(v_{0}/1.8632)` |
| 4 | 5.918e-01 | `Inv(Divide(Pi, v_{0}))` | `(1/(2.1027/v_{0}))` |
| 5 | 1.866e-01 | `Times(v_{0}, Log(Sqrt(v_{1})))` | `(v_{0})*(np.log(np.sqrt(v_{1})))` |
| 6 | 1.340e-01 | `Divide(Log(v_{0}), Divide(Pi, v_{1}))` | `(np.log(v_{0})/(2.1675/v_{1}))` |

### `I.37.4`  — 正しい数式: `I1+I2+2*sqrt(I1*I2)*cos(delta)` （変数: I1, I2, delta）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 2.936e+00 | `Pi` | `np.pi` |
| 3 | 2.849e+00 | `Exp(ArcTan(v_{1}))` | `np.exp(np.arctan(v_{1}))` |
| 4 | 2.424e+00 | `EML(1, Cos(v_{2}))` | `(np.exp(1) - np.log(np.cos(v_{2})))` |
| 5 | 2.133e+00 | `Plus(v_{0}, Log(Tan(v_{2})))` | `(v_{0})+(np.log(np.tan(v_{2})))` |
| 6 | 1.751e+00 | `EML(Exp(Cos(v_{2})), Cos(v_{2}))` | `(np.exp(np.exp(np.cos(v_{2}))) - np.log(np.cos(v_{2})))` |

### `I.39.1`  — 正しい数式: `3/2*pr*V` （変数: pr, V）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.204e+01 | `v_{1}` | `v_{1}` |
| 2 | 7.617e+00 | `Exp(E)` | `np.exp(np.e)` |
| 3 | 4.985e+00 | `Times(v_{0}, v_{1})` | `(v_{0})*(v_{1})` |
| 5 | 2.341e+00 | `Plus(v_{1}, Times(v_{0}, v_{1}))` | `(v_{1})+((v_{0})*(v_{1}))` |
| 6 | 8.287e-02 | `Times(v_{1}, Times(v_{0}, Tan(1)))` | `(v_{1})*((v_{0})*(np.tan(0.9853)))` |

### `I.39.11`  — 正しい数式: `1/(gamma-1)*pr*V` （変数: gamma, pr, V）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 2.726e+00 | `v_{2}` | `v_{2}` |
| 3 | 2.487e+00 | `Plus(1, v_{2})` | `(1)+(v_{2})` |
| 4 | 2.232e+00 | `Times(v_{2}, Sqrt(v_{1}))` | `(v_{2})*(np.sqrt(v_{1}))` |
| 5 | 1.911e+00 | `Times(v_{2}, Divide(v_{1}, v_{0}))` | `(v_{2})*((v_{1}/v_{0}))` |
| 6 | 1.547e+00 | `Divide(Times(v_{1}, v_{2}), Sqrt(v_{0}))` | `((v_{1})*(v_{2})/np.sqrt(v_{0}))` |

### `I.39.22`  — 正しい数式: `n*kb*T/V` （変数: n, T, V, kb）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.247e+01 | `v_{0}` | `v_{0}` |
| 2 | 1.120e+01 | `Exp(E)` | `np.exp(np.e)` |
| 3 | 8.345e+00 | `Times(v_{0}, v_{3})` | `(v_{0})*(v_{3})` |
| 5 | 7.772e+00 | `Plus(v_{1}, Times(v_{0}, v_{3}))` | `(v_{1})+((v_{0})*(v_{3}))` |
| 6 | 6.956e+00 | `Times(v_{3}, Times(v_{0}, Log(v_{1})))` | `(v_{3})*((v_{0})*(np.log(v_{1})))` |

### `I.40.1`  — 正しい数式: `n_0*exp(-m*g*x/(kb*T))` （変数: n_0, m, x, T, g, kb）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 7.960e-01 | `0` | `0` |
| 2 | 6.063e-01 | `Inv(v_{1})` | `(1/v_{1})` |
| 3 | 6.053e-01 | `ArcSin(Inv(v_{4}))` | `np.arcsin((1/v_{4}))` |
| 4 | 5.651e-01 | `Divide(Log(v_{3}), v_{1})` | `(np.log(v_{3})/v_{1})` |
| 5 | 4.804e-01 | `Divide(Divide(v_{0}, v_{4}), v_{1})` | `((v_{0}/v_{4})/v_{1})` |

### `I.41.16`  — 正しい数式: `h/(2*pi)*omega**3/(pi**2*c**2*(exp((h/(2*pi))*omega/(kb*T))-1))` （変数: omega, T, h, kb, c）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 3.091e+00 | `v_{0}` | `v_{0}` |
| 2 | 2.941e+00 | `Sqrt(v_{0})` | `np.sqrt(v_{0})` |
| 3 | 2.555e+00 | `Divide(v_{0}, v_{4})` | `(v_{0}/v_{4})` |
| 4 | 2.424e+00 | `Exp(Subtract(Pi, v_{4}))` | `np.exp((np.pi)-(v_{4}))` |
| 5 | 2.121e+00 | `Divide(Sqrt(Exp(v_{0})), v_{4})` | `(np.sqrt(np.exp(v_{0}))/v_{4})` |
| 6 | 1.896e+00 | `Times(v_{0}, Tan(Sqrt(ArcSin(v_{4}))))` | `(v_{0})*(np.tan(np.sqrt(np.arcsin(v_{4}))))` |

### `I.43.16`  — 正しい数式: `mu_drift*q*Volt/d` （変数: mu_drift, q, Volt, d）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.339e+01 | `v_{2}` | `v_{2}` |
| 2 | 1.160e+01 | `Exp(E)` | `np.exp(np.e)` |
| 3 | 9.132e+00 | `Times(v_{0}, v_{1})` | `(v_{0})*(v_{1})` |
| 5 | 8.347e+00 | `Plus(v_{2}, Times(v_{0}, v_{1}))` | `(v_{2})+((v_{0})*(v_{1}))` |
| 6 | 7.492e+00 | `Times(v_{2}, Times(v_{0}, Log(v_{1})))` | `(v_{2})*((v_{0})*(np.log(v_{1})))` |

### `I.43.43`  — 正しい数式: `1/(gamma-1)*kb*v/A` （変数: gamma, kb, A, v）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.690e+00 | `1` | `1` |
| 2 | 1.435e+00 | `Sqrt(v_{3})` | `np.sqrt(v_{3})` |
| 3 | 1.250e+00 | `Divide(v_{3}, v_{2})` | `(v_{3}/v_{2})` |
| 4 | 1.205e+00 | `Divide(v_{3}, Sqrt(v_{2}))` | `(v_{3}/np.sqrt(v_{2}))` |
| 5 | 1.153e+00 | `Divide(Plus(v_{1}, v_{3}), v_{0})` | `((v_{1})+(v_{3})/v_{0})` |
| 6 | 1.008e+00 | `Times(Log(v_{1}), Divide(v_{3}, v_{2}))` | `(np.log(v_{1}))*((v_{3}/v_{2}))` |

### `I.44.4`  — 正しい数式: `n*kb*T*ln(V2/V1)` （変数: n, kb, T, V1, V2）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 2.037e+01 | `v_{4}` | `v_{4}` |
| 3 | 1.963e+01 | `Subtract(v_{4}, v_{3})` | `(v_{4})-(v_{3})` |
| 4 | 1.738e+01 | `Exp(ArcCos(Neg(v_{3})))` | `np.exp(np.arccos((-v_{3})))` |
| 5 | 1.657e+01 | `Times(v_{1}, Subtract(v_{4}, v_{3}))` | `(v_{1})*((v_{4})-(v_{3}))` |
| 6 | 1.475e+01 | `Times(Exp(E), Subtract(v_{4}, v_{3}))` | `(np.exp(np.e))*((v_{4})-(v_{3}))` |

### `I.47.23`  — 正しい数式: `sqrt(gamma*pr/rho)` （変数: gamma, pr, rho）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.029e+00 | `1` | `1` |
| 2 | 5.834e-01 | `Sqrt(v_{0})` | `np.sqrt(v_{0})` |
| 3 | 5.446e-01 | `EML(1, v_{2})` | `(np.exp(1.0148) - np.log(v_{2}))` |
| 4 | 5.080e-01 | `Log(Plus(v_{0}, v_{1}))` | `np.log((v_{0})+(v_{1}))` |
| 5 | 4.032e-01 | `EML(0, Divide(v_{2}, v_{1}))` | `(np.exp(0.5957) - np.log((v_{2}/v_{1})))` |
| 6 | 2.802e-01 | `Divide(Plus(v_{0}, v_{1}), ArcSin(v_{2}))` | `((v_{0})+(v_{1})/np.arcsin(v_{2}))` |

### `I.48.2`  — 正しい数式: `m*c**2/sqrt(1-v**2/c**2)` （変数: m, v, c）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.622e+02 | `v_{2}` | `v_{2}` |
| 2 | 1.332e+02 | `Exp(v_{0})` | `np.exp(v_{0})` |
| 3 | 1.173e+02 | `Tan(Tan(1))` | `np.tan(np.tan(1))` |
| 4 | 7.110e+01 | `Times(v_{2}, Exp(Pi))` | `(v_{2})*(np.exp(np.pi))` |
| 5 | 4.399e+00 | `Times(v_{0}, Times(v_{2}, v_{2}))` | `(v_{0})*((v_{2})*(v_{2}))` |

### `I.50.26`  — 正しい数式: `x1*(cos(omega*t)+alpha*cos(omega*t)**2)` （変数: x1, omega, t, alpha）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.928e+00 | `v_{3}` | `v_{3}` |
| 3 | 1.890e+00 | `Plus(I, v_{3})` | `(-0.2943)+(v_{3})` |
| 4 | 1.833e+00 | `Times(v_{2}, Log(v_{3}))` | `(v_{2})*(np.log(v_{3}))` |
| 5 | 1.791e+00 | `Subtract(v_{1}, Subtract(E, v_{3}))` | `(v_{1})-((2.4837)-(v_{3}))` |
| 6 | 1.790e+00 | `Subtract(Plus(v_{2}, v_{3}), Tan(1))` | `((v_{2})+(v_{3}))-(np.tan(214.7923))` |

### `II.2.42`  — 正しい数式: `kappa*(T2-T1)*A/d` （変数: kappa, T1, T2, A, d）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 7.209e+00 | `v_{2}` | `v_{2}` |
| 2 | 6.920e+00 | `Sin(v_{1})` | `np.sin(v_{1})` |
| 3 | 6.087e+00 | `Subtract(v_{2}, v_{1})` | `(v_{2})-(v_{1})` |
| 5 | 3.924e+00 | `Times(v_{3}, Subtract(v_{2}, v_{1}))` | `(v_{3})*((v_{2})-(v_{1}))` |

### `II.3.24`  — 正しい数式: `Pwr/(4*pi*r**2)` （変数: Pwr, r）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 7.130e-02 | `0` | `0` |
| 2 | 5.576e-02 | `Tan(I)` | `np.tan(0.0344)` |
| 3 | 4.497e-02 | `Divide(I, v_{1})` | `(0.2045/v_{1})` |
| 4 | 2.716e-02 | `Exp(Subtract(I, v_{1}))` | `np.exp((1j)-(v_{1}))` |
| 5 | 2.699e-02 | `Tan(Exp(Subtract(I, v_{1})))` | `np.tan(np.exp((-0.6550)-(v_{1})))` |
| 6 | 1.092e-02 | `Divide(Log(Sqrt(v_{0})), Exp(v_{1}))` | `(np.log(np.sqrt(v_{0}))/np.exp(v_{1}))` |

### `II.4.23`  — 正しい数式: `q/(4*pi*epsilon*r)` （変数: q, epsilon, r）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 4.767e-02 | `0` | `0` |
| 2 | 3.204e-02 | `Tan(I)` | `np.tan(0.0273)` |
| 3 | 2.846e-02 | `Times(I, v_{0})` | `(0.0094)*(v_{0})` |
| 4 | 2.632e-02 | `Divide(Cos(1), v_{1})` | `(np.cos(3180.7689)/v_{1})` |
| 5 | 2.595e-02 | `Divide(Inv(v_{1}), Exp(v_{2}))` | `((1/v_{1})/np.exp(v_{2}))` |
| 6 | 1.985e-02 | `Divide(Inv(v_{2}), Times(Pi, v_{1}))` | `((1/v_{2})/(5.2707)*(v_{1}))` |

### `II.6.11`  — 正しい数式: `1/(4*pi*epsilon)*p_d*cos(theta)/r**2` （変数: epsilon, p_d, theta, r）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 2.294e-02 | `0` | `0` |
| 2 | 2.065e-02 | `Tan(I)` | `np.tan(-0.0078)` |
| 3 | 2.048e-02 | `Divide(I, E)` | `(-0.0285/2.7183)` |
| 4 | 1.742e-02 | `Times(I, Exp(v_{2}))` | `(-0.0012)*(np.exp(v_{2}))` |
| 5 | 1.517e-02 | `Divide(Cos(v_{2}), ArcSin(E))` | `(np.cos(v_{2})/np.arcsin(568.5703))` |
| 6 | 1.516e-02 | `Tan(Divide(Cos(v_{2}), ArcSin(E)))` | `np.tan((np.cos(v_{2})/np.arcsin(568.5703)))` |

### `II.6.15a`  — 正しい数式: `p_d/(4*pi*epsilon)*3*z/r**5*sqrt(x**2+y**2)` （変数: epsilon, p_d, r, x, y, z）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 4.025e-01 | `0` | `0` |
| 2 | 3.583e-01 | `Tan(E)` | `np.tan(575.1066)` |
| 3 | 2.491e-01 | `Sqrt(Cos(v_{2}))` | `np.sqrt(np.cos(v_{2}))` |
| 4 | 2.318e-01 | `Tan(Exp(ArcCos(v_{2})))` | `np.tan(np.exp(np.arccos(v_{2})))` |
| 6 | 1.897e-01 | `Divide(Inv(v_{0}), Log(Tan(v_{2})))` | `((1/v_{0})/np.log(np.tan(v_{2})))` |

### `II.6.15b`  — 正しい数式: `p_d/(4*pi*epsilon)*3*cos(theta)*sin(theta)/r**3` （変数: epsilon, p_d, theta, r）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 3.319e-02 | `0` | `0` |
| 2 | 3.158e-02 | `Tan(I)` | `np.tan(-0.0079)` |
| 3 | 3.147e-02 | `Divide(I, E)` | `(-0.0291/2.7183)` |
| 4 | 3.007e-02 | `Times(I, Exp(v_{2}))` | `(-0.0011)*(np.exp(v_{2}))` |
| 5 | 2.740e-02 | `Divide(Cos(v_{2}), ArcSin(E))` | `(np.cos(v_{2})/np.arcsin(545.9369))` |
| 6 | 2.526e-02 | `Divide(Cos(v_{2}), Exp(Exp(v_{3})))` | `(np.cos(v_{2})/np.exp(np.exp(v_{3})))` |

### `II.8.7`  — 正しい数式: `3/5*q**2/(4*pi*epsilon*d)` （変数: q, epsilon, d）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.281e-01 | `0` | `0` |
| 2 | 9.956e-02 | `Sqrt(I)` | `np.sqrt(0.0060)` |
| 3 | 8.785e-02 | `Times(I, v_{0})` | `(0.0231)*(v_{0})` |
| 4 | 8.562e-02 | `Divide(v_{0}, Exp(Pi))` | `(v_{0}/np.exp(3.5993))` |
| 5 | 7.053e-02 | `Divide(Log(ArcTan(v_{0})), v_{2})` | `(np.log(np.arctan(v_{0}))/v_{2})` |
| 6 | 6.515e-02 | `Times(v_{0}, Divide(Sqrt(I), v_{2}))` | `(v_{0})*((np.sqrt(0.0060)/v_{2}))` |

### `II.8.31`  — 正しい数式: `epsilon*Ef**2/2` （変数: epsilon, Ef）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.750e+01 | `v_{1}` | `v_{1}` |
| 2 | 1.281e+01 | `Exp(E)` | `np.exp(np.e)` |
| 3 | 9.818e+00 | `Times(v_{1}, v_{1})` | `(v_{1})*(v_{1})` |
| 4 | 8.619e+00 | `Exp(Subtract(v_{1}, 1))` | `np.exp((v_{1})-(1))` |
| 5 | 2.037e+00 | `Times(v_{0}, Sqrt(Exp(v_{1})))` | `(v_{0})*(np.sqrt(np.exp(v_{1})))` |
| 6 | 1.621e+00 | `Times(v_{0}, Sqrt(EML(v_{1}, Pi)))` | `(v_{0})*(np.sqrt((np.exp(v_{1}) - np.log(11.4920))))` |

### `II.10.9`  — 正しい数式: `sigma_den/epsilon*1/(1+chi)` （変数: sigma_den, epsilon, chi）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 4.198e-01 | `0` | `0` |
| 2 | 2.055e-01 | `Inv(v_{1})` | `(1/v_{1})` |
| 3 | 1.950e-01 | `Cos(ArcTan(v_{1}))` | `np.cos(np.arctan(v_{1}))` |
| 4 | 1.652e-01 | `Divide(Log(v_{0}), v_{1})` | `(np.log(v_{0})/v_{1})` |
| 5 | 1.311e-01 | `Divide(v_{0}, Times(Pi, v_{1}))` | `(v_{0}/(3.8189)*(v_{1}))` |
| 6 | 7.008e-02 | `Divide(Divide(v_{0}, v_{1}), ArcSin(v_{2}))` | `((v_{0}/v_{1})/np.arcsin(v_{2}))` |

### `II.11.3`  — 正しい数式: `q*Ef/(m*(omega_0**2-omega**2))` （変数: q, Ef, m, omega_0, omega）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 2.223e-01 | `0` | `0` |
| 2 | 1.335e-01 | `Inv(v_{3})` | `(1/v_{3})` |
| 3 | 1.143e-01 | `Exp(Neg(v_{2}))` | `np.exp((-v_{2}))` |
| 4 | 1.057e-01 | `Divide(Log(v_{0}), v_{3})` | `(np.log(v_{0})/v_{3})` |
| 5 | 9.645e-02 | `Divide(Tan(Inv(v_{2})), v_{3})` | `(np.tan((1/v_{2}))/v_{3})` |
| 6 | 7.792e-02 | `Divide(Divide(v_{0}, v_{3}), ArcSin(v_{2}))` | `((v_{0}/v_{3})/np.arcsin(v_{2}))` |

### `II.11.17`  — 正しい数式: `n_0*(1+p_d*Ef*cos(theta)/(kb*T))` （変数: n_0, kb, T, theta, p_d, Ef）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.649e+00 | `1` | `1` |
| 2 | 1.577e+00 | `Sin(v_{3})` | `np.sin(v_{3})` |
| 3 | 1.245e+00 | `Exp(ArcSin(v_{3}))` | `np.exp(np.arcsin(v_{3}))` |
| 4 | 1.208e+00 | `Exp(Subtract(v_{0}, v_{3}))` | `np.exp((v_{0})-(v_{3}))` |
| 5 | 1.128e+00 | `Times(v_{0}, ArcCos(Log(v_{3})))` | `(v_{0})*(np.arccos(np.log(v_{3})))` |
| 6 | 1.042e+00 | `Times(v_{0}, ArcCos(Tan(Log(v_{3}))))` | `(v_{0})*(np.arccos(np.tan(np.log(v_{3}))))` |

### `II.11.20`  — 正しい数式: `n_rho*p_d**2*Ef/(3*kb*T)` （変数: n_rho, p_d, Ef, kb, T）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 6.330e+00 | `v_{1}` | `v_{1}` |
| 3 | 5.641e+00 | `Sqrt(Exp(v_{1}))` | `np.sqrt(np.exp(v_{1}))` |
| 4 | 5.391e+00 | `Times(v_{1}, Sqrt(v_{0}))` | `(v_{1})*(np.sqrt(v_{0}))` |
| 5 | 4.932e+00 | `Times(v_{0}, Divide(v_{1}, v_{4}))` | `(v_{0})*((v_{1}/v_{4}))` |
| 6 | 4.623e+00 | `Times(v_{0}, Divide(v_{1}, Sqrt(v_{4})))` | `(v_{0})*((v_{1}/np.sqrt(v_{4})))` |

### `II.11.27`  — 正しい数式: `n*alpha/(1-(n*alpha/3))*epsilon*Ef` （変数: n, alpha, epsilon, Ef）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 5.792e-01 | `v_{0}` | `v_{0}` |
| 2 | 5.475e-01 | `Tan(v_{0})` | `np.tan(v_{0})` |
| 3 | 5.206e-01 | `Plus(v_{0}, v_{1})` | `(v_{0})+(v_{1})` |
| 4 | 3.777e-01 | `Times(v_{0}, Exp(v_{1}))` | `(v_{0})*(np.exp(v_{1}))` |
| 5 | 2.445e-01 | `Times(E, Times(v_{0}, v_{1}))` | `(2.7822)*((v_{0})*(v_{1}))` |
| 6 | 2.364e-01 | `Times(E, ArcSin(Times(v_{0}, v_{1})))` | `(2.5998)*(np.arcsin((v_{0})*(v_{1})))` |

### `II.11.28`  — 正しい数式: `1+n*alpha/(1-(n*alpha/3))` （変数: n, alpha）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 4.362e-01 | `1` | `1` |
| 2 | 3.075e-01 | `Tan(1)` | `np.tan(-5.3680)` |
| 3 | 2.267e-01 | `Sqrt(Exp(v_{1}))` | `np.sqrt(np.exp(v_{1}))` |
| 4 | 4.446e-02 | `Exp(Times(v_{0}, v_{1}))` | `np.exp((v_{0})*(v_{1}))` |
| 6 | 3.790e-02 | `Plus(I, Exp(Times(v_{0}, v_{1})))` | `(-0.0180)+(np.exp((v_{0})*(v_{1})))` |

### `II.13.17`  — 正しい数式: `1/(4*pi*epsilon*c**2)*2*I/r` （変数: epsilon, c, I, r）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 3.007e-02 | `0` | `0` |
| 2 | 2.533e-02 | `Tan(I)` | `np.tan(0.0126)` |
| 3 | 2.429e-02 | `Divide(I, v_{1})` | `(0.0764/v_{1})` |
| 4 | 2.122e-02 | `Divide(I, Exp(v_{1}))` | `(0.2382/np.exp(v_{1}))` |
| 5 | 1.890e-02 | `Divide(Divide(I, v_{1}), v_{3})` | `((0.0764/v_{1})/v_{3})` |
| 6 | 1.804e-02 | `Divide(Divide(I, v_{1}), Sqrt(v_{3}))` | `((0.0764/v_{1})/np.sqrt(v_{3}))` |

### `II.13.23`  — 正しい数式: `rho_c_0/sqrt(1-v**2/c**2)` （変数: rho_c_0, v, c）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 2.133e-01 | `v_{0}` | `v_{0}` |
| 3 | 1.665e-01 | `Plus(I, v_{0})` | `(0.1033)+(v_{0})` |
| 4 | 1.303e-01 | `Plus(v_{0}, Inv(v_{2}))` | `(v_{0})+((1/v_{2}))` |
| 5 | 1.006e-01 | `Divide(v_{0}, ArcTan(ArcTan(v_{2})))` | `(v_{0}/np.arctan(np.arctan(v_{2})))` |

### `II.13.34`  — 正しい数式: `rho_c_0*v/sqrt(1-v**2/c**2)` （変数: rho_c_0, v, c）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 2.225e+00 | `v_{0}` | `v_{0}` |
| 2 | 1.988e+00 | `Exp(v_{1})` | `np.exp(v_{1})` |
| 3 | 3.559e-01 | `Times(v_{0}, v_{1})` | `(v_{0})*(v_{1})` |
| 5 | 2.690e-01 | `Times(v_{0}, Plus(I, v_{1}))` | `(v_{0})*((0.0576)+(v_{1}))` |
| 6 | 2.088e-01 | `Times(v_{1}, Plus(v_{0}, Inv(v_{2})))` | `(v_{1})*((v_{0})+((1/v_{2})))` |

### `II.15.4`  — 正しい数式: `-mom*B*cos(theta)` （変数: mom, B, theta）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 4.844e+00 | `v_{1}` | `v_{1}` |
| 3 | 4.812e+00 | `Plus(1, v_{1})` | `(1)+(v_{1})` |
| 4 | 4.447e+00 | `Subtract(v_{1}, Cos(v_{2}))` | `(v_{1})-(np.cos(v_{2}))` |
| 5 | 3.412e+00 | `Divide(v_{0}, Exp(Cos(v_{2})))` | `(v_{0}/np.exp(np.cos(v_{2})))` |
| 6 | 3.293e+00 | `Divide(Cos(v_{2}), EML(0, Pi))` | `(np.cos(v_{2})/(np.exp(0) - np.log(np.pi)))` |

### `II.15.5`  — 正しい数式: `-p_d*Ef*cos(theta)` （変数: p_d, Ef, theta）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 5.128e+00 | `v_{0}` | `v_{0}` |
| 3 | 4.961e+00 | `Plus(1, v_{0})` | `(1)+(v_{0})` |
| 4 | 4.677e+00 | `Subtract(v_{0}, Cos(v_{2}))` | `(v_{0})-(np.cos(v_{2}))` |
| 5 | 3.419e+00 | `Divide(v_{0}, Exp(Cos(v_{2})))` | `(v_{0}/np.exp(np.cos(v_{2})))` |

### `II.21.32`  — 正しい数式: `q/(4*pi*epsilon*r*(1-v/c))` （変数: q, epsilon, r, v, c）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 7.086e-02 | `0` | `0` |
| 2 | 4.815e-02 | `Cos(1)` | `np.cos(30801.6978)` |
| 3 | 4.482e-02 | `Times(I, v_{0})` | `(0.0135)*(v_{0})` |
| 4 | 4.145e-02 | `Divide(I, ArcSin(v_{1}))` | `(0.1463/np.arcsin(v_{1}))` |
| 5 | 3.576e-02 | `Divide(Inv(v_{2}), Exp(v_{1}))` | `((1/v_{2})/np.exp(v_{1}))` |
| 6 | 2.919e-02 | `Divide(Inv(v_{1}), Times(Pi, v_{2}))` | `((1/v_{1})/(3.1092)*(v_{2}))` |

### `II.24.17`  — 正しい数式: `sqrt(omega**2/c**2-pi**2/d**2)` （変数: omega, c, d）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 8.484e-01 | `Pi` | `np.pi` |
| 3 | 2.178e-01 | `Divide(v_{0}, v_{1})` | `(v_{0}/v_{1})` |
| 5 | 1.096e-01 | `Plus(I, Divide(v_{0}, v_{1}))` | `(-0.1458)+((v_{0}/v_{1}))` |
| 6 | 9.598e-02 | `Subtract(Divide(v_{0}, v_{1}), Inv(v_{0}))` | `((v_{0}/v_{1}))-((1/v_{0}))` |

### `II.27.16`  — 正しい数式: `epsilon*c*Ef**2` （変数: epsilon, c, Ef）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.269e+02 | `v_{2}` | `v_{2}` |
| 2 | 9.076e+01 | `Exp(v_{2})` | `np.exp(v_{2})` |
| 4 | 7.698e+01 | `Times(v_{2}, Exp(Pi))` | `(v_{2})*(np.exp(np.pi))` |
| 5 | 6.559e+01 | `Times(Exp(v_{2}), Sqrt(v_{1}))` | `(np.exp(v_{2}))*(np.sqrt(v_{1}))` |
| 6 | 5.369e+01 | `Times(v_{1}, Exp(Exp(ArcTan(v_{2}))))` | `(v_{1})*(np.exp(np.exp(np.arctan(v_{2}))))` |

### `II.34.2a`  — 正しい数式: `q*v/(2*pi*r)` （変数: q, v, r）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 6.203e-01 | `1` | `1` |
| 2 | 4.051e-01 | `Inv(v_{2})` | `(1/v_{2})` |
| 3 | 3.771e-01 | `Tan(Inv(v_{2}))` | `np.tan((1/v_{2}))` |
| 4 | 2.914e-01 | `Divide(Sqrt(v_{1}), v_{2})` | `(np.sqrt(v_{1})/v_{2})` |
| 5 | 2.544e-01 | `Divide(v_{1}, Plus(v_{2}, v_{2}))` | `(v_{1}/(v_{2})+(v_{2}))` |
| 6 | 2.541e-01 | `Times(Sin(E), Divide(v_{1}, v_{2}))` | `(np.sin(-1147.1993))*((v_{1}/v_{2}))` |

### `II.34.2`  — 正しい数式: `q*v*r/2` （変数: q, v, r）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.352e+01 | `v_{0}` | `v_{0}` |
| 2 | 9.641e+00 | `Exp(E)` | `np.exp(np.e)` |
| 3 | 7.553e+00 | `Times(v_{0}, v_{1})` | `(v_{0})*(v_{1})` |
| 5 | 5.531e+00 | `Plus(v_{2}, Times(v_{0}, v_{1}))` | `(v_{2})+((v_{0})*(v_{1}))` |
| 6 | 2.995e+00 | `Times(v_{2}, Times(v_{1}, Sqrt(v_{0})))` | `(v_{2})*((v_{1})*(np.sqrt(v_{0})))` |

### `II.34.11`  — 正しい数式: `g_*q*B/(2*m)` （変数: g_, q, B, m）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 5.247e+00 | `v_{0}` | `v_{0}` |
| 3 | 4.257e+00 | `Plus(v_{0}, v_{1})` | `(v_{0})+(v_{1})` |
| 4 | 4.071e+00 | `Times(v_{0}, Sqrt(v_{1}))` | `(v_{0})*(np.sqrt(v_{1}))` |
| 5 | 3.487e+00 | `Times(v_{0}, Divide(v_{1}, v_{3}))` | `(v_{0})*((v_{1}/v_{3}))` |
| 6 | 2.941e+00 | `Divide(Times(v_{0}, v_{1}), Sqrt(v_{3}))` | `((v_{0})*(v_{1})/np.sqrt(v_{3}))` |

### `II.34.29a`  — 正しい数式: `q*h/(4*pi*m)` （変数: q, h, m）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 3.551e-01 | `0` | `0` |
| 2 | 2.176e-01 | `Inv(v_{2})` | `(1/v_{2})` |
| 3 | 1.846e-01 | `Inv(ArcSin(v_{2}))` | `(1/np.arcsin(v_{2}))` |
| 4 | 1.762e-01 | `Divide(v_{0}, Exp(v_{2}))` | `(v_{0}/np.exp(v_{2}))` |
| 5 | 1.263e-01 | `Divide(v_{0}, Times(Pi, v_{2}))` | `(v_{0}/(4.1798)*(v_{2}))` |

### `II.34.29b`  — 正しい数式: `g_*mom*B*Jz/(h/(2*pi))` （変数: g_, h, Jz, mom, B）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 2.878e+02 | `v_{4}` | `v_{4}` |
| 2 | 2.575e+02 | `Exp(v_{4})` | `np.exp(v_{4})` |
| 3 | 2.458e+02 | `Tan(Tan(1))` | `np.tan(np.tan(1))` |
| 4 | 2.018e+02 | `Times(v_{2}, Exp(v_{4}))` | `(v_{2})*(np.exp(v_{4}))` |
| 5 | 1.967e+02 | `Times(v_{4}, Tan(Tan(1)))` | `(v_{4})*(np.tan(np.tan(1)))` |
| 6 | 1.756e+02 | `Times(v_{3}, Times(v_{2}, Exp(Pi)))` | `(v_{3})*((v_{2})*(np.exp(np.pi)))` |

### `II.35.18`  — 正しい数式: `n_0/(exp(mom*B/(kb*T))+exp(-mom*B/(kb*T)))` （変数: n_0, kb, T, mom, B）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 5.203e-01 | `1` | `1` |
| 2 | 3.046e-01 | `Inv(v_{3})` | `(1/v_{3})` |
| 3 | 2.756e-01 | `Divide(v_{0}, Pi)` | `(v_{0}/3.1978)` |
| 4 | 2.689e-01 | `Divide(ArcTan(v_{0}), v_{3})` | `(np.arctan(v_{0})/v_{3})` |
| 5 | 2.277e-01 | `Divide(v_{0}, Plus(v_{3}, v_{4}))` | `(v_{0}/(v_{3})+(v_{4}))` |
| 6 | 2.181e-01 | `ArcSin(Divide(v_{0}, Plus(v_{3}, v_{4})))` | `np.arcsin((v_{0}/(v_{3})+(v_{4})))` |

### `II.35.21`  — 正しい数式: `n_rho*mom*tanh(mom*B/(kb*T))` （変数: n_rho, mom, B, kb, T）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 5.490e+00 | `v_{1}` | `v_{1}` |
| 3 | 3.156e+00 | `Times(v_{0}, v_{1})` | `(v_{0})*(v_{1})` |
| 5 | 2.246e+00 | `Subtract(Times(v_{0}, v_{1}), v_{3})` | `((v_{0})*(v_{1}))-(v_{3})` |
| 6 | 2.115e+00 | `Times(v_{0}, Subtract(v_{1}, Sqrt(I)))` | `(v_{0})*((v_{1})-(np.sqrt(1j)))` |

### `II.36.38`  — 正しい数式: `mom*H/(kb*T)+(mom*alpha)/(epsilon*c**2*kb*T)*M` （変数: mom, H, kb, T, alpha, epsilon, c, M）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.116e+00 | `v_{0}` | `v_{0}` |
| 3 | 1.030e+00 | `Divide(Pi, v_{2})` | `(3.1036/v_{2})` |
| 4 | 9.252e-01 | `Plus(v_{0}, Cos(v_{3}))` | `(v_{0})+(np.cos(v_{3}))` |
| 5 | 9.028e-01 | `Times(E, Divide(v_{0}, v_{3}))` | `(1.5268)*((v_{0}/v_{3}))` |
| 6 | 8.275e-01 | `EML(Inv(v_{2}), Divide(v_{3}, v_{0}))` | `(np.exp((1/v_{2})) - np.log((v_{3}/v_{0})))` |

### `II.38.3`  — 正しい数式: `Y*A*x/d` （変数: Y, A, d, x）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.346e+01 | `v_{3}` | `v_{3}` |
| 2 | 1.157e+01 | `Exp(E)` | `np.exp(np.e)` |
| 3 | 9.005e+00 | `Times(v_{1}, v_{3})` | `(v_{1})*(v_{3})` |
| 5 | 8.233e+00 | `Plus(v_{0}, Times(v_{1}, v_{3}))` | `(v_{0})+((v_{1})*(v_{3}))` |
| 6 | 7.515e+00 | `Times(v_{3}, Times(v_{1}, Log(v_{0})))` | `(v_{3})*((v_{1})*(np.log(v_{0})))` |

### `II.38.14`  — 正しい数式: `Y/(2*(1+sigma))` （変数: Y, sigma）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 4.631e-01 | `0` | `0` |
| 2 | 1.816e-01 | `Inv(v_{1})` | `(1/v_{1})` |
| 3 | 1.709e-01 | `ArcTan(Inv(v_{1}))` | `np.arctan((1/v_{1}))` |
| 4 | 9.354e-02 | `Divide(Log(v_{0}), v_{1})` | `(np.log(v_{0})/v_{1})` |
| 5 | 6.336e-02 | `Sin(Divide(Log(v_{0}), v_{1}))` | `np.sin((np.log(v_{0})/v_{1}))` |
| 6 | 4.713e-02 | `Sin(Divide(v_{0}, Times(E, v_{1})))` | `np.sin((v_{0}/(2.8301)*(v_{1})))` |

### `III.4.32`  — 正しい数式: `1/(exp((h/(2*pi))*omega/(kb*T))-1)` （変数: h, omega, kb, T）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.036e+01 | `v_{3}` | `v_{3}` |
| 3 | 7.852e+00 | `Times(v_{2}, v_{3})` | `(v_{2})*(v_{3})` |
| 4 | 7.697e+00 | `Divide(Exp(Pi), v_{0})` | `(np.exp(np.pi)/v_{0})` |
| 5 | 7.539e+00 | `Divide(EML(Pi, v_{1}), v_{0})` | `((np.exp(np.pi) - np.log(v_{1}))/v_{0})` |
| 6 | 6.901e+00 | `Times(v_{3}, Plus(v_{2}, Sin(v_{0})))` | `(v_{3})*((v_{2})+(np.sin(v_{0})))` |

### `III.4.33`  — 正しい数式: `(h/(2*pi))*omega/(exp((h/(2*pi))*omega/(kb*T))-1)` （変数: h, omega, kb, T）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 6.953e+00 | `v_{3}` | `v_{3}` |
| 3 | 7.615e-01 | `Times(v_{2}, v_{3})` | `(v_{2})*(v_{3})` |
| 5 | 3.774e-01 | `Subtract(Times(v_{2}, v_{3}), 1)` | `((v_{2})*(v_{3}))-(0.7500)` |
| 6 | 3.690e-01 | `Subtract(Times(v_{2}, v_{3}), Log(Pi))` | `((v_{2})*(v_{3}))-(np.log(2.0003))` |

### `III.7.38`  — 正しい数式: `2*mom*B/(h/(2*pi))` （変数: mom, B, h）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 5.649e+01 | `v_{0}` | `v_{0}` |
| 2 | 3.792e+01 | `Exp(v_{0})` | `np.exp(v_{0})` |
| 4 | 3.111e+01 | `Times(v_{0}, Exp(E))` | `(v_{0})*(np.exp(np.e))` |
| 5 | 2.922e+01 | `Times(v_{0}, Times(v_{0}, v_{1}))` | `(v_{0})*((v_{0})*(v_{1}))` |

### `III.8.54`  — 正しい数式: `sin(E_n*t/(h/(2*pi)))**2` （変数: E_n, t, h）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 5.911e-01 | `0` | `0` |
| 2 | 3.540e-01 | `Sin(E)` | `np.sin(2.6751)` |
| 3 | 3.531e-01 | `Tan(Sin(I))` | `np.tan(np.sin(35546.6641))` |
| 4 | 3.522e-01 | `Divide(Sqrt(v_{0}), E)` | `(np.sqrt(v_{0})/2.6892)` |
| 5 | 3.514e-01 | `ArcTan(Divide(ArcTan(v_{0}), Pi))` | `np.arctan((np.arctan(v_{0})/1.9228))` |
| 6 | 3.512e-01 | `EML(Sqrt(Tan(ArcCos(v_{0}))), E)` | `(np.exp(np.sqrt(np.tan(np.arccos(v_{0})))) - np.log(...` |

### `III.9.52`  — 正しい数式: `(p_d*Ef*t/(h/(2*pi)))*sin((omega-omega_0)*t/2)**2/((omega-omega_0)*t/2)**2` （変数: p_d, Ef, t, h, omega, omega_0）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.907e+01 | `Pi` | `np.pi` |
| 2 | 1.451e+01 | `Exp(E)` | `np.exp(np.e)` |
| 3 | 1.444e+01 | `EML(E, v_{3})` | `(np.exp(np.e) - np.log(v_{3}))` |
| 4 | 1.379e+01 | `Times(v_{1}, Exp(v_{0}))` | `(v_{1})*(np.exp(v_{0}))` |
| 5 | 1.300e+01 | `Plus(Exp(v_{0}), Exp(v_{1}))` | `(np.exp(v_{0}))+(np.exp(v_{1}))` |
| 6 | 1.265e+01 | `Times(v_{1}, Exp(EML(1, v_{3})))` | `(v_{1})*(np.exp((np.exp(1) - np.log(v_{3}))))` |

### `III.12.43`  — 正しい数式: `n*(h/(2*pi))` （変数: n, h）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 9.674e-01 | `1` | `1` |
| 2 | 6.745e-01 | `Sqrt(v_{0})` | `np.sqrt(v_{0})` |
| 3 | 6.622e-01 | `Divide(v_{0}, Pi)` | `(v_{0}/1.7384)` |
| 4 | 6.010e-01 | `Log(Sqrt(Exp(v_{0})))` | `np.log(np.sqrt(np.exp(v_{0})))` |
| 5 | 1.895e-01 | `Times(v_{1}, Log(Sqrt(v_{0})))` | `(v_{1})*(np.log(np.sqrt(v_{0})))` |
| 6 | 1.773e-01 | `Divide(Log(v_{1}), Divide(Pi, v_{0}))` | `(np.log(v_{1})/(2.3155/v_{0}))` |

### `III.13.18`  — 正しい数式: `2*E_n*d**2*k/(h/(2*pi))` （変数: E_n, d, k, h）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 7.059e+02 | `v_{1}` | `v_{1}` |
| 2 | 6.687e+02 | `Exp(v_{1})` | `np.exp(v_{1})` |
| 3 | 6.614e+02 | `Tan(Tan(1))` | `np.tan(np.tan(1))` |
| 4 | 5.145e+02 | `Exp(Plus(E, v_{1}))` | `np.exp((np.e)+(v_{1}))` |
| 5 | 4.473e+02 | `Exp(Times(Pi, Sqrt(v_{1})))` | `np.exp((np.pi)*(np.sqrt(v_{1})))` |
| 6 | 3.597e+02 | `Times(v_{0}, Times(v_{2}, Exp(v_{1})))` | `(v_{0})*((v_{2})*(np.exp(v_{1})))` |

### `III.14.14`  — 正しい数式: `I_0*(exp(q*Volt/(kb*T))-1)` （変数: I_0, q, Volt, kb, T）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 7.479e+00 | `v_{0}` | `v_{0}` |
| 2 | 6.718e+00 | `Exp(v_{2})` | `np.exp(v_{2})` |
| 3 | 6.282e+00 | `Plus(v_{0}, v_{0})` | `(v_{0})+(v_{0})` |
| 4 | 5.815e+00 | `Times(v_{1}, Exp(v_{2}))` | `(v_{1})*(np.exp(v_{2}))` |
| 5 | 5.294e+00 | `Times(v_{1}, Times(v_{0}, v_{2}))` | `(v_{1})*((v_{0})*(v_{2}))` |

### `III.15.12`  — 正しい数式: `2*U*(1-cos(k*d))` （変数: U, k, d）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 5.760e+00 | `v_{0}` | `v_{0}` |
| 3 | 4.626e+00 | `Plus(v_{0}, v_{0})` | `(v_{0})+(v_{0})` |
| 5 | 4.573e+00 | `EML(Sqrt(v_{0}), Log(v_{2}))` | `(np.exp(np.sqrt(v_{0})) - np.log(np.log(v_{2})))` |
| 6 | 4.569e+00 | `EML(Sqrt(v_{0}), Sin(Log(v_{2})))` | `(np.exp(np.sqrt(v_{0})) - np.log(np.sin(np.log(v_{2}...` |

### `III.15.14`  — 正しい数式: `(h/(2*pi))**2/(2*E_n*d**2)` （変数: h, E_n, d）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.921e-02 | `0` | `0` |
| 2 | 1.639e-02 | `Sqrt(I)` | `np.sqrt(0.0001)` |
| 3 | 1.516e-02 | `Divide(I, v_{2})` | `(0.0456/v_{2})` |
| 4 | 1.322e-02 | `Divide(I, Exp(v_{2}))` | `(0.1409/np.exp(v_{2}))` |
| 5 | 1.259e-02 | `Divide(Divide(I, v_{2}), v_{2})` | `((0.0456/v_{2})/v_{2})` |
| 6 | 1.113e-02 | `Times(v_{0}, Divide(Sqrt(I), v_{2}))` | `(v_{0})*((np.sqrt(0.0001)/v_{2}))` |

### `III.15.27`  — 正しい数式: `2*pi*alpha/(n*d)` （変数: alpha, n, d）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 2.143e+00 | `v_{0}` | `v_{0}` |
| 4 | 1.871e+00 | `Plus(v_{0}, Sin(v_{2}))` | `(v_{0})+(np.sin(v_{2}))` |
| 5 | 1.648e+00 | `Times(E, Divide(v_{0}, v_{1}))` | `(2.1976)*((v_{0}/v_{1}))` |
| 6 | 1.480e+00 | `Divide(Exp(E), Times(v_{1}, v_{2}))` | `(np.exp(2.8509)/(v_{1})*(v_{2}))` |

### `III.17.37`  — 正しい数式: `beta*(1+alpha*cos(theta))` （変数: beta, alpha, theta）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 4.907e+00 | `0` | `0` |
| 2 | 4.469e+00 | `Cos(v_{2})` | `np.cos(v_{2})` |
| 3 | 4.219e+00 | `Log(Tan(v_{2}))` | `np.log(np.tan(v_{2}))` |
| 4 | 3.409e+00 | `Times(v_{1}, Cos(v_{2}))` | `(v_{1})*(np.cos(v_{2}))` |
| 5 | 3.041e+00 | `Times(v_{1}, Tan(Cos(v_{2})))` | `(v_{1})*(np.tan(np.cos(v_{2})))` |
| 6 | 2.745e+00 | `Times(Cos(v_{2}), Sqrt(Exp(v_{1})))` | `(np.cos(v_{2}))*(np.sqrt(np.exp(v_{1})))` |

### `III.21.20`  — 正しい数式: `-rho_c_0*q*A_vec/m` （変数: rho_c_0, q, A_vec, m）

| 複雑度 | RMSE | eml-sr 式 | Python 表現 |
|--------|------|-----------|------------|
| 1 | 1.464e+01 | `0` | `0` |
| 2 | 1.220e+01 | `Neg(v_{1})` | `(-v_{1})` |
| 3 | 9.865e+00 | `Tan(Sqrt(E))` | `np.tan(np.sqrt(np.e))` |
| 4 | 7.773e+00 | `Neg(Times(v_{1}, v_{2}))` | `(-(v_{1})*(v_{2}))` |
| 6 | 6.997e+00 | `Times(v_{2}, Subtract(Sin(v_{0}), v_{1}))` | `(v_{2})*((np.sin(v_{0}))-(v_{1}))` |

---

## 8. 変数数別の成功率

| 変数数 | 対象数 | 成功 (ok) | 部分回収 | 合計回収率 |
|--------|--------|-----------|----------|-----------|
| 1 | 1 | 0 | 1 | 100% |
| 2 | 16 | 6 | 0 | 38% |
| 3 | 37 | 3 | 0 | 8% |
| 4 | 25 | 0 | 0 | 0% |
| 5 | 12 | 0 | 0 | 0% |
| 6 | 6 | 0 | 0 | 0% |
| 8 | 1 | 0 | 0 | 0% |
| 9 | 1 | 0 | 0 | 0% |

---

## 9. 考察

### 9.1 前回実験との比較

| 指標 | 前回 (BEAM=300, N=200) | 今回 (BEAM=1000, N=750) | 変化 |
|------|----------------------|------------------------|------|
| 実行時間 | 2940s (49.0min) | 7576s (126.3min) | +2.6倍 |
| 回収成功 (ok) | 9式 (9.1%) | 9式 (9.1%) | 変化なし |
| 部分回収 (partial) | 3式 (3.0%) | 1式 (1.0%) | -2式 |
| 成功+部分合計 | 12式 (12.1%) | 10式 (10.1%) | -2式 |
| 1式平均時間 | ~30s | ~77s | +2.6倍 |

**結論:** BEAM_WIDTH を 300→1000、N_SAMPLES を 200→750 に増加させたが、**回収成功数は同一の9式**にとどまった。部分回収はむしろ減少（3→1式）。これは以下の理由が考えられる：

- 回収できた9式は複雑度が低く（3〜6）、ビーム幅に依存せず前回も正確に発見できていた
- 失敗した式は構造的な理由（Gaussian型・多変数・有理数係数）で失敗しており、ビーム幅増加だけでは解決できない
- N_SAMPLES 増加による数値的安定性向上は、部分回収の精度改善よりも**過学習リスク**の方向に働いた可能性がある

### 9.2 I.6.2a の部分回収精度改善

唯一 partial のまま残った I.6.2a（Gaussian分布の標準化）は：
- 前回: RMSE = **9.034e-3**（partial）
- 今回: RMSE = **6.013e-3**（partial）→ 精度が33%改善

このことは、N_SAMPLES 増加が数値的安定性に効いており、より精密な近似が可能になったことを示す。

### 9.3 EML 演算子の特性と失敗パターン

eml-sr の EML (Exponential-Modular-Logarithmic) 演算子は `exp(a) - log(b)` 型の構造をコンパクトに表現できる。しかし以下のパターンは複雑度 6 の範囲では依然として困難である：

**① Gaussian・確率型（複雑な指数構造）**
- `exp(-θ²/2)/√(2π)` (I.6.2a): exp内の二乗が複雑度6で表現困難
- `(m₀/√(1-v²/c²))` (I.10.7): 差の二乗＋sqrt の組み合わせ

**② 多変数の積・和（4変数以上）**
- 変数数4以上の方程式は全て失敗。探索空間の指数的拡大
- I.9.18 (9変数): 240秒かけても失敗

**③ 有理数係数を含む式**
- `1/2·k·x²` (I.14.4): 係数1/2が標準定数セット外
- `3/2·pr·V` (I.39.1): 係数3/2も同様

**④ スキップされた式**
- I.18.12, I.18.14, I.38.12, III.10.19, III.19.51: サンプル生成失敗（負値・ゼロ割り）

### 9.4 失敗例からの数学的推定の可能性

セクション 7 に示した Pareto front の全候補群は、正しい数式の部分構造を含む可能性がある：

- **複雑度 1〜2 の候補**: 「最も支配的な変数」を特定するのに有用。例えば I.9.18 (重力式) で複雑度2候補 `v_{1}/v_{7}` は `m₂/z₂` に相当し、引力が質量と距離に依存することを示している
- **複雑度 3〜4 の候補**: 演算子の組み合わせの一部を捉えている。例えば I.10.7 (ローレンツ因子) で `v_{0}/ArcTan(ArcTan(v_{2}))` は `m₀/f(v/c)` の形を示唆している
- **RMSE が低い失敗例**: 閾値ギリギリの失敗（RMSE < 0.1）は、正しい式の変形として解釈可能な場合がある

### 9.5 今後の改善方針

| 改善案 | 期待効果 | コスト |
|--------|---------|--------|
| MAX_COMPLEXITY を 7〜8 に増加 | Gaussian型・相対論式の回収 | 1式あたり10〜100倍の時間増 |
| 定数セットに 1/2, 3/2, 2/3 を追加 | 有理数係数の式を回収 | 探索空間増 |
| 変数の次元情報を活用 | 4変数以上の絞り込み | 前処理コスト |
| 区分的回帰（変数をグループ化） | 多変数問題の分解 | アーキテクチャ変更 |
| Pareto front を初期解として再探索 | 失敗例の段階的改善 | 追加実行コスト |

今回の実験では 99 個の方程式に対して **9式 (9.1%)** を完全回収し、部分的回収も含めると **10式 (10.1%)** となった。BEAM_WIDTH・N_SAMPLES の増加は実行時間を2.6倍に増加させたが回収率は変わらず、**探索パラメータよりも MAX_COMPLEXITY と定数セットの拡張が回収率向上の鍵**であることが示された。

---

*本レポートは `gh3_allfunc_moreestimate_sr.py` により自動生成・手動補完されました。*
