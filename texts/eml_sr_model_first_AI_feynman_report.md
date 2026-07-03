# EML-SR Model First AI — Feynman Equations 全式推定レポート

**生成日時:** 2026-07-03 04:20:43  
**エンジン:** `eml-sr_model_first_AI` (Rust / PyO3)  

---

## 1. 実験概要

### 目的

従来の Python 版 eml-sr (`sample_code/gh3_allfunc_moreestimate_sr.py`) と同一のハイパーパラメータ (BEAM_WIDTH=1000, N_SAMPLES=750, MAX_COMPLEXITY=6) を用い、新モデル **eml-sr_model_first_AI** により Feynman 方程式ベンチマークを実行する。従来実装との回収率・実行時間・出力数式の差異を比較可能な形式で記録する。

### 実験設定

| パラメータ | 値 | 備考 |
|-----------|-----|------|
| N_SAMPLES | 750 | 従来実験と同一 |
| BEAM_WIDTH | 1000 | 従来実験と同一 |
| MAX_COMPLEXITY | 6 | 従来実験と同一 |
| RMSE 閾値 (ok) | 0.0001 | 従来実験と同一 |
| RMSE 閾値 (partial) | 0.01 | 従来実験と同一 |
| 合計実行時間 | 10558s (176.0min) | — |

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

| # | 式ID | 変数数 | ステータス | RMSE | 複雑度 | 発見式 | 時間(s) |
|---|------|--------|------------|------|--------|--------|---------|
| 1 | `I.6.2a` | 1 | 🟡 partial | 6.623e-03 | 6 | `Tan(Subtract(C, Sin(ArcTan(v_{0}))))` | 50.8 |
| 2 | `I.6.2` | 2 | ❌ failed | 2.157e-02 | 6 | `Divide(ArcTan(C), ArcSin(Exp(v_{1})))` | 70.2 |
| 3 | `I.6.2b` | 3 | ❌ failed | 3.747e-02 | 6 | `Sin(Divide(Cos(C), ArcSin(v_{0})))` | 90.3 |
| 4 | `I.8.14` | 4 | ❌ failed | 9.811e-01 | 6 | `EML(C, Subtract(C, Cos(v_{1})))` | 124.2 |
| 5 | `I.9.18` | 9 | ❌ failed | 1.038e-01 | 6 | `Times(v_{0}, Times(v_{2}, Sin(C)))` | 213.9 |
| 6 | `I.10.7` | 3 | ❌ failed | 8.658e-02 | 5 | `Divide(v_{0}, ArcTan(ArcTan(v_{2})))` | 97.1 |
| 7 | `I.11.19` | 6 | ❌ failed | 6.961e+00 | 6 | `Plus(Neg(C), Times(v_{1}, v_{4}))` | 212.3 |
| 8 | `I.12.1` | 2 | ✅ ok | 0.000e+00 | 3 | `Times(v_{0}, v_{1})` | 86.0 |
| 9 | `I.12.2` | 4 | ❌ failed | 4.236e-02 | 6 | `Divide(Inv(v_{2}), EML(v_{3}, v_{0}))` | 105.4 |
| 10 | `I.12.4` | 3 | ❌ failed | 1.525e-02 | 6 | `Divide(Divide(ArcTan(C), v_{1}), v_{2})` | 91.4 |
| 11 | `I.12.5` | 2 | ✅ ok | 0.000e+00 | 3 | `Times(v_{0}, v_{1})` | 83.8 |
| 12 | `I.12.11` | 5 | ❌ failed | 1.816e+01 | 6 | `Times(v_{0}, Cos(Log(Neg(v_{4}))))` | 161.6 |
| 13 | `I.13.4` | 4 | ❌ failed | 1.801e+01 | 6 | `Times(v_{0}, Divide(v_{1}, Sin(C)))` | 130.9 |
| 14 | `I.13.12` | 5 | ❌ failed | 6.314e+00 | 5 | `Times(v_{1}, Subtract(v_{2}, v_{3}))` | 115.1 |
| 15 | `I.14.3` | 3 | ✅ ok | 3.254e-15 | 5 | `Times(v_{1}, Times(v_{0}, v_{2}))` | 111.5 |
| 16 | `I.14.4` | 2 | ❌ failed | 1.754e+00 | 6 | `Times(v_{0}, Sqrt(EML(v_{1}, v_{0})))` | 90.3 |
| 17 | `I.15.3x` | 4 | ❌ failed | 2.101e-01 | 5 | `Subtract(v_{0}, Times(v_{1}, v_{3}))` | 131.4 |
| 18 | `I.15.3t` | 4 | ❌ failed | 1.009e-01 | 6 | `Subtract(v_{3}, Divide(v_{0}, Tan(C)))` | 112.8 |
| 19 | `I.15.1` | 3 | ❌ failed | 1.952e-01 | 6 | `Times(v_{1}, Plus(v_{0}, Inv(v_{2})))` | 102.1 |
| 20 | `I.16.6` | 3 | ❌ failed | 3.590e-01 | 5 | `Times(v_{0}, Sin(ArcTan(v_{1})))` | 90.2 |
| 21 | `I.18.4` | 4 | ❌ failed | 2.200e-01 | 6 | `Times(Cos(C), Plus(v_{2}, v_{3}))` | 119.5 |
| 22 | `I.18.12` | 2 | ⏭️ skipped | N/A | None | `N/A` | 0.0 |
| 23 | `I.18.14` | 3 | ⏭️ skipped | N/A | None | `N/A` | 0.0 |
| 24 | `I.24.6` | 4 | ❌ failed | 7.933e+00 | 6 | `Times(v_{0}, EML(v_{3}, Tan(v_{2})))` | 133.5 |
| 25 | `I.25.13` | 2 | ✅ ok | 1.180e-16 | 3 | `Divide(v_{0}, v_{1})` | 56.9 |
| 26 | `I.26.2` | 2 | ✅ ok | 4.258e-17 | 5 | `ArcSin(Times(v_{0}, Sin(v_{1})))` | 68.4 |
| 27 | `I.27.6` | 3 | ❌ failed | 1.682e-01 | 4 | `ArcTan(Divide(v_{1}, v_{2}))` | 76.6 |
| 28 | `I.29.4` | 2 | ✅ ok | 1.846e-16 | 3 | `Divide(v_{0}, v_{1})` | 66.3 |
| 29 | `I.29.16` | 4 | ❌ failed | 1.662e+00 | 6 | `Times(Tan(C), Plus(v_{0}, v_{1}))` | 132.7 |
| 30 | `I.30.3` | 3 | ❌ failed | 1.869e+00 | 6 | `Times(v_{0}, Exp(Sqrt(Cos(v_{1}))))` | 100.9 |
| 31 | `I.30.5` | 3 | ✅ ok | 2.671e-17 | 6 | `ArcSin(Divide(v_{0}, Times(v_{1}, v_{2})))` | 86.2 |
| 32 | `I.32.5` | 4 | ❌ failed | 5.779e-01 | 5 | `Tan(Divide(Log(v_{0}), v_{3}))` | 111.7 |
| 33 | `I.32.17` | 6 | ❌ failed | 4.474e+00 | 6 | `Divide(Exp(v_{4}), Subtract(v_{5}, C))` | 162.0 |
| 34 | `I.34.8` | 4 | ❌ failed | 7.028e+00 | 6 | `Times(v_{1}, Times(v_{0}, Log(v_{2})))` | 134.8 |
| 35 | `I.34.1` | 3 | ❌ failed | 6.098e-01 | 6 | `EML(Sqrt(v_{2}), Subtract(v_{0}, v_{1}))` | 103.1 |
| 36 | `I.34.14` | 3 | ❌ failed | 2.866e-01 | 6 | `Times(v_{2}, EML(C, ArcCos(v_{0})))` | 104.3 |
| 37 | `I.34.27` | 2 | ❌ failed | 1.866e-01 | 5 | `Times(v_{0}, Log(Sqrt(v_{1})))` | 65.5 |
| 38 | `I.37.4` | 3 | ❌ failed | 1.436e+00 | 5 | `Exp(ArcCos(Neg(Cos(v_{2}))))` | 110.2 |
| 39 | `I.38.12` | 3 | ⏭️ skipped | N/A | None | `N/A` | 0.0 |
| 40 | `I.39.1` | 2 | ❌ failed | 7.058e-01 | 6 | `Times(v_{1}, Times(v_{0}, ArcTan(C)))` | 84.9 |
| 41 | `I.39.11` | 3 | ❌ failed | 1.547e+00 | 6 | `Times(v_{1}, Divide(v_{2}, Sqrt(v_{0})))` | 100.9 |
| 42 | `I.39.22` | 4 | ❌ failed | 6.956e+00 | 6 | `Times(v_{3}, Times(v_{0}, Log(v_{1})))` | 131.5 |
| 43 | `I.40.1` | 6 | ❌ failed | 4.804e-01 | 5 | `Divide(v_{0}, Times(v_{1}, v_{4}))` | 142.0 |
| 44 | `I.41.16` | 5 | ❌ failed | 1.885e+00 | 6 | `Times(v_{0}, Inv(Plus(C, v_{4})))` | 136.3 |
| 45 | `I.43.16` | 4 | ❌ failed | 7.272e+00 | 6 | `EML(EML(ArcTan(v_{0}), v_{3}), v_{0})` | 140.0 |
| 46 | `I.43.31` | 3 | ✅ ok | 0.000e+00 | 5 | `Times(v_{1}, Times(v_{0}, v_{2}))` | 110.2 |
| 47 | `I.43.43` | 4 | ❌ failed | 1.008e+00 | 6 | `Times(Log(v_{1}), Divide(v_{3}, v_{2}))` | 109.9 |
| 48 | `I.44.4` | 5 | ❌ failed | 1.385e+01 | 6 | `Times(Log(C), Subtract(v_{4}, v_{3}))` | 113.7 |
| 49 | `I.47.23` | 3 | ❌ failed | 2.802e-01 | 6 | `Divide(Plus(v_{0}, v_{1}), ArcSin(v_{2}))` | 90.6 |
| 50 | `I.48.2` | 3 | ❌ failed | 4.399e+00 | 5 | `Times(v_{0}, Times(v_{2}, v_{2}))` | 106.0 |
| 51 | `I.50.26` | 4 | ❌ failed | 1.790e+00 | 6 | `Subtract(Plus(v_{1}, v_{3}), Log(C))` | 102.1 |
| 52 | `II.2.42` | 5 | ❌ failed | 3.924e+00 | 5 | `Times(v_{3}, Subtract(v_{2}, v_{1}))` | 113.6 |
| 53 | `II.3.24` | 2 | ❌ failed | 1.092e-02 | 6 | `Divide(Log(Sqrt(v_{0})), Exp(v_{1}))` | 73.2 |
| 54 | `II.4.23` | 3 | ❌ failed | 1.775e-02 | 6 | `Divide(Divide(Neg(C), v_{1}), v_{2})` | 95.6 |
| 55 | `II.6.11` | 4 | ❌ failed | 1.514e-02 | 6 | `Times(Cos(v_{2}), Log(ArcSin(C)))` | 126.5 |
| 56 | `II.6.15a` | 6 | ❌ failed | 1.897e-01 | 6 | `Divide(Inv(v_{0}), Log(Tan(v_{2})))` | 166.2 |
| 57 | `II.6.15b` | 4 | ❌ failed | 2.526e-02 | 6 | `Divide(Cos(v_{2}), Exp(Exp(v_{3})))` | 120.3 |
| 58 | `II.8.7` | 3 | ❌ failed | 6.529e-02 | 6 | `Times(v_{0}, Divide(Log(C), v_{2}))` | 88.6 |
| 59 | `II.8.31` | 2 | ❌ failed | 1.754e+00 | 6 | `Times(v_{0}, Sqrt(EML(v_{1}, v_{0})))` | 88.8 |
| 60 | `II.10.9` | 3 | ❌ failed | 7.008e-02 | 6 | `Divide(Divide(v_{0}, v_{1}), ArcSin(v_{2}))` | 80.5 |
| 61 | `II.11.3` | 5 | ❌ failed | 7.792e-02 | 6 | `Divide(Divide(v_{0}, v_{3}), ArcSin(v_{2}))` | 129.3 |
| 62 | `II.11.17` | 6 | ❌ failed | 1.029e+00 | 6 | `Times(v_{0}, Neg(Subtract(v_{3}, C)))` | 137.1 |
| 63 | `II.11.20` | 5 | ❌ failed | 4.623e+00 | 6 | `Times(v_{0}, Divide(v_{1}, Sqrt(v_{4})))` | 154.1 |
| 64 | `II.11.27` | 4 | ❌ failed | 2.448e-01 | 6 | `Divide(Times(v_{0}, v_{1}), Tan(C))` | 85.5 |
| 65 | `II.11.28` | 2 | ❌ failed | 4.446e-02 | 4 | `Exp(Times(v_{0}, v_{1}))` | 62.2 |
| 66 | `II.13.17` | 4 | ❌ failed | 1.697e-02 | 6 | `Divide(Divide(ArcTan(C), v_{3}), v_{1})` | 123.7 |
| 67 | `II.13.23` | 3 | ❌ failed | 1.006e-01 | 5 | `Divide(v_{0}, ArcTan(ArcTan(v_{2})))` | 94.3 |
| 68 | `II.13.34` | 3 | ❌ failed | 2.088e-01 | 6 | `Times(v_{1}, Plus(v_{0}, Inv(v_{2})))` | 107.6 |
| 69 | `II.15.4` | 3 | ❌ failed | 3.126e+00 | 6 | `Times(Cos(v_{2}), Neg(Log(C)))` | 91.6 |
| 70 | `II.15.5` | 3 | ❌ failed | 3.419e+00 | 5 | `Divide(v_{0}, Exp(Cos(v_{2})))` | 89.8 |
| 71 | `II.21.32` | 5 | ❌ failed | 2.939e-02 | 6 | `Divide(Divide(Neg(C), v_{1}), v_{2})` | 146.5 |
| 72 | `II.24.17` | 3 | ❌ failed | 9.598e-02 | 6 | `Subtract(Divide(v_{0}, v_{1}), Inv(v_{0}))` | 107.6 |
| 73 | `II.27.16` | 3 | ❌ failed | 5.369e+01 | 6 | `Times(v_{1}, Exp(Exp(ArcTan(v_{2}))))` | 105.2 |
| 74 | `II.27.18` | 2 | ✅ ok | 0.000e+00 | 5 | `Times(v_{0}, Times(v_{1}, v_{1}))` | 87.8 |
| 75 | `II.34.2a` | 3 | ❌ failed | 2.544e-01 | 5 | `Divide(v_{1}, Plus(v_{2}, v_{2}))` | 73.9 |
| 76 | `II.34.2` | 3 | ❌ failed | 2.995e+00 | 6 | `Times(v_{2}, Times(v_{1}, Sqrt(v_{0})))` | 110.8 |
| 77 | `II.34.11` | 4 | ❌ failed | 2.941e+00 | 6 | `Times(v_{1}, Divide(v_{0}, Sqrt(v_{3})))` | 129.3 |
| 78 | `II.34.29a` | 3 | ❌ failed | 1.262e-01 | 6 | `Divide(Times(v_{0}, Log(C)), v_{2})` | 86.5 |
| 79 | `II.34.29b` | 5 | ❌ failed | 1.682e+02 | 6 | `Times(v_{4}, Divide(Exp(C), v_{1}))` | 163.6 |
| 80 | `II.35.18` | 5 | ❌ failed | 2.181e-01 | 6 | `ArcSin(Divide(v_{0}, Plus(v_{3}, v_{4})))` | 116.7 |
| 81 | `II.35.21` | 5 | ❌ failed | 2.081e+00 | 6 | `Times(v_{0}, Subtract(v_{1}, Log(v_{3})))` | 173.5 |
| 82 | `II.36.38` | 8 | ❌ failed | 8.275e-01 | 6 | `EML(Inv(v_{2}), Divide(v_{3}, v_{0}))` | 210.0 |
| 83 | `II.38.3` | 4 | ❌ failed | 7.515e+00 | 6 | `Times(v_{3}, Times(v_{1}, Log(v_{0})))` | 127.7 |
| 84 | `II.38.14` | 2 | ❌ failed | 5.823e-02 | 6 | `Times(Log(v_{0}), ArcTan(Inv(v_{1})))` | 58.7 |
| 85 | `III.4.32` | 4 | ❌ failed | 6.128e+00 | 6 | `Divide(Divide(Exp(C), v_{0}), v_{1})` | 144.9 |
| 86 | `III.4.33` | 4 | ❌ failed | 3.682e-01 | 6 | `Subtract(Times(v_{2}, v_{3}), Sin(C))` | 137.8 |
| 87 | `III.7.38` | 3 | ❌ failed | 2.101e+01 | 6 | `Divide(Divide(v_{0}, ArcSin(C)), v_{2})` | 103.8 |
| 88 | `III.8.54` | 3 | ❌ failed | 3.513e-01 | 6 | `Divide(v_{0}, ArcSin(ArcSin(ArcSin(v_{0}))))` | 85.1 |
| 89 | `III.9.52` | 6 | ❌ failed | 1.261e+01 | 6 | `Times(v_{1}, Divide(Neg(C), v_{3}))` | 196.9 |
| 90 | `III.10.19` | 3 | ⏭️ skipped | N/A | None | `N/A` | 0.0 |
| 91 | `III.12.43` | 2 | ❌ failed | 1.895e-01 | 5 | `Times(v_{1}, Log(Sqrt(v_{0})))` | 65.7 |
| 92 | `III.13.18` | 4 | ❌ failed | 3.597e+02 | 6 | `Times(v_{2}, Times(v_{0}, Exp(v_{1})))` | 144.3 |
| 93 | `III.14.14` | 5 | ❌ failed | 5.294e+00 | 5 | `Times(v_{1}, Times(v_{0}, v_{2}))` | 156.0 |
| 94 | `III.15.12` | 3 | ❌ failed | 4.530e+00 | 6 | `Times(v_{0}, ArcCos(Neg(Inv(v_{2}))))` | 111.0 |
| 95 | `III.15.14` | 3 | ❌ failed | 1.054e-02 | 6 | `Divide(Divide(v_{0}, v_{2}), Tan(C))` | 94.1 |
| 96 | `III.15.27` | 3 | ❌ failed | 1.627e+00 | 6 | `Times(Log(C), Divide(v_{0}, v_{1}))` | 92.0 |
| 97 | `III.17.37` | 3 | ❌ failed | 2.745e+00 | 6 | `Times(Cos(v_{2}), Sqrt(Exp(v_{1})))` | 89.1 |
| 98 | `III.19.51` | 4 | ⏭️ skipped | N/A | None | `N/A` | 0.0 |
| 99 | `III.21.20` | 4 | ❌ failed | 6.827e+00 | 6 | `Divide(Neg(C), Divide(v_{3}, v_{1}))` | 131.7 |

