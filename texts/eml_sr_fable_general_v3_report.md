# EML-SR Fable — 非 Feynman 一般ベンチ (v3)

合成 20 式 (負値域変数・符号混在ターゲットを含む)。1.test 条件 + ホールドアウト 250 点評価。

**結果: 完全回収 18/20、部分 0、失敗 2、総時間 0.6 分**

| # | 式 | カテゴリ | ステータス | test RMSE | 複雑度 | 発見式 | 時間(s) |
|---|-----|---------|-----------|-----------|--------|--------|---------|
| 1 | `poly3` | polynomial | ❌ failed | 1.564e-01 | 20 | `Plus(Plus(Plus(Times(10.390612100496076, v_{0})...` | 4.1 |
| 2 | `poly_cross` | polynomial | ✅ ok | 2.053e-16 | 8 | `Plus(Times(Square(v_{0}), v_{1}), Times(v_{0}, ...` | 0.0 |
| 3 | `nguyen5` | trig-comp | ✅ ok | 3.846e-17 | 8 | `Plus(Times(Cos(v_{0}), Sin(Square(v_{0}))), -1)` | 3.9 |
| 4 | `nguyen6` | trig-comp | ❌ failed | 9.686e-02 | 28 | `Divide(Plus(Plus(Plus(-2.2961510312647047, Time...` | 5.2 |
| 5 | `nguyen7` | log | ✅ ok | 7.273e-06 | 36 | `Divide(Plus(Plus(Plus(6.91208985959771, Times(-...` | 9.8 |
| 6 | `rational1` | rational | ✅ ok | 4.487e-15 | 36 | `Divide(Plus(Plus(Times(0.0000000000000000003523...` | 0.3 |
| 7 | `rational2` | rational | ✅ ok | 6.114e-16 | 5 | `Inv(Plus(1.0000000000000018, Square(v_{0})))` | 0.0 |
| 8 | `rational3` | rational | ✅ ok | 2.041e-15 | 8 | `Inv(Plus(Inv(v_{0}), Times(0.9999999999999962, ...` | 0.3 |
| 9 | `shifted_inv` | rational | ✅ ok | 1.362e-14 | 6 | `Inv(Plus(0.8333333333333319, Times(0.3333333333...` | 0.0 |
| 10 | `logistic1` | logistic | ✅ ok | 9.919e-17 | 7 | `Inv(Plus(1, Exp(Times(-1.9999999999999991, v_{0...` | 0.1 |
| 11 | `logistic2` | logistic | ✅ ok | 5.205e-16 | 11 | `Inv(Plus(1, Exp(Plus(Times(-2.0000000000000013,...` | 0.1 |
| 12 | `tanh_sum` | logistic | ✅ ok | 9.928e-17 | 4 | `Tanh(Plus(v_{0}, v_{1}))` | 0.4 |
| 13 | `saturation` | exp | ✅ ok | 3.714e-08 | 37 | `Divide(Plus(Plus(Plus(0.8061003727739423, Times...` | 10.8 |
| 14 | `exp_decay` | exp | ✅ ok | 1.469e-14 | 7 | `Exp(Plus(Times(-0.4999999999999974, Square(v_{0...` | 0.0 |
| 15 | `mixed_prod` | mixed | ✅ ok | 8.568e-17 | 5 | `Divide(v_{0}, Exp(Square(v_{1})))` | 1.2 |
| 16 | `sine_amp` | mixed | ✅ ok | 0.000e+00 | 4 | `Times(v_{0}, Sin(v_{1}))` | 0.3 |
| 17 | `gauss2d` | exp | ✅ ok | 3.861e-16 | 25 | `Exp(Plus(Plus(Plus(Times(0.00000000000000000015...` | 0.1 |
| 18 | `diff_sq` | polynomial | ✅ ok | 8.977e-15 | 15 | `Plus(Plus(Plus(Times(-2.0000000000000013, Times...` | 0.0 |
| 19 | `cos_diff` | trig-comp | ✅ ok | 3.853e-15 | 6 | `Times(1.9999999999999944, Cos(Subtract(v_{0}, v...` | 0.0 |
| 20 | `sqrt_sum` | mixed | ✅ ok | 1.750e-15 | 17 | `Sqrt(Plus(Plus(Times(0.000000000000000499600361...` | 0.1 |