---

## 4. 回収成功した方程式の詳細

### `I.12.1`

- **変数:** mu, Nn
- **発見式:** `Times(v_{0}, v_{1})`
- **Python 表現:** `(v_{0})*(v_{1})`
- **RMSE:** 0.000e+00
- **複雑度:** 3
- **実行時間:** 85.95s

### `I.12.5`

- **変数:** q2, Ef
- **発見式:** `Times(v_{0}, v_{1})`
- **Python 表現:** `(v_{0})*(v_{1})`
- **RMSE:** 0.000e+00
- **複雑度:** 3
- **実行時間:** 83.80s

### `I.14.3`

- **変数:** m, g, z
- **発見式:** `Times(v_{1}, Times(v_{0}, v_{2}))`
- **Python 表現:** `(v_{1})*((v_{0})*(v_{2}))`
- **RMSE:** 3.254e-15
- **複雑度:** 5
- **実行時間:** 111.47s

### `I.25.13`

- **変数:** q, C
- **発見式:** `Divide(v_{0}, v_{1})`
- **Python 表現:** `(v_{0}/v_{1})`
- **RMSE:** 1.180e-16
- **複雑度:** 3
- **実行時間:** 56.93s

### `I.26.2`

- **変数:** n, theta2
- **発見式:** `ArcSin(Times(v_{0}, Sin(v_{1})))`
- **Python 表現:** `np.arcsin((v_{0})*(np.sin(v_{1})))`
- **RMSE:** 4.258e-17
- **複雑度:** 5
- **実行時間:** 68.40s

### `I.29.4`

- **変数:** omega, c
- **発見式:** `Divide(v_{0}, v_{1})`
- **Python 表現:** `(v_{0}/v_{1})`
- **RMSE:** 1.846e-16
- **複雑度:** 3
- **実行時間:** 66.30s

### `I.30.5`

- **変数:** lambd, d, n
- **発見式:** `ArcSin(Divide(v_{0}, Times(v_{1}, v_{2})))`
- **Python 表現:** `np.arcsin((v_{0}/(v_{1})*(v_{2})))`
- **RMSE:** 2.671e-17
- **複雑度:** 6
- **実行時間:** 86.23s

### `I.43.31`

- **変数:** mob, T, kb
- **発見式:** `Times(v_{1}, Times(v_{0}, v_{2}))`
- **Python 表現:** `(v_{1})*((v_{0})*(v_{2}))`
- **RMSE:** 0.000e+00
- **複雑度:** 5
- **実行時間:** 110.20s

### `II.27.18`

- **変数:** epsilon, Ef
- **発見式:** `Times(v_{0}, Times(v_{1}, v_{1}))`
- **Python 表現:** `(v_{0})*((v_{1})*(v_{1}))`
- **RMSE:** 0.000e+00
- **複雑度:** 5
- **実行時間:** 87.83s

---

## 5. 部分的回収の方程式

### `I.6.2a`
- **変数:** theta
- **発見式:** `Tan(Subtract(C, Sin(ArcTan(v_{0}))))`
- **RMSE:** 6.623e-03
- **複雑度:** 6

---

## 6. 失敗した方程式と出力数式

以下は推定に失敗（RMSE >= 1e-4）した方程式の一覧である。Pareto front の最良候補を記録する。

| # | 式ID | 変数 | RMSE | 発見式 | Python 表現 |
|---|------|------|------|--------|-------------|
| 2 | `I.6.2` | sigma, theta | 2.157e-02 | `Divide(ArcTan(C), ArcSin(Exp(v_{1})))` | `(np.arctan(p_{0})/np.arcsin(np.exp(v_{1})))` |
| 3 | `I.6.2b` | sigma, theta, theta1 | 3.747e-02 | `Sin(Divide(Cos(C), ArcSin(v_{0})))` | `np.sin((np.cos(p_{0})/np.arcsin(v_{0})))` |
| 4 | `I.8.14` | x1, x2, y1, y2 | 9.811e-01 | `EML(C, Subtract(C, Cos(v_{1})))` | `(np.exp(p_{0}) - np.log((p_{1})-(np.cos(v_{1}))))` |
| 5 | `I.9.18` | m1, m2, G, x1, x2, y1, y2, z1, z2 | 1.038e-01 | `Times(v_{0}, Times(v_{2}, Sin(C)))` | `(v_{0})*((v_{2})*(np.sin(p_{0})))` |
| 6 | `I.10.7` | m_0, v, c | 8.658e-02 | `Divide(v_{0}, ArcTan(ArcTan(v_{2})))` | `(v_{0}/np.arctan(np.arctan(v_{2})))` |
| 7 | `I.11.19` | x1, x2, x3, y1, y2, y3 | 6.961e+00 | `Plus(Neg(C), Times(v_{1}, v_{4}))` | `((-p_{0}))+((v_{1})*(v_{4}))` |
| 9 | `I.12.2` | q1, q2, epsilon, r | 4.236e-02 | `Divide(Inv(v_{2}), EML(v_{3}, v_{0}))` | `((1/v_{2})/(np.exp(v_{3}) - np.log(v_{0})))` |
| 10 | `I.12.4` | q1, epsilon, r | 1.525e-02 | `Divide(Divide(ArcTan(C), v_{1}), v_{2})` | `((np.arctan(p_{0})/v_{1})/v_{2})` |
| 12 | `I.12.11` | q, Ef, B, v, theta | 1.816e+01 | `Times(v_{0}, Cos(Log(Neg(v_{4}))))` | `(v_{0})*(np.cos(np.log((-v_{4}))))` |
| 13 | `I.13.4` | m, v, u, w | 1.801e+01 | `Times(v_{0}, Divide(v_{1}, Sin(C)))` | `(v_{0})*((v_{1}/np.sin(p_{0})))` |
| 14 | `I.13.12` | m1, m2, r1, r2, G | 6.314e+00 | `Times(v_{1}, Subtract(v_{2}, v_{3}))` | `(v_{1})*((v_{2})-(v_{3}))` |
| 16 | `I.14.4` | k_spring, x | 1.754e+00 | `Times(v_{0}, Sqrt(EML(v_{1}, v_{0})))` | `(v_{0})*(np.sqrt((np.exp(v_{1}) - np.log(v_{0}))))` |
| 17 | `I.15.3x` | x, u, c, t | 2.101e-01 | `Subtract(v_{0}, Times(v_{1}, v_{3}))` | `(v_{0})-((v_{1})*(v_{3}))` |
| 18 | `I.15.3t` | x, c, u, t | 1.009e-01 | `Subtract(v_{3}, Divide(v_{0}, Tan(C)))` | `(v_{3})-((v_{0}/np.tan(p_{0})))` |
| 19 | `I.15.1` | m_0, v, c | 1.952e-01 | `Times(v_{1}, Plus(v_{0}, Inv(v_{2})))` | `(v_{1})*((v_{0})+((1/v_{2})))` |
| 20 | `I.16.6` | c, v, u | 3.590e-01 | `Times(v_{0}, Sin(ArcTan(v_{1})))` | `(v_{0})*(np.sin(np.arctan(v_{1})))` |
| 21 | `I.18.4` | m1, m2, r1, r2 | 2.200e-01 | `Times(Cos(C), Plus(v_{2}, v_{3}))` | `(np.cos(p_{0}))*((v_{2})+(v_{3}))` |
| 24 | `I.24.6` | m, omega, omega_0, x | 7.933e+00 | `Times(v_{0}, EML(v_{3}, Tan(v_{2})))` | `(v_{0})*((np.exp(v_{3}) - np.log(np.tan(v_{2}))))` |
| 27 | `I.27.6` | d1, d2, n | 1.682e-01 | `ArcTan(Divide(v_{1}, v_{2}))` | `np.arctan((v_{1}/v_{2}))` |
| 29 | `I.29.16` | x1, x2, theta1, theta2 | 1.662e+00 | `Times(Tan(C), Plus(v_{0}, v_{1}))` | `(np.tan(p_{0}))*((v_{0})+(v_{1}))` |
| 30 | `I.30.3` | Int_0, theta, n | 1.869e+00 | `Times(v_{0}, Exp(Sqrt(Cos(v_{1}))))` | `(v_{0})*(np.exp(np.sqrt(np.cos(v_{1}))))` |
| 32 | `I.32.5` | q, a, epsilon, c | 5.779e-01 | `Tan(Divide(Log(v_{0}), v_{3}))` | `np.tan((np.log(v_{0})/v_{3}))` |
| 33 | `I.32.17` | epsilon, c, Ef, r, omega, omega_0 | 4.474e+00 | `Divide(Exp(v_{4}), Subtract(v_{5}, C))` | `(np.exp(v_{4})/(v_{5})-(p_{0}))` |
| 34 | `I.34.8` | q, v, B, p | 7.028e+00 | `Times(v_{1}, Times(v_{0}, Log(v_{2})))` | `(v_{1})*((v_{0})*(np.log(v_{2})))` |
| 35 | `I.34.1` | c, v, omega_0 | 6.098e-01 | `EML(Sqrt(v_{2}), Subtract(v_{0}, v_{1}))` | `(np.exp(np.sqrt(v_{2})) - np.log((v_{0})-(v_{1})))` |
| 36 | `I.34.14` | c, v, omega_0 | 2.866e-01 | `Times(v_{2}, EML(C, ArcCos(v_{0})))` | `(v_{2})*((np.exp(p_{0}) - np.log(np.arccos(v_{0}))))` |
| 37 | `I.34.27` | omega, h | 1.866e-01 | `Times(v_{0}, Log(Sqrt(v_{1})))` | `(v_{0})*(np.log(np.sqrt(v_{1})))` |
| 38 | `I.37.4` | I1, I2, delta | 1.436e+00 | `Exp(ArcCos(Neg(Cos(v_{2}))))` | `np.exp(np.arccos((-np.cos(v_{2}))))` |
| 40 | `I.39.1` | pr, V | 7.058e-01 | `Times(v_{1}, Times(v_{0}, ArcTan(C)))` | `(v_{1})*((v_{0})*(np.arctan(p_{0})))` |
| 41 | `I.39.11` | gamma, pr, V | 1.547e+00 | `Times(v_{1}, Divide(v_{2}, Sqrt(v_{0})))` | `(v_{1})*((v_{2}/np.sqrt(v_{0})))` |
| 42 | `I.39.22` | n, T, V, kb | 6.956e+00 | `Times(v_{3}, Times(v_{0}, Log(v_{1})))` | `(v_{3})*((v_{0})*(np.log(v_{1})))` |
| 43 | `I.40.1` | n_0, m, x, T, g, kb | 4.804e-01 | `Divide(v_{0}, Times(v_{1}, v_{4}))` | `(v_{0}/(v_{1})*(v_{4}))` |
| 44 | `I.41.16` | omega, T, h, kb, c | 1.885e+00 | `Times(v_{0}, Inv(Plus(C, v_{4})))` | `(v_{0})*((1/(p_{0})+(v_{4})))` |
| 45 | `I.43.16` | mu_drift, q, Volt, d | 7.272e+00 | `EML(EML(ArcTan(v_{0}), v_{3}), v_{0})` | `(np.exp((np.exp(np.arctan(v_{0})) - np.log(v_{3}))) - np....` |
| 47 | `I.43.43` | gamma, kb, A, v | 1.008e+00 | `Times(Log(v_{1}), Divide(v_{3}, v_{2}))` | `(np.log(v_{1}))*((v_{3}/v_{2}))` |
| 48 | `I.44.4` | n, kb, T, V1, V2 | 1.385e+01 | `Times(Log(C), Subtract(v_{4}, v_{3}))` | `(np.log(p_{0}))*((v_{4})-(v_{3}))` |
| 49 | `I.47.23` | gamma, pr, rho | 2.802e-01 | `Divide(Plus(v_{0}, v_{1}), ArcSin(v_{2}))` | `((v_{0})+(v_{1})/np.arcsin(v_{2}))` |
| 50 | `I.48.2` | m, v, c | 4.399e+00 | `Times(v_{0}, Times(v_{2}, v_{2}))` | `(v_{0})*((v_{2})*(v_{2}))` |
| 51 | `I.50.26` | x1, omega, t, alpha | 1.790e+00 | `Subtract(Plus(v_{1}, v_{3}), Log(C))` | `((v_{1})+(v_{3}))-(np.log(p_{0}))` |
| 52 | `II.2.42` | kappa, T1, T2, A, d | 3.924e+00 | `Times(v_{3}, Subtract(v_{2}, v_{1}))` | `(v_{3})*((v_{2})-(v_{1}))` |
| 53 | `II.3.24` | Pwr, r | 1.092e-02 | `Divide(Log(Sqrt(v_{0})), Exp(v_{1}))` | `(np.log(np.sqrt(v_{0}))/np.exp(v_{1}))` |
| 54 | `II.4.23` | q, epsilon, r | 1.775e-02 | `Divide(Divide(Neg(C), v_{1}), v_{2})` | `(((-p_{0})/v_{1})/v_{2})` |
| 55 | `II.6.11` | epsilon, p_d, theta, r | 1.514e-02 | `Times(Cos(v_{2}), Log(ArcSin(C)))` | `(np.cos(v_{2}))*(np.log(np.arcsin(p_{0})))` |
| 56 | `II.6.15a` | epsilon, p_d, r, x, y, z | 1.897e-01 | `Divide(Inv(v_{0}), Log(Tan(v_{2})))` | `((1/v_{0})/np.log(np.tan(v_{2})))` |
| 57 | `II.6.15b` | epsilon, p_d, theta, r | 2.526e-02 | `Divide(Cos(v_{2}), Exp(Exp(v_{3})))` | `(np.cos(v_{2})/np.exp(np.exp(v_{3})))` |
| 58 | `II.8.7` | q, epsilon, d | 6.529e-02 | `Times(v_{0}, Divide(Log(C), v_{2}))` | `(v_{0})*((np.log(p_{0})/v_{2}))` |
| 59 | `II.8.31` | epsilon, Ef | 1.754e+00 | `Times(v_{0}, Sqrt(EML(v_{1}, v_{0})))` | `(v_{0})*(np.sqrt((np.exp(v_{1}) - np.log(v_{0}))))` |
| 60 | `II.10.9` | sigma_den, epsilon, chi | 7.008e-02 | `Divide(Divide(v_{0}, v_{1}), ArcSin(v_{2}))` | `((v_{0}/v_{1})/np.arcsin(v_{2}))` |
| 61 | `II.11.3` | q, Ef, m, omega_0, omega | 7.792e-02 | `Divide(Divide(v_{0}, v_{3}), ArcSin(v_{2}))` | `((v_{0}/v_{3})/np.arcsin(v_{2}))` |
| 62 | `II.11.17` | n_0, kb, T, theta, p_d, Ef | 1.029e+00 | `Times(v_{0}, Neg(Subtract(v_{3}, C)))` | `(v_{0})*((-(v_{3})-(p_{0})))` |
| 63 | `II.11.20` | n_rho, p_d, Ef, kb, T | 4.623e+00 | `Times(v_{0}, Divide(v_{1}, Sqrt(v_{4})))` | `(v_{0})*((v_{1}/np.sqrt(v_{4})))` |
| 64 | `II.11.27` | n, alpha, epsilon, Ef | 2.448e-01 | `Divide(Times(v_{0}, v_{1}), Tan(C))` | `((v_{0})*(v_{1})/np.tan(p_{0}))` |
| 65 | `II.11.28` | n, alpha | 4.446e-02 | `Exp(Times(v_{0}, v_{1}))` | `np.exp((v_{0})*(v_{1}))` |
| 66 | `II.13.17` | epsilon, c, I, r | 1.697e-02 | `Divide(Divide(ArcTan(C), v_{3}), v_{1})` | `((np.arctan(p_{0})/v_{3})/v_{1})` |
| 67 | `II.13.23` | rho_c_0, v, c | 1.006e-01 | `Divide(v_{0}, ArcTan(ArcTan(v_{2})))` | `(v_{0}/np.arctan(np.arctan(v_{2})))` |
| 68 | `II.13.34` | rho_c_0, v, c | 2.088e-01 | `Times(v_{1}, Plus(v_{0}, Inv(v_{2})))` | `(v_{1})*((v_{0})+((1/v_{2})))` |
| 69 | `II.15.4` | mom, B, theta | 3.126e+00 | `Times(Cos(v_{2}), Neg(Log(C)))` | `(np.cos(v_{2}))*((-np.log(p_{0})))` |
| 70 | `II.15.5` | p_d, Ef, theta | 3.419e+00 | `Divide(v_{0}, Exp(Cos(v_{2})))` | `(v_{0}/np.exp(np.cos(v_{2})))` |
| 71 | `II.21.32` | q, epsilon, r, v, c | 2.939e-02 | `Divide(Divide(Neg(C), v_{1}), v_{2})` | `(((-p_{0})/v_{1})/v_{2})` |
| 72 | `II.24.17` | omega, c, d | 9.598e-02 | `Subtract(Divide(v_{0}, v_{1}), Inv(v_{0}))` | `((v_{0}/v_{1}))-((1/v_{0}))` |
| 73 | `II.27.16` | epsilon, c, Ef | 5.369e+01 | `Times(v_{1}, Exp(Exp(ArcTan(v_{2}))))` | `(v_{1})*(np.exp(np.exp(np.arctan(v_{2}))))` |
| 75 | `II.34.2a` | q, v, r | 2.544e-01 | `Divide(v_{1}, Plus(v_{2}, v_{2}))` | `(v_{1}/(v_{2})+(v_{2}))` |
| 76 | `II.34.2` | q, v, r | 2.995e+00 | `Times(v_{2}, Times(v_{1}, Sqrt(v_{0})))` | `(v_{2})*((v_{1})*(np.sqrt(v_{0})))` |
| 77 | `II.34.11` | g_, q, B, m | 2.941e+00 | `Times(v_{1}, Divide(v_{0}, Sqrt(v_{3})))` | `(v_{1})*((v_{0}/np.sqrt(v_{3})))` |
| 78 | `II.34.29a` | q, h, m | 1.262e-01 | `Divide(Times(v_{0}, Log(C)), v_{2})` | `((v_{0})*(np.log(p_{0}))/v_{2})` |
| 79 | `II.34.29b` | g_, h, Jz, mom, B | 1.682e+02 | `Times(v_{4}, Divide(Exp(C), v_{1}))` | `(v_{4})*((np.exp(p_{0})/v_{1}))` |
| 80 | `II.35.18` | n_0, kb, T, mom, B | 2.181e-01 | `ArcSin(Divide(v_{0}, Plus(v_{3}, v_{4})))` | `np.arcsin((v_{0}/(v_{3})+(v_{4})))` |
| 81 | `II.35.21` | n_rho, mom, B, kb, T | 2.081e+00 | `Times(v_{0}, Subtract(v_{1}, Log(v_{3})))` | `(v_{0})*((v_{1})-(np.log(v_{3})))` |
| 82 | `II.36.38` | mom, H, kb, T, alpha, epsilon, c, M | 8.275e-01 | `EML(Inv(v_{2}), Divide(v_{3}, v_{0}))` | `(np.exp((1/v_{2})) - np.log((v_{3}/v_{0})))` |
| 83 | `II.38.3` | Y, A, d, x | 7.515e+00 | `Times(v_{3}, Times(v_{1}, Log(v_{0})))` | `(v_{3})*((v_{1})*(np.log(v_{0})))` |
| 84 | `II.38.14` | Y, sigma | 5.823e-02 | `Times(Log(v_{0}), ArcTan(Inv(v_{1})))` | `(np.log(v_{0}))*(np.arctan((1/v_{1})))` |
| 85 | `III.4.32` | h, omega, kb, T | 6.128e+00 | `Divide(Divide(Exp(C), v_{0}), v_{1})` | `((np.exp(p_{0})/v_{0})/v_{1})` |
| 86 | `III.4.33` | h, omega, kb, T | 3.682e-01 | `Subtract(Times(v_{2}, v_{3}), Sin(C))` | `((v_{2})*(v_{3}))-(np.sin(p_{0}))` |
| 87 | `III.7.38` | mom, B, h | 2.101e+01 | `Divide(Divide(v_{0}, ArcSin(C)), v_{2})` | `((v_{0}/np.arcsin(p_{0}))/v_{2})` |
| 88 | `III.8.54` | E_n, t, h | 3.513e-01 | `Divide(v_{0}, ArcSin(ArcSin(ArcSin(v_{0}))))` | `(v_{0}/np.arcsin(np.arcsin(np.arcsin(v_{0}))))` |
| 89 | `III.9.52` | p_d, Ef, t, h, omega, omega_0 | 1.261e+01 | `Times(v_{1}, Divide(Neg(C), v_{3}))` | `(v_{1})*(((-p_{0})/v_{3}))` |
| 91 | `III.12.43` | n, h | 1.895e-01 | `Times(v_{1}, Log(Sqrt(v_{0})))` | `(v_{1})*(np.log(np.sqrt(v_{0})))` |
| 92 | `III.13.18` | E_n, d, k, h | 3.597e+02 | `Times(v_{2}, Times(v_{0}, Exp(v_{1})))` | `(v_{2})*((v_{0})*(np.exp(v_{1})))` |
| 93 | `III.14.14` | I_0, q, Volt, kb, T | 5.294e+00 | `Times(v_{1}, Times(v_{0}, v_{2}))` | `(v_{1})*((v_{0})*(v_{2}))` |
| 94 | `III.15.12` | U, k, d | 4.530e+00 | `Times(v_{0}, ArcCos(Neg(Inv(v_{2}))))` | `(v_{0})*(np.arccos((-(1/v_{2}))))` |
| 95 | `III.15.14` | h, E_n, d | 1.054e-02 | `Divide(Divide(v_{0}, v_{2}), Tan(C))` | `((v_{0}/v_{2})/np.tan(p_{0}))` |
| 96 | `III.15.27` | alpha, n, d | 1.627e+00 | `Times(Log(C), Divide(v_{0}, v_{1}))` | `(np.log(p_{0}))*((v_{0}/v_{1}))` |
| 97 | `III.17.37` | beta, alpha, theta | 2.745e+00 | `Times(Cos(v_{2}), Sqrt(Exp(v_{1})))` | `(np.cos(v_{2}))*(np.sqrt(np.exp(v_{1})))` |
| 99 | `III.21.20` | rho_c_0, q, A_vec, m | 6.827e+00 | `Divide(Neg(C), Divide(v_{3}, v_{1}))` | `((-p_{0})/(v_{3}/v_{1}))` |

---

## 7. 失敗した方程式の Pareto Front 全候補

### `I.6.2` （変数: sigma, theta）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 8.858e-01 | `C` | `p_{0}` |
| 2 | 5.291e-02 | `Log(C)` | `np.log(p_{0})` |
| 3 | 4.191e-02 | `ArcCos(ArcSin(C))` | `np.arccos(np.arcsin(p_{0}))` |
| 4 | 2.835e-02 | `Tan(ArcSin(ArcSin(v_{1})))` | `np.tan(np.arcsin(np.arcsin(v_{1})))` |
| 5 | 2.270e-02 | `Divide(Neg(Tan(C)), v_{1})` | `((-np.tan(p_{0}))/v_{1})` |
| 6 | 2.157e-02 | `Divide(ArcTan(C), ArcSin(Exp(v_{1})))` | `(np.arctan(p_{0})/np.arcsin(np.exp(v_{1})))` |

### `I.6.2b` （変数: sigma, theta, theta1）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 8.073e-01 | `C` | `p_{0}` |
| 2 | 6.710e-02 | `Tan(C)` | `np.tan(p_{0})` |
| 3 | 6.140e-02 | `Tan(Tan(C))` | `np.tan(np.tan(p_{0}))` |
| 4 | 4.297e-02 | `Inv(ArcSin(Exp(v_{0})))` | `(1/np.arcsin(np.exp(v_{0})))` |
| 5 | 3.750e-02 | `Divide(Cos(C), ArcSin(v_{0}))` | `(np.cos(p_{0})/np.arcsin(v_{0}))` |
| 6 | 3.747e-02 | `Sin(Divide(Cos(C), ArcSin(v_{0})))` | `np.sin((np.cos(p_{0})/np.arcsin(v_{0})))` |

### `I.8.14` （変数: x1, x2, y1, y2）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.494e+00 | `C` | `p_{0}` |
| 2 | 1.020e+00 | `Tan(C)` | `np.tan(p_{0})` |
| 3 | 1.017e+00 | `Sqrt(Neg(C))` | `np.sqrt((-p_{0}))` |
| 4 | 1.014e+00 | `Exp(Cos(Cos(v_{1})))` | `np.exp(np.cos(np.cos(v_{1})))` |
| 5 | 9.907e-01 | `Plus(C, Exp(Cos(v_{1})))` | `(p_{0})+(np.exp(np.cos(v_{1})))` |
| 6 | 9.811e-01 | `EML(C, Subtract(C, Cos(v_{1})))` | `(np.exp(p_{0}) - np.log((p_{1})-(np.cos(v_{1}))))` |

### `I.9.18` （変数: m1, m2, G, x1, x2, y1, y2, z1, z2）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 7.235e-01 | `C` | `p_{0}` |
| 2 | 1.306e-01 | `Inv(v_{3})` | `(1/v_{3})` |
| 3 | 1.305e-01 | `ArcSin(Inv(v_{3}))` | `np.arcsin((1/v_{3}))` |
| 4 | 1.177e-01 | `Times(v_{0}, Log(C))` | `(v_{0})*(np.log(p_{0}))` |
| 5 | 1.145e-01 | `Divide(Plus(C, v_{0}), v_{3})` | `((p_{0})+(v_{0})/v_{3})` |
| 6 | 1.038e-01 | `Times(v_{0}, Times(v_{2}, Sin(C)))` | `(v_{0})*((v_{2})*(np.sin(p_{0})))` |

### `I.10.7` （変数: m_0, v, c）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.852e-01 | `v_{0}` | `v_{0}` |
| 4 | 1.128e-01 | `Plus(v_{0}, Inv(v_{2}))` | `(v_{0})+((1/v_{2}))` |
| 5 | 8.658e-02 | `Divide(v_{0}, ArcTan(ArcTan(v_{2})))` | `(v_{0}/np.arctan(np.arctan(v_{2})))` |

### `I.11.19` （変数: x1, x2, x3, y1, y2, y3）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 2.558e+01 | `v_{2}` | `v_{2}` |
| 2 | 1.142e+01 | `Neg(C)` | `(-p_{0})` |
| 3 | 9.036e+00 | `Exp(Tan(C))` | `np.exp(np.tan(p_{0}))` |
| 4 | 8.592e+00 | `Plus(v_{4}, Neg(C))` | `(v_{4})+((-p_{0}))` |
| 5 | 8.111e+00 | `Plus(v_{4}, Plus(C, v_{1}))` | `(v_{4})+((p_{0})+(v_{1}))` |
| 6 | 6.961e+00 | `Plus(Neg(C), Times(v_{1}, v_{4}))` | `((-p_{0}))+((v_{1})*(v_{4}))` |

### `I.12.2` （変数: q1, q2, epsilon, r）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 9.485e-01 | `C` | `p_{0}` |
| 2 | 8.556e-02 | `Sin(C)` | `np.sin(p_{0})` |
| 3 | 8.164e-02 | `Exp(Neg(v_{3}))` | `np.exp((-v_{3}))` |
| 4 | 7.032e-02 | `Divide(Log(C), v_{3})` | `(np.log(p_{0})/v_{3})` |
| 5 | 5.852e-02 | `Divide(Inv(v_{2}), Exp(v_{3}))` | `((1/v_{2})/np.exp(v_{3}))` |
| 6 | 4.236e-02 | `Divide(Inv(v_{2}), EML(v_{3}, v_{0}))` | `((1/v_{2})/(np.exp(v_{3}) - np.log(v_{0})))` |

### `I.12.4` （変数: q1, epsilon, r）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 9.811e-01 | `C` | `p_{0}` |
| 2 | 2.581e-02 | `Log(C)` | `np.log(p_{0})` |
| 3 | 2.537e-02 | `Tan(Log(C))` | `np.tan(np.log(p_{0}))` |
| 4 | 2.094e-02 | `Divide(Log(C), v_{2})` | `(np.log(p_{0})/v_{2})` |
| 5 | 1.720e-02 | `Divide(Sin(C), Exp(v_{2}))` | `(np.sin(p_{0})/np.exp(v_{2}))` |
| 6 | 1.525e-02 | `Divide(Divide(ArcTan(C), v_{1}), v_{2})` | `((np.arctan(p_{0})/v_{1})/v_{2})` |

### `I.12.11` （変数: q, Ef, B, v, theta）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 2.723e+01 | `v_{0}` | `v_{0}` |
| 2 | 2.603e+01 | `Log(C)` | `np.log(p_{0})` |
| 3 | 2.530e+01 | `Subtract(C, v_{4})` | `(p_{0})-(v_{4})` |
| 4 | 2.236e+01 | `Divide(Exp(C), v_{4})` | `(np.exp(p_{0})/v_{4})` |
| 5 | 1.849e+01 | `Times(Sin(v_{4}), Tan(C))` | `(np.sin(v_{4}))*(np.tan(p_{0}))` |
| 6 | 1.816e+01 | `Times(v_{0}, Cos(Log(Neg(v_{4}))))` | `(v_{0})*(np.cos(np.log((-v_{4}))))` |

### `I.13.4` （変数: m, v, u, w）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 5.062e+01 | `v_{0}` | `v_{0}` |
| 2 | 2.921e+01 | `Neg(C)` | `(-p_{0})` |
| 3 | 2.712e+01 | `Sqrt(Neg(C))` | `np.sqrt((-p_{0}))` |
| 4 | 2.014e+01 | `Times(v_{0}, Log(C))` | `(v_{0})*(np.log(p_{0}))` |
| 6 | 1.801e+01 | `Times(v_{0}, Divide(v_{1}, Sin(C)))` | `(v_{0})*((v_{1}/np.sin(p_{0})))` |

### `I.13.12` （変数: m1, m2, r1, r2, G）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 9.613e+00 | `v_{2}` | `v_{2}` |
| 2 | 9.367e+00 | `Sin(v_{3})` | `np.sin(v_{3})` |
| 3 | 8.589e+00 | `Subtract(v_{2}, v_{3})` | `(v_{2})-(v_{3})` |
| 5 | 6.314e+00 | `Times(v_{1}, Subtract(v_{2}, v_{3}))` | `(v_{1})*((v_{2})-(v_{3}))` |

### `I.14.4` （変数: k_spring, x）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.746e+01 | `v_{1}` | `v_{1}` |
| 2 | 1.286e+01 | `Log(C)` | `np.log(p_{0})` |
| 3 | 9.559e+00 | `Times(v_{1}, v_{1})` | `(v_{1})*(v_{1})` |
| 4 | 8.839e+00 | `Times(v_{1}, Exp(C))` | `(v_{1})*(np.exp(p_{0}))` |
| 5 | 2.131e+00 | `Times(v_{0}, Sqrt(Exp(v_{1})))` | `(v_{0})*(np.sqrt(np.exp(v_{1})))` |
| 6 | 1.754e+00 | `Times(v_{0}, Sqrt(EML(v_{1}, v_{0})))` | `(v_{0})*(np.sqrt((np.exp(v_{1}) - np.log(v_{0}))))` |

### `I.15.3x` （変数: x, u, c, t）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 2.218e+00 | `v_{0}` | `v_{0}` |
| 2 | 1.696e+00 | `Exp(C)` | `np.exp(p_{0})` |
| 3 | 6.606e-01 | `Subtract(v_{0}, C)` | `(v_{0})-(p_{0})` |
| 4 | 6.324e-01 | `Subtract(v_{0}, Tan(C))` | `(v_{0})-(np.tan(p_{0}))` |
| 5 | 2.101e-01 | `Subtract(v_{0}, Times(v_{1}, v_{3}))` | `(v_{0})-((v_{1})*(v_{3}))` |

### `I.15.3t` （変数: x, c, u, t）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.102e-01 | `v_{3}` | `v_{3}` |
| 4 | 1.053e-01 | `Log(EML(v_{3}, v_{2}))` | `np.log((np.exp(v_{3}) - np.log(v_{2})))` |
| 6 | 1.009e-01 | `Subtract(v_{3}, Divide(v_{0}, Tan(C)))` | `(v_{3})-((v_{0}/np.tan(p_{0})))` |

### `I.15.1` （変数: m_0, v, c）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 2.128e+00 | `v_{0}` | `v_{0}` |
| 2 | 1.865e+00 | `Exp(v_{1})` | `np.exp(v_{1})` |
| 3 | 3.448e-01 | `Times(v_{0}, v_{1})` | `(v_{0})*(v_{1})` |
| 5 | 2.897e-01 | `Divide(v_{1}, ArcTan(Inv(v_{0})))` | `(v_{1}/np.arctan((1/v_{0})))` |
| 6 | 1.952e-01 | `Times(v_{1}, Plus(v_{0}, Inv(v_{2})))` | `(v_{1})*((v_{0})+((1/v_{2})))` |

### `I.16.6` （変数: c, v, u）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 4.098e-01 | `v_{0}` | `v_{0}` |
| 4 | 3.680e-01 | `Times(v_{0}, Cos(C))` | `(v_{0})*(np.cos(p_{0}))` |
| 5 | 3.590e-01 | `Times(v_{0}, Sin(ArcTan(v_{1})))` | `(v_{0})*(np.sin(np.arctan(v_{1})))` |

### `I.18.4` （変数: m1, m2, r1, r2）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 8.334e-01 | `v_{3}` | `v_{3}` |
| 3 | 7.116e-01 | `Exp(ArcTan(v_{3}))` | `np.exp(np.arctan(v_{3}))` |
| 4 | 3.097e-01 | `Sqrt(Times(v_{2}, v_{3}))` | `np.sqrt((v_{2})*(v_{3}))` |
| 6 | 2.200e-01 | `Times(Cos(C), Plus(v_{2}, v_{3}))` | `(np.cos(p_{0}))*((v_{2})+(v_{3}))` |

### `I.24.6` （変数: m, omega, omega_0, x）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 2.154e+01 | `v_{3}` | `v_{3}` |
| 2 | 1.474e+01 | `Log(C)` | `np.log(p_{0})` |
| 3 | 1.420e+01 | `Plus(C, v_{3})` | `(p_{0})+(v_{3})` |
| 4 | 8.562e+00 | `Times(v_{0}, Exp(v_{3}))` | `(v_{0})*(np.exp(v_{3}))` |
| 6 | 7.933e+00 | `Times(v_{0}, EML(v_{3}, Tan(v_{2})))` | `(v_{0})*((np.exp(v_{3}) - np.log(np.tan(v_{2}))))` |

### `I.27.6` （変数: d1, d2, n）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 4.319e-01 | `C` | `p_{0}` |
| 2 | 3.579e-01 | `ArcTan(C)` | `np.arctan(p_{0})` |
| 3 | 2.838e-01 | `ArcSin(ArcSin(v_{2}))` | `np.arcsin(np.arcsin(v_{2}))` |
| 4 | 1.682e-01 | `ArcTan(Divide(v_{1}, v_{2}))` | `np.arctan((v_{1}/v_{2}))` |

### `I.29.16` （変数: x1, x2, theta1, theta2）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.906e+00 | `v_{1}` | `v_{1}` |
| 2 | 1.817e+00 | `Exp(C)` | `np.exp(p_{0})` |
| 3 | 1.735e+00 | `Exp(ArcTan(v_{1}))` | `np.exp(np.arctan(v_{1}))` |
| 4 | 1.719e+00 | `Exp(Sqrt(Sqrt(v_{1})))` | `np.exp(np.sqrt(np.sqrt(v_{1})))` |
| 5 | 1.671e+00 | `Plus(Sqrt(v_{0}), Sqrt(v_{1}))` | `(np.sqrt(v_{0}))+(np.sqrt(v_{1}))` |
| 6 | 1.662e+00 | `Times(Tan(C), Plus(v_{0}, v_{1}))` | `(np.tan(p_{0}))*((v_{0})+(v_{1}))` |

### `I.30.3` （変数: Int_0, theta, n）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 2.409e+00 | `v_{0}` | `v_{0}` |
| 3 | 2.349e+00 | `Subtract(v_{0}, C)` | `(v_{0})-(p_{0})` |
| 4 | 2.170e+00 | `Divide(v_{0}, ArcTan(v_{1}))` | `(v_{0}/np.arctan(v_{1}))` |
| 5 | 1.944e+00 | `Times(v_{0}, Exp(Cos(v_{1})))` | `(v_{0})*(np.exp(np.cos(v_{1})))` |
| 6 | 1.869e+00 | `Times(v_{0}, Exp(Sqrt(Cos(v_{1}))))` | `(v_{0})*(np.exp(np.sqrt(np.cos(v_{1}))))` |

### `I.32.5` （変数: q, a, epsilon, c）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.108e+00 | `C` | `p_{0}` |
| 2 | 7.817e-01 | `Inv(v_{3})` | `(1/v_{3})` |
| 3 | 7.737e-01 | `Tan(Inv(v_{3}))` | `np.tan((1/v_{3}))` |
| 4 | 6.926e-01 | `Divide(v_{1}, Exp(v_{3}))` | `(v_{1}/np.exp(v_{3}))` |
| 5 | 5.779e-01 | `Tan(Divide(Log(v_{0}), v_{3}))` | `np.tan((np.log(v_{0})/v_{3}))` |

### `I.32.17` （変数: epsilon, c, Ef, r, omega, omega_0）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 5.710e+00 | `v_{4}` | `v_{4}` |
| 2 | 5.369e+00 | `Exp(v_{4})` | `np.exp(v_{4})` |
| 3 | 5.052e+00 | `EML(v_{4}, v_{5})` | `(np.exp(v_{4}) - np.log(v_{5}))` |
| 4 | 4.907e+00 | `EML(v_{4}, Tan(v_{5}))` | `(np.exp(v_{4}) - np.log(np.tan(v_{5})))` |
| 5 | 4.519e+00 | `Exp(Divide(Exp(v_{4}), v_{5}))` | `np.exp((np.exp(v_{4})/v_{5}))` |
| 6 | 4.474e+00 | `Divide(Exp(v_{4}), Subtract(v_{5}, C))` | `(np.exp(v_{4})/(v_{5})-(p_{0}))` |

### `I.34.8` （変数: q, v, B, p）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.258e+01 | `v_{1}` | `v_{1}` |
| 2 | 1.056e+01 | `Log(C)` | `np.log(p_{0})` |
| 3 | 8.409e+00 | `Times(v_{0}, v_{1})` | `(v_{0})*(v_{1})` |
| 5 | 7.525e+00 | `Exp(EML(ArcTan(v_{1}), v_{3}))` | `np.exp((np.exp(np.arctan(v_{1})) - np.log(v_{3})))` |
| 6 | 7.028e+00 | `Times(v_{1}, Times(v_{0}, Log(v_{2})))` | `(v_{1})*((v_{0})*(np.log(v_{2})))` |

### `I.34.1` （変数: c, v, omega_0）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.477e+00 | `v_{2}` | `v_{2}` |
| 3 | 8.915e-01 | `Plus(v_{1}, v_{2})` | `(v_{1})+(v_{2})` |
| 4 | 6.940e-01 | `EML(Sqrt(v_{2}), v_{0})` | `(np.exp(np.sqrt(v_{2})) - np.log(v_{0}))` |
| 6 | 6.098e-01 | `EML(Sqrt(v_{2}), Subtract(v_{0}, v_{1}))` | `(np.exp(np.sqrt(v_{2})) - np.log((v_{0})-(v_{1})))` |

### `I.34.14` （変数: c, v, omega_0）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.130e+00 | `v_{2}` | `v_{2}` |
| 3 | 6.172e-01 | `Plus(C, v_{2})` | `(p_{0})+(v_{2})` |
| 4 | 5.064e-01 | `Divide(v_{2}, Cos(C))` | `(v_{2}/np.cos(p_{0}))` |
| 5 | 4.590e-01 | `Times(v_{2}, ArcTan(Exp(v_{1})))` | `(v_{2})*(np.arctan(np.exp(v_{1})))` |
| 6 | 2.866e-01 | `Times(v_{2}, EML(C, ArcCos(v_{0})))` | `(v_{2})*((np.exp(p_{0}) - np.log(np.arccos(v_{0}))))` |

### `I.34.27` （変数: omega, h）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 9.142e-01 | `C` | `p_{0}` |
| 2 | 6.687e-01 | `Sqrt(v_{0})` | `np.sqrt(v_{0})` |
| 4 | 5.931e-01 | `Log(Sqrt(Exp(v_{0})))` | `np.log(np.sqrt(np.exp(v_{0})))` |
| 5 | 1.866e-01 | `Times(v_{0}, Log(Sqrt(v_{1})))` | `(v_{0})*(np.log(np.sqrt(v_{1})))` |

### `I.37.4` （変数: I1, I2, delta）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 2.941e+00 | `v_{1}` | `v_{1}` |
| 2 | 2.923e+00 | `Exp(C)` | `np.exp(p_{0})` |
| 3 | 2.815e+00 | `EML(C, v_{2})` | `(np.exp(p_{0}) - np.log(v_{2}))` |
| 4 | 2.422e+00 | `EML(C, Cos(v_{2}))` | `(np.exp(p_{0}) - np.log(np.cos(v_{2})))` |
| 5 | 1.436e+00 | `Exp(ArcCos(Neg(Cos(v_{2}))))` | `np.exp(np.arccos((-np.cos(v_{2}))))` |

### `I.39.1` （変数: pr, V）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.204e+01 | `v_{1}` | `v_{1}` |
| 2 | 7.357e+00 | `Log(C)` | `np.log(p_{0})` |
| 3 | 4.985e+00 | `Times(v_{0}, v_{1})` | `(v_{0})*(v_{1})` |
| 5 | 2.341e+00 | `Plus(v_{1}, Times(v_{0}, v_{1}))` | `(v_{1})+((v_{0})*(v_{1}))` |
| 6 | 7.058e-01 | `Times(v_{1}, Times(v_{0}, ArcTan(C)))` | `(v_{1})*((v_{0})*(np.arctan(p_{0})))` |

### `I.39.11` （変数: gamma, pr, V）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 2.726e+00 | `v_{2}` | `v_{2}` |
| 3 | 2.484e+00 | `Plus(C, v_{2})` | `(p_{0})+(v_{2})` |
| 4 | 2.232e+00 | `Times(v_{2}, Sqrt(v_{1}))` | `(v_{2})*(np.sqrt(v_{1}))` |
| 5 | 1.911e+00 | `Times(v_{2}, Divide(v_{1}, v_{0}))` | `(v_{2})*((v_{1}/v_{0}))` |
| 6 | 1.547e+00 | `Times(v_{1}, Divide(v_{2}, Sqrt(v_{0})))` | `(v_{1})*((v_{2}/np.sqrt(v_{0})))` |

### `I.39.22` （変数: n, T, V, kb）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.247e+01 | `v_{0}` | `v_{0}` |
| 2 | 1.067e+01 | `Log(C)` | `np.log(p_{0})` |
| 3 | 8.345e+00 | `Times(v_{0}, v_{3})` | `(v_{0})*(v_{3})` |
| 5 | 7.772e+00 | `Plus(v_{1}, Times(v_{0}, v_{3}))` | `(v_{1})+((v_{0})*(v_{3}))` |
| 6 | 6.956e+00 | `Times(v_{3}, Times(v_{0}, Log(v_{1})))` | `(v_{3})*((v_{0})*(np.log(v_{1})))` |

### `I.40.1` （変数: n_0, m, x, T, g, kb）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 8.520e-01 | `C` | `p_{0}` |
| 2 | 6.063e-01 | `Inv(v_{1})` | `(1/v_{1})` |
| 3 | 6.053e-01 | `ArcSin(Inv(v_{4}))` | `np.arcsin((1/v_{4}))` |
| 4 | 5.651e-01 | `Divide(Log(v_{3}), v_{1})` | `(np.log(v_{3})/v_{1})` |
| 5 | 4.804e-01 | `Divide(v_{0}, Times(v_{1}, v_{4}))` | `(v_{0}/(v_{1})*(v_{4}))` |

### `I.41.16` （変数: omega, T, h, kb, c）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 3.091e+00 | `v_{0}` | `v_{0}` |
| 2 | 2.941e+00 | `Sqrt(v_{0})` | `np.sqrt(v_{0})` |
| 3 | 2.555e+00 | `Divide(v_{0}, v_{4})` | `(v_{0}/v_{4})` |
| 4 | 2.536e+00 | `Exp(ArcCos(Log(v_{4})))` | `np.exp(np.arccos(np.log(v_{4})))` |
| 5 | 1.940e+00 | `Divide(v_{0}, Plus(C, v_{4}))` | `(v_{0}/(p_{0})+(v_{4}))` |
| 6 | 1.885e+00 | `Times(v_{0}, Inv(Plus(C, v_{4})))` | `(v_{0})*((1/(p_{0})+(v_{4})))` |

### `I.43.16` （変数: mu_drift, q, Volt, d）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.339e+01 | `v_{2}` | `v_{2}` |
| 2 | 1.119e+01 | `Log(C)` | `np.log(p_{0})` |
| 3 | 9.132e+00 | `Times(v_{0}, v_{1})` | `(v_{0})*(v_{1})` |
| 5 | 7.418e+00 | `Exp(EML(ArcTan(v_{0}), v_{3}))` | `np.exp((np.exp(np.arctan(v_{0})) - np.log(v_{3})))` |
| 6 | 7.272e+00 | `EML(EML(ArcTan(v_{0}), v_{3}), v_{0})` | `(np.exp((np.exp(np.arctan(v_{0})) - np.log(v_{3}))) ...` |

### `I.43.43` （変数: gamma, kb, A, v）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.690e+00 | `C` | `p_{0}` |
| 2 | 1.435e+00 | `Sqrt(v_{3})` | `np.sqrt(v_{3})` |
| 3 | 1.250e+00 | `Divide(v_{3}, v_{2})` | `(v_{3}/v_{2})` |
| 4 | 1.205e+00 | `Divide(v_{3}, Sqrt(v_{2}))` | `(v_{3}/np.sqrt(v_{2}))` |
| 5 | 1.153e+00 | `Divide(Plus(v_{1}, v_{3}), v_{0})` | `((v_{1})+(v_{3})/v_{0})` |
| 6 | 1.008e+00 | `Times(Log(v_{1}), Divide(v_{3}, v_{2}))` | `(np.log(v_{1}))*((v_{3}/v_{2}))` |

### `I.44.4` （変数: n, kb, T, V1, V2）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 2.037e+01 | `v_{4}` | `v_{4}` |
| 3 | 1.963e+01 | `Subtract(v_{4}, v_{3})` | `(v_{4})-(v_{3})` |
| 4 | 1.738e+01 | `Exp(ArcCos(Neg(v_{3})))` | `np.exp(np.arccos((-v_{3})))` |
| 5 | 1.657e+01 | `Times(v_{1}, Subtract(v_{4}, v_{3}))` | `(v_{1})*((v_{4})-(v_{3}))` |
| 6 | 1.385e+01 | `Times(Log(C), Subtract(v_{4}, v_{3}))` | `(np.log(p_{0}))*((v_{4})-(v_{3}))` |

### `I.47.23` （変数: gamma, pr, rho）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.029e+00 | `C` | `p_{0}` |
| 2 | 5.834e-01 | `Sqrt(v_{0})` | `np.sqrt(v_{0})` |
| 3 | 5.446e-01 | `EML(C, v_{2})` | `(np.exp(p_{0}) - np.log(v_{2}))` |
| 4 | 5.080e-01 | `Log(Plus(v_{0}, v_{1}))` | `np.log((v_{0})+(v_{1}))` |
| 5 | 4.069e-01 | `EML(ArcTan(Sqrt(v_{1})), v_{2})` | `(np.exp(np.arctan(np.sqrt(v_{1}))) - np.log(v_{2}))` |
| 6 | 2.802e-01 | `Divide(Plus(v_{0}, v_{1}), ArcSin(v_{2}))` | `((v_{0})+(v_{1})/np.arcsin(v_{2}))` |

### `I.48.2` （変数: m, v, c）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.622e+02 | `v_{2}` | `v_{2}` |
| 2 | 1.064e+02 | `Neg(C)` | `(-p_{0})` |
| 3 | 1.042e+02 | `Plus(C, v_{2})` | `(p_{0})+(v_{2})` |
| 4 | 8.442e+01 | `Times(v_{2}, Log(C))` | `(v_{2})*(np.log(p_{0}))` |
| 5 | 4.399e+00 | `Times(v_{0}, Times(v_{2}, v_{2}))` | `(v_{0})*((v_{2})*(v_{2}))` |

### `I.50.26` （変数: x1, omega, t, alpha）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.928e+00 | `v_{3}` | `v_{3}` |
| 3 | 1.893e+00 | `Subtract(v_{3}, C)` | `(v_{3})-(p_{0})` |
| 4 | 1.833e+00 | `Times(v_{2}, Log(v_{3}))` | `(v_{2})*(np.log(v_{3}))` |
| 5 | 1.793e+00 | `Subtract(v_{1}, Subtract(C, v_{3}))` | `(v_{1})-((p_{0})-(v_{3}))` |
| 6 | 1.790e+00 | `Subtract(Plus(v_{1}, v_{3}), Log(C))` | `((v_{1})+(v_{3}))-(np.log(p_{0}))` |

### `II.2.42` （変数: kappa, T1, T2, A, d）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 7.209e+00 | `v_{2}` | `v_{2}` |
| 2 | 6.920e+00 | `Sin(v_{1})` | `np.sin(v_{1})` |
| 3 | 6.087e+00 | `Subtract(v_{2}, v_{1})` | `(v_{2})-(v_{1})` |
| 5 | 3.924e+00 | `Times(v_{3}, Subtract(v_{2}, v_{1}))` | `(v_{3})*((v_{2})-(v_{1}))` |

### `II.3.24` （変数: Pwr, r）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 9.557e-01 | `C` | `p_{0}` |
| 2 | 5.589e-02 | `Log(C)` | `np.log(p_{0})` |
| 3 | 5.458e-02 | `Inv(Log(C))` | `(1/np.log(p_{0}))` |
| 4 | 3.675e-02 | `Divide(Cos(C), v_{1})` | `(np.cos(p_{0})/v_{1})` |
| 5 | 2.696e-02 | `Tan(Plus(C, ArcCos(v_{1})))` | `np.tan((p_{0})+(np.arccos(v_{1})))` |
| 6 | 1.092e-02 | `Divide(Log(Sqrt(v_{0})), Exp(v_{1}))` | `(np.log(np.sqrt(v_{0}))/np.exp(v_{1}))` |

### `II.4.23` （変数: q, epsilon, r）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 9.640e-01 | `C` | `p_{0}` |
| 2 | 3.215e-02 | `Log(C)` | `np.log(p_{0})` |
| 3 | 3.084e-02 | `Tan(Log(C))` | `np.tan(np.log(p_{0}))` |
| 4 | 2.627e-02 | `Divide(Cos(C), v_{1})` | `(np.cos(p_{0})/v_{1})` |
| 5 | 2.595e-02 | `Divide(Inv(v_{1}), Exp(v_{2}))` | `((1/v_{1})/np.exp(v_{2}))` |
| 6 | 1.775e-02 | `Divide(Divide(Neg(C), v_{1}), v_{2})` | `(((-p_{0})/v_{1})/v_{2})` |

### `II.6.11` （変数: epsilon, p_d, theta, r）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.011e+00 | `C` | `p_{0}` |
| 2 | 2.064e-02 | `Log(C)` | `np.log(p_{0})` |
| 3 | 2.049e-02 | `Tan(Log(C))` | `np.tan(np.log(p_{0}))` |
| 4 | 1.842e-02 | `Times(v_{2}, Sin(C))` | `(v_{2})*(np.sin(p_{0}))` |
| 5 | 1.657e-02 | `Divide(Cos(v_{2}), Log(C))` | `(np.cos(v_{2})/np.log(p_{0}))` |
| 6 | 1.514e-02 | `Times(Cos(v_{2}), Log(ArcSin(C)))` | `(np.cos(v_{2}))*(np.log(np.arcsin(p_{0})))` |

### `II.6.15a` （変数: epsilon, p_d, r, x, y, z）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 8.912e-01 | `C` | `p_{0}` |
| 2 | 3.622e-01 | `Log(C)` | `np.log(p_{0})` |
| 3 | 2.491e-01 | `Sqrt(Cos(v_{2}))` | `np.sqrt(np.cos(v_{2}))` |
| 4 | 2.318e-01 | `Tan(Exp(ArcCos(v_{2})))` | `np.tan(np.exp(np.arccos(v_{2})))` |
| 6 | 1.897e-01 | `Divide(Inv(v_{0}), Log(Tan(v_{2})))` | `((1/v_{0})/np.log(np.tan(v_{2})))` |

### `II.6.15b` （変数: epsilon, p_d, theta, r）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.011e+00 | `C` | `p_{0}` |
| 2 | 3.158e-02 | `Log(C)` | `np.log(p_{0})` |
| 3 | 3.147e-02 | `Tan(Log(C))` | `np.tan(np.log(p_{0}))` |
| 4 | 3.011e-02 | `Times(v_{2}, Log(C))` | `(v_{2})*(np.log(p_{0}))` |
| 5 | 2.784e-02 | `Divide(Cos(v_{2}), Log(C))` | `(np.cos(v_{2})/np.log(p_{0}))` |
| 6 | 2.526e-02 | `Divide(Cos(v_{2}), Exp(Exp(v_{3})))` | `(np.cos(v_{2})/np.exp(np.exp(v_{3})))` |

### `II.8.7` （変数: q, epsilon, d）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 9.247e-01 | `C` | `p_{0}` |
| 2 | 1.019e-01 | `Log(C)` | `np.log(p_{0})` |
| 3 | 9.952e-02 | `ArcSin(ArcSin(C))` | `np.arcsin(np.arcsin(p_{0}))` |
| 4 | 8.805e-02 | `Inv(Log(Neg(v_{0})))` | `(1/np.log((-v_{0})))` |
| 5 | 7.053e-02 | `Divide(Log(ArcTan(v_{0})), v_{2})` | `(np.log(np.arctan(v_{0}))/v_{2})` |
| 6 | 6.529e-02 | `Times(v_{0}, Divide(Log(C), v_{2}))` | `(v_{0})*((np.log(p_{0})/v_{2}))` |

### `II.8.31` （変数: epsilon, Ef）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.750e+01 | `v_{1}` | `v_{1}` |
| 2 | 1.292e+01 | `Log(C)` | `np.log(p_{0})` |
| 3 | 9.818e+00 | `Times(v_{1}, v_{1})` | `(v_{1})*(v_{1})` |
| 4 | 8.745e+00 | `Times(v_{1}, Exp(C))` | `(v_{1})*(np.exp(p_{0}))` |
| 5 | 2.037e+00 | `Times(v_{0}, Sqrt(Exp(v_{1})))` | `(v_{0})*(np.sqrt(np.exp(v_{1})))` |
| 6 | 1.754e+00 | `Times(v_{0}, Sqrt(EML(v_{1}, v_{0})))` | `(v_{0})*(np.sqrt((np.exp(v_{1}) - np.log(v_{0}))))` |

### `II.10.9` （変数: sigma_den, epsilon, chi）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 7.096e-01 | `C` | `p_{0}` |
| 2 | 2.055e-01 | `Inv(v_{1})` | `(1/v_{1})` |
| 3 | 1.950e-01 | `Cos(ArcTan(v_{1}))` | `np.cos(np.arctan(v_{1}))` |
| 4 | 1.652e-01 | `Divide(Log(v_{0}), v_{1})` | `(np.log(v_{0})/v_{1})` |
| 5 | 1.406e-01 | `Divide(v_{0}, ArcSin(Exp(v_{1})))` | `(v_{0}/np.arcsin(np.exp(v_{1})))` |
| 6 | 7.008e-02 | `Divide(Divide(v_{0}, v_{1}), ArcSin(v_{2}))` | `((v_{0}/v_{1})/np.arcsin(v_{2}))` |

### `II.11.3` （変数: q, Ef, m, omega_0, omega）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 8.274e-01 | `C` | `p_{0}` |
| 2 | 1.335e-01 | `Inv(v_{3})` | `(1/v_{3})` |
| 3 | 1.143e-01 | `Exp(Neg(v_{2}))` | `np.exp((-v_{2}))` |
| 4 | 1.057e-01 | `Divide(Log(v_{0}), v_{3})` | `(np.log(v_{0})/v_{3})` |
| 5 | 9.011e-02 | `Divide(EML(C, v_{3}), v_{2})` | `((np.exp(p_{0}) - np.log(v_{3}))/v_{2})` |
| 6 | 7.792e-02 | `Divide(Divide(v_{0}, v_{3}), ArcSin(v_{2}))` | `((v_{0}/v_{3})/np.arcsin(v_{2}))` |

### `II.11.17` （変数: n_0, kb, T, theta, p_d, Ef）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.649e+00 | `C` | `p_{0}` |
| 2 | 1.577e+00 | `Sin(v_{3})` | `np.sin(v_{3})` |
| 3 | 1.245e+00 | `Exp(ArcSin(v_{3}))` | `np.exp(np.arcsin(v_{3}))` |
| 4 | 1.208e+00 | `Exp(Subtract(v_{0}, v_{3}))` | `np.exp((v_{0})-(v_{3}))` |
| 5 | 1.032e+00 | `Times(v_{0}, Subtract(C, v_{3}))` | `(v_{0})*((p_{0})-(v_{3}))` |
| 6 | 1.029e+00 | `Times(v_{0}, Neg(Subtract(v_{3}, C)))` | `(v_{0})*((-(v_{3})-(p_{0})))` |

### `II.11.20` （変数: n_rho, p_d, Ef, kb, T）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 6.330e+00 | `v_{1}` | `v_{1}` |
| 3 | 5.641e+00 | `Sqrt(Exp(v_{1}))` | `np.sqrt(np.exp(v_{1}))` |
| 4 | 5.391e+00 | `Times(v_{1}, Sqrt(v_{0}))` | `(v_{1})*(np.sqrt(v_{0}))` |
| 5 | 4.932e+00 | `Times(v_{0}, Divide(v_{1}, v_{4}))` | `(v_{0})*((v_{1}/v_{4}))` |
| 6 | 4.623e+00 | `Times(v_{0}, Divide(v_{1}, Sqrt(v_{4})))` | `(v_{0})*((v_{1}/np.sqrt(v_{4})))` |

### `II.11.27` （変数: n, alpha, epsilon, Ef）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 5.792e-01 | `v_{0}` | `v_{0}` |
| 2 | 5.475e-01 | `Tan(v_{0})` | `np.tan(v_{0})` |
| 3 | 5.206e-01 | `Plus(v_{0}, v_{1})` | `(v_{0})+(v_{1})` |
| 4 | 3.777e-01 | `Times(v_{0}, Exp(v_{1}))` | `(v_{0})*(np.exp(v_{1}))` |
| 5 | 3.573e-01 | `Times(v_{1}, Plus(v_{0}, v_{0}))` | `(v_{1})*((v_{0})+(v_{0}))` |
| 6 | 2.448e-01 | `Divide(Times(v_{0}, v_{1}), Tan(C))` | `((v_{0})*(v_{1})/np.tan(p_{0}))` |

### `II.11.28` （変数: n, alpha）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 4.362e-01 | `C` | `p_{0}` |
| 2 | 3.075e-01 | `Tan(C)` | `np.tan(p_{0})` |
| 3 | 2.267e-01 | `Sqrt(Exp(v_{1}))` | `np.sqrt(np.exp(v_{1}))` |
| 4 | 4.446e-02 | `Exp(Times(v_{0}, v_{1}))` | `np.exp((v_{0})*(v_{1}))` |

### `II.13.17` （変数: epsilon, c, I, r）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 9.836e-01 | `C` | `p_{0}` |
| 2 | 2.534e-02 | `Log(C)` | `np.log(p_{0})` |
| 3 | 2.498e-02 | `Neg(Cos(C))` | `(-np.cos(p_{0}))` |
| 4 | 2.208e-02 | `Divide(Log(C), v_{1})` | `(np.log(p_{0})/v_{1})` |
| 5 | 1.930e-02 | `Divide(Sin(C), Exp(v_{1}))` | `(np.sin(p_{0})/np.exp(v_{1}))` |
| 6 | 1.697e-02 | `Divide(Divide(ArcTan(C), v_{3}), v_{1})` | `((np.arctan(p_{0})/v_{3})/v_{1})` |

### `II.13.23` （変数: rho_c_0, v, c）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 2.133e-01 | `v_{0}` | `v_{0}` |
| 4 | 1.303e-01 | `Plus(v_{0}, Inv(v_{2}))` | `(v_{0})+((1/v_{2}))` |
| 5 | 1.006e-01 | `Divide(v_{0}, ArcTan(ArcTan(v_{2})))` | `(v_{0}/np.arctan(np.arctan(v_{2})))` |

### `II.13.34` （変数: rho_c_0, v, c）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 2.225e+00 | `v_{0}` | `v_{0}` |
| 2 | 1.988e+00 | `Exp(v_{1})` | `np.exp(v_{1})` |
| 3 | 3.559e-01 | `Times(v_{0}, v_{1})` | `(v_{0})*(v_{1})` |
| 5 | 3.084e-01 | `Divide(v_{1}, ArcTan(Inv(v_{0})))` | `(v_{1}/np.arctan((1/v_{0})))` |
| 6 | 2.088e-01 | `Times(v_{1}, Plus(v_{0}, Inv(v_{2})))` | `(v_{1})*((v_{0})+((1/v_{2})))` |

### `II.15.4` （変数: mom, B, theta）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 4.844e+00 | `v_{1}` | `v_{1}` |
| 3 | 4.800e+00 | `Plus(C, v_{1})` | `(p_{0})+(v_{1})` |
| 4 | 4.287e+00 | `EML(C, Tan(v_{2}))` | `(np.exp(p_{0}) - np.log(np.tan(v_{2})))` |
| 5 | 3.412e+00 | `Divide(v_{0}, Exp(Cos(v_{2})))` | `(v_{0}/np.exp(np.cos(v_{2})))` |
| 6 | 3.126e+00 | `Times(Cos(v_{2}), Neg(Log(C)))` | `(np.cos(v_{2}))*((-np.log(p_{0})))` |

### `II.15.5` （変数: p_d, Ef, theta）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 5.128e+00 | `v_{0}` | `v_{0}` |
| 3 | 4.950e+00 | `Plus(C, v_{0})` | `(p_{0})+(v_{0})` |
| 4 | 4.505e+00 | `EML(C, Tan(v_{2}))` | `(np.exp(p_{0}) - np.log(np.tan(v_{2})))` |
| 5 | 3.419e+00 | `Divide(v_{0}, Exp(Cos(v_{2})))` | `(v_{0}/np.exp(np.cos(v_{2})))` |

### `II.21.32` （変数: q, epsilon, r, v, c）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 9.490e-01 | `C` | `p_{0}` |
| 2 | 4.815e-02 | `Cos(C)` | `np.cos(p_{0})` |
| 3 | 4.792e-02 | `Tan(Cos(C))` | `np.tan(np.cos(p_{0}))` |
| 4 | 3.933e-02 | `Divide(Cos(C), v_{1})` | `(np.cos(p_{0})/v_{1})` |
| 5 | 3.576e-02 | `Divide(Inv(v_{2}), Exp(v_{1}))` | `((1/v_{2})/np.exp(v_{1}))` |
| 6 | 2.939e-02 | `Divide(Divide(Neg(C), v_{1}), v_{2})` | `(((-p_{0})/v_{1})/v_{2})` |

### `II.24.17` （変数: omega, c, d）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.042e+00 | `v_{2}` | `v_{2}` |
| 2 | 8.670e-01 | `Exp(C)` | `np.exp(p_{0})` |
| 3 | 2.178e-01 | `Divide(v_{0}, v_{1})` | `(v_{0}/v_{1})` |
| 6 | 9.598e-02 | `Subtract(Divide(v_{0}, v_{1}), Inv(v_{0}))` | `((v_{0}/v_{1}))-((1/v_{0}))` |

### `II.27.16` （変数: epsilon, c, Ef）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.269e+02 | `v_{2}` | `v_{2}` |
| 2 | 9.076e+01 | `Exp(v_{2})` | `np.exp(v_{2})` |
| 4 | 7.155e+01 | `Plus(C, Exp(v_{2}))` | `(p_{0})+(np.exp(v_{2}))` |
| 5 | 6.559e+01 | `Times(Exp(v_{2}), Sqrt(v_{1}))` | `(np.exp(v_{2}))*(np.sqrt(v_{1}))` |
| 6 | 5.369e+01 | `Times(v_{1}, Exp(Exp(ArcTan(v_{2}))))` | `(v_{1})*(np.exp(np.exp(np.arctan(v_{2}))))` |

### `II.34.2a` （変数: q, v, r）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 6.203e-01 | `C` | `p_{0}` |
| 2 | 4.051e-01 | `Inv(v_{2})` | `(1/v_{2})` |
| 3 | 3.771e-01 | `Tan(Inv(v_{2}))` | `np.tan((1/v_{2}))` |
| 4 | 2.914e-01 | `Divide(Sqrt(v_{1}), v_{2})` | `(np.sqrt(v_{1})/v_{2})` |
| 5 | 2.544e-01 | `Divide(v_{1}, Plus(v_{2}, v_{2}))` | `(v_{1}/(v_{2})+(v_{2}))` |

### `II.34.2` （変数: q, v, r）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.352e+01 | `v_{0}` | `v_{0}` |
| 2 | 9.465e+00 | `Log(C)` | `np.log(p_{0})` |
| 3 | 7.553e+00 | `Times(v_{0}, v_{1})` | `(v_{0})*(v_{1})` |
| 5 | 5.531e+00 | `Plus(v_{2}, Times(v_{0}, v_{1}))` | `(v_{2})+((v_{0})*(v_{1}))` |
| 6 | 2.995e+00 | `Times(v_{2}, Times(v_{1}, Sqrt(v_{0})))` | `(v_{2})*((v_{1})*(np.sqrt(v_{0})))` |

### `II.34.11` （変数: g_, q, B, m）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 5.247e+00 | `v_{0}` | `v_{0}` |
| 2 | 5.074e+00 | `Exp(C)` | `np.exp(p_{0})` |
| 3 | 4.257e+00 | `Plus(v_{0}, v_{1})` | `(v_{0})+(v_{1})` |
| 4 | 4.071e+00 | `Times(v_{0}, Sqrt(v_{1}))` | `(v_{0})*(np.sqrt(v_{1}))` |
| 5 | 3.487e+00 | `Times(v_{1}, Divide(v_{0}, v_{3}))` | `(v_{1})*((v_{0}/v_{3}))` |
| 6 | 2.941e+00 | `Times(v_{1}, Divide(v_{0}, Sqrt(v_{3})))` | `(v_{1})*((v_{0}/np.sqrt(v_{3})))` |

### `II.34.29a` （変数: q, h, m）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 7.584e-01 | `C` | `p_{0}` |
| 2 | 2.176e-01 | `Inv(v_{2})` | `(1/v_{2})` |
| 3 | 1.846e-01 | `Inv(ArcSin(v_{2}))` | `(1/np.arcsin(v_{2}))` |
| 4 | 1.762e-01 | `Divide(v_{0}, Exp(v_{2}))` | `(v_{0}/np.exp(v_{2}))` |
| 5 | 1.322e-01 | `Divide(ArcTan(Log(v_{0})), v_{2})` | `(np.arctan(np.log(v_{0}))/v_{2})` |
| 6 | 1.262e-01 | `Divide(Times(v_{0}, Log(C)), v_{2})` | `((v_{0})*(np.log(p_{0}))/v_{2})` |

### `II.34.29b` （変数: g_, h, Jz, mom, B）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 2.878e+02 | `v_{4}` | `v_{4}` |
| 2 | 2.190e+02 | `Neg(C)` | `(-p_{0})` |
| 3 | 2.163e+02 | `Inv(Exp(C))` | `(1/np.exp(p_{0}))` |
| 4 | 2.018e+02 | `Times(v_{2}, Exp(v_{4}))` | `(v_{2})*(np.exp(v_{4}))` |
| 5 | 1.954e+02 | `Divide(v_{4}, Inv(Neg(C)))` | `(v_{4}/(1/(-p_{0})))` |
| 6 | 1.682e+02 | `Times(v_{4}, Divide(Exp(C), v_{1}))` | `(v_{4})*((np.exp(p_{0})/v_{1}))` |

### `II.35.18` （変数: n_0, kb, T, mom, B）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 5.203e-01 | `C` | `p_{0}` |
| 2 | 3.046e-01 | `Inv(v_{3})` | `(1/v_{3})` |
| 3 | 2.875e-01 | `Sin(Log(v_{0}))` | `np.sin(np.log(v_{0}))` |
| 4 | 2.689e-01 | `Divide(ArcTan(v_{0}), v_{3})` | `(np.arctan(v_{0})/v_{3})` |
| 5 | 2.277e-01 | `Divide(v_{0}, Plus(v_{3}, v_{4}))` | `(v_{0}/(v_{3})+(v_{4}))` |
| 6 | 2.181e-01 | `ArcSin(Divide(v_{0}, Plus(v_{3}, v_{4})))` | `np.arcsin((v_{0}/(v_{3})+(v_{4})))` |

### `II.35.21` （変数: n_rho, mom, B, kb, T）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 5.490e+00 | `v_{1}` | `v_{1}` |
| 2 | 4.936e+00 | `Exp(C)` | `np.exp(p_{0})` |
| 3 | 3.156e+00 | `Times(v_{0}, v_{1})` | `(v_{0})*(v_{1})` |
| 5 | 2.246e+00 | `Subtract(Times(v_{0}, v_{1}), v_{3})` | `((v_{0})*(v_{1}))-(v_{3})` |
| 6 | 2.081e+00 | `Times(v_{0}, Subtract(v_{1}, Log(v_{3})))` | `(v_{0})*((v_{1})-(np.log(v_{3})))` |

### `II.36.38` （変数: mom, H, kb, T, alpha, epsilon, c, M）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.116e+00 | `v_{0}` | `v_{0}` |
| 3 | 1.056e+00 | `Exp(Inv(v_{2}))` | `np.exp((1/v_{2}))` |
| 4 | 9.252e-01 | `Plus(v_{0}, Cos(v_{3}))` | `(v_{0})+(np.cos(v_{3}))` |
| 5 | 9.056e-01 | `Divide(Sqrt(Exp(v_{0})), v_{2})` | `(np.sqrt(np.exp(v_{0}))/v_{2})` |
| 6 | 8.275e-01 | `EML(Inv(v_{2}), Divide(v_{3}, v_{0}))` | `(np.exp((1/v_{2})) - np.log((v_{3}/v_{0})))` |

### `II.38.3` （変数: Y, A, d, x）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.346e+01 | `v_{3}` | `v_{3}` |
| 2 | 1.118e+01 | `Log(C)` | `np.log(p_{0})` |
| 3 | 9.005e+00 | `Times(v_{1}, v_{3})` | `(v_{1})*(v_{3})` |
| 5 | 7.843e+00 | `Exp(EML(ArcTan(v_{0}), v_{2}))` | `np.exp((np.exp(np.arctan(v_{0})) - np.log(v_{2})))` |
| 6 | 7.515e+00 | `Times(v_{3}, Times(v_{1}, Log(v_{0})))` | `(v_{3})*((v_{1})*(np.log(v_{0})))` |

### `II.38.14` （変数: Y, sigma）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 6.226e-01 | `C` | `p_{0}` |
| 2 | 1.816e-01 | `Inv(v_{1})` | `(1/v_{1})` |
| 3 | 1.709e-01 | `ArcTan(Inv(v_{1}))` | `np.arctan((1/v_{1}))` |
| 4 | 9.354e-02 | `Divide(Log(v_{0}), v_{1})` | `(np.log(v_{0})/v_{1})` |
| 5 | 6.336e-02 | `Sin(Divide(Log(v_{0}), v_{1}))` | `np.sin((np.log(v_{0})/v_{1}))` |
| 6 | 5.823e-02 | `Times(Log(v_{0}), ArcTan(Inv(v_{1})))` | `(np.log(v_{0}))*(np.arctan((1/v_{1})))` |

### `III.4.32` （変数: h, omega, kb, T）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.036e+01 | `v_{3}` | `v_{3}` |
| 2 | 9.015e+00 | `Exp(C)` | `np.exp(p_{0})` |
| 3 | 7.852e+00 | `Times(v_{2}, v_{3})` | `(v_{2})*(v_{3})` |
| 5 | 7.666e+00 | `Times(v_{2}, Subtract(C, v_{0}))` | `(v_{2})*((p_{0})-(v_{0}))` |
| 6 | 6.128e+00 | `Divide(Divide(Exp(C), v_{0}), v_{1})` | `((np.exp(p_{0})/v_{0})/v_{1})` |

### `III.4.33` （変数: h, omega, kb, T）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 6.953e+00 | `v_{3}` | `v_{3}` |
| 2 | 5.155e+00 | `Exp(C)` | `np.exp(p_{0})` |
| 3 | 7.615e-01 | `Times(v_{2}, v_{3})` | `(v_{2})*(v_{3})` |
| 5 | 3.774e-01 | `Subtract(Times(v_{2}, v_{3}), C)` | `((v_{2})*(v_{3}))-(p_{0})` |
| 6 | 3.682e-01 | `Subtract(Times(v_{2}, v_{3}), Sin(C))` | `((v_{2})*(v_{3}))-(np.sin(p_{0}))` |

### `III.7.38` （変数: mom, B, h）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 5.649e+01 | `v_{0}` | `v_{0}` |
| 2 | 3.792e+01 | `Exp(v_{0})` | `np.exp(v_{0})` |
| 3 | 3.726e+01 | `Sqrt(Neg(C))` | `np.sqrt((-p_{0}))` |
| 4 | 3.113e+01 | `Times(v_{0}, Log(C))` | `(v_{0})*(np.log(p_{0}))` |
| 5 | 2.922e+01 | `Times(v_{1}, Times(v_{0}, v_{0}))` | `(v_{1})*((v_{0})*(v_{0}))` |
| 6 | 2.101e+01 | `Divide(Divide(v_{0}, ArcSin(C)), v_{2})` | `((v_{0}/np.arcsin(p_{0}))/v_{2})` |

### `III.8.54` （変数: E_n, t, h）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 6.335e-01 | `C` | `p_{0}` |
| 2 | 3.577e-01 | `Cos(C)` | `np.cos(p_{0})` |
| 3 | 3.531e-01 | `Inv(Tan(C))` | `(1/np.tan(p_{0}))` |
| 4 | 3.527e-01 | `Cos(Inv(Sin(v_{0})))` | `np.cos((1/np.sin(v_{0})))` |
| 5 | 3.517e-01 | `Times(ArcTan(v_{0}), Tan(C))` | `(np.arctan(v_{0}))*(np.tan(p_{0}))` |
| 6 | 3.513e-01 | `Divide(v_{0}, ArcSin(ArcSin(ArcSin(v_{0}))))` | `(v_{0}/np.arcsin(np.arcsin(np.arcsin(v_{0}))))` |

### `III.9.52` （変数: p_d, Ef, t, h, omega, omega_0）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.919e+01 | `v_{5}` | `v_{5}` |
| 2 | 1.459e+01 | `Log(C)` | `np.log(p_{0})` |
| 4 | 1.357e+01 | `Times(v_{1}, Tan(C))` | `(v_{1})*(np.tan(p_{0}))` |
| 5 | 1.300e+01 | `Plus(Exp(v_{0}), Exp(v_{1}))` | `(np.exp(v_{0}))+(np.exp(v_{1}))` |
| 6 | 1.261e+01 | `Times(v_{1}, Divide(Neg(C), v_{3}))` | `(v_{1})*(((-p_{0})/v_{3}))` |

### `III.12.43` （変数: n, h）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 9.674e-01 | `C` | `p_{0}` |
| 2 | 6.745e-01 | `Sqrt(v_{0})` | `np.sqrt(v_{0})` |
| 4 | 6.010e-01 | `Log(Sqrt(Exp(v_{0})))` | `np.log(np.sqrt(np.exp(v_{0})))` |
| 5 | 1.895e-01 | `Times(v_{1}, Log(Sqrt(v_{0})))` | `(v_{1})*(np.log(np.sqrt(v_{0})))` |

### `III.13.18` （変数: E_n, d, k, h）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 7.059e+02 | `v_{1}` | `v_{1}` |
| 2 | 5.448e+02 | `Neg(C)` | `(-p_{0})` |
| 3 | 5.372e+02 | `Exp(Tan(C))` | `np.exp(np.tan(p_{0}))` |
| 4 | 4.672e+02 | `Times(v_{1}, Neg(C))` | `(v_{1})*((-p_{0}))` |
| 5 | 4.521e+02 | `Times(Log(v_{1}), Neg(C))` | `(np.log(v_{1}))*((-p_{0}))` |
| 6 | 3.597e+02 | `Times(v_{2}, Times(v_{0}, Exp(v_{1})))` | `(v_{2})*((v_{0})*(np.exp(v_{1})))` |

### `III.14.14` （変数: I_0, q, Volt, kb, T）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 7.479e+00 | `v_{0}` | `v_{0}` |
| 2 | 6.691e+00 | `Exp(C)` | `np.exp(p_{0})` |
| 3 | 6.282e+00 | `Plus(v_{0}, v_{0})` | `(v_{0})+(v_{0})` |
| 4 | 5.815e+00 | `Times(v_{1}, Exp(v_{2}))` | `(v_{1})*(np.exp(v_{2}))` |
| 5 | 5.294e+00 | `Times(v_{1}, Times(v_{0}, v_{2}))` | `(v_{1})*((v_{0})*(v_{2}))` |

### `III.15.12` （変数: U, k, d）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 5.760e+00 | `v_{0}` | `v_{0}` |
| 2 | 5.241e+00 | `Exp(C)` | `np.exp(p_{0})` |
| 3 | 4.626e+00 | `Plus(v_{0}, v_{0})` | `(v_{0})+(v_{0})` |
| 4 | 4.621e+00 | `Times(v_{0}, Tan(C))` | `(v_{0})*(np.tan(p_{0}))` |
| 5 | 4.573e+00 | `EML(Sqrt(v_{0}), Log(v_{2}))` | `(np.exp(np.sqrt(v_{0})) - np.log(np.log(v_{2})))` |
| 6 | 4.530e+00 | `Times(v_{0}, ArcCos(Neg(Inv(v_{2}))))` | `(v_{0})*(np.arccos((-(1/v_{2}))))` |

### `III.15.14` （変数: h, E_n, d）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 9.900e-01 | `C` | `p_{0}` |
| 2 | 1.652e-02 | `Log(C)` | `np.log(p_{0})` |
| 3 | 1.634e-02 | `Tan(Log(C))` | `np.tan(np.log(p_{0}))` |
| 4 | 1.433e-02 | `Divide(Log(C), v_{2})` | `(np.log(p_{0})/v_{2})` |
| 5 | 1.253e-02 | `Tan(Plus(C, ArcCos(v_{2})))` | `np.tan((p_{0})+(np.arccos(v_{2})))` |
| 6 | 1.054e-02 | `Divide(Divide(v_{0}, v_{2}), Tan(C))` | `((v_{0}/v_{2})/np.tan(p_{0}))` |

### `III.15.27` （変数: alpha, n, d）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 2.143e+00 | `v_{0}` | `v_{0}` |
| 4 | 1.871e+00 | `Plus(v_{0}, Sin(v_{2}))` | `(v_{0})+(np.sin(v_{2}))` |
| 5 | 1.718e+00 | `Divide(Plus(v_{0}, v_{0}), v_{1})` | `((v_{0})+(v_{0})/v_{1})` |
| 6 | 1.627e+00 | `Times(Log(C), Divide(v_{0}, v_{1}))` | `(np.log(p_{0}))*((v_{0}/v_{1}))` |

### `III.17.37` （変数: beta, alpha, theta）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 5.244e+00 | `C` | `p_{0}` |
| 2 | 4.469e+00 | `Cos(v_{2})` | `np.cos(v_{2})` |
| 3 | 4.219e+00 | `Log(Tan(v_{2}))` | `np.log(np.tan(v_{2}))` |
| 4 | 3.409e+00 | `Times(v_{1}, Cos(v_{2}))` | `(v_{1})*(np.cos(v_{2}))` |
| 5 | 3.041e+00 | `Times(v_{1}, Tan(Cos(v_{2})))` | `(v_{1})*(np.tan(np.cos(v_{2})))` |
| 6 | 2.745e+00 | `Times(Cos(v_{2}), Sqrt(Exp(v_{1})))` | `(np.cos(v_{2}))*(np.sqrt(np.exp(v_{1})))` |

### `III.21.20` （変数: rho_c_0, q, A_vec, m）

| 複雑度 | RMSE | 式 | Python 表現 |
|--------|------|-----|------------|
| 1 | 1.541e+01 | `C` | `p_{0}` |
| 2 | 1.001e+01 | `Neg(C)` | `(-p_{0})` |
| 3 | 9.505e+00 | `Subtract(C, v_{1})` | `(p_{0})-(v_{1})` |
| 4 | 7.773e+00 | `Neg(Times(v_{1}, v_{2}))` | `(-(v_{1})*(v_{2}))` |
| 5 | 7.542e+00 | `Subtract(C, Times(v_{1}, v_{2}))` | `(p_{0})-((v_{1})*(v_{2}))` |
| 6 | 6.827e+00 | `Divide(Neg(C), Divide(v_{3}, v_{1}))` | `((-p_{0})/(v_{3}/v_{1}))` |

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

### 9.1 従来 eml-sr 実験との比較

従来実験 (`sample_code/allfunc_moreestimate_sr_report.md`) では 99式中 9式 (9.1%) を完全回収、部分回収含め 10式 (10.1%) であった。本実験は同一データ・同一ハイパーパラメータで eml-sr_model_first_AI を適用し、Rust 実装による速度・数値安定性・探索挙動の差異を評価する。

### 9.2 EML 演算子と失敗パターン

EML 演算子 $\mathrm{EML}(a,b) = e^a - \ln b$ は指数・対数構造をコンパクトに表現できるが、Gaussian 型 ($\exp(-\theta^2/2)$)、相対論的因子 ($1/\sqrt{1-v^2/c^2}$)、有理数係数 ($1/2$, $3/2$)、4変数以上の式は複雑度 6 の範囲では依然困難である。

今回の実験では 99 個の方程式に対して **9 式 (9.1%)** を完全回収し、部分的回収も含めると **10 式 (10.1%)** となった。

---

*本レポートは `feynman_eml_sr_model_first_AI.py` により自動生成されました。*
