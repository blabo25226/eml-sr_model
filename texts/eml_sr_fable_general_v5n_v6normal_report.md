# EML-SR Fable — 非 Feynman 一般ベンチ + 1% ノイズ (v5n, tag=v6normal)

**結果: 完全回収 18/20、部分 1、失敗 1、総時間 3.3 分**

| # | 式 | カテゴリ | ステータス | test RMSE | 複雑度 | 発見式 | 時間(s) |
|---|-----|---------|-----------|-----------|--------|--------|---------|
| 1 | `poly3` | polynomial | ✅ ok | 3.698e-03 | 13 | `Plus(Plus(Times(0.9996994561933896, Cube(v_{0})...` | 5.5 |
| 2 | `poly_cross` | polynomial | ✅ ok | 9.403e-04 | 12 | `Plus(Times(1.0004090981602163, Times(Square(v_{...` | 10.8 |
| 3 | `nguyen5` | trig-comp | ✅ ok | 5.890e-05 | 10 | `Plus(Times(1.0003304005918414, Times(Cos(v_{0})...` | 5.1 |
| 4 | `nguyen6` | trig-comp | ❌ failed | 4.027e-02 | 35 | `Subtract(Exp(Plus(Plus(Plus(Plus(Plus(Times(2.0...` | 7.7 |
| 5 | `nguyen7` | log | ✅ ok | 1.879e-03 | 17 | `Exp(Plus(Plus(Plus(0.47047556885155084, Times(-...` | 12.3 |
| 6 | `rational1` | rational | 🟡 partial | 2.034e-03 | 15 | `Divide(Plus(Times(0.9975582235962677, v_{1}), T...` | 10.2 |
| 7 | `rational2` | rational | ✅ ok | 6.171e-05 | 7 | `Inv(Plus(Times(1.0000016779765695, Square(v_{0}...` | 5.8 |
| 8 | `rational3` | rational | ✅ ok | 4.119e-04 | 10 | `Inv(Plus(Times(0.9988846669690532, Inv(v_{1})),...` | 29.8 |
| 9 | `shifted_inv` | rational | ✅ ok | 1.527e-03 | 5 | `Divide(3.000659887661622, Plus(v_{0}, 2.4993946...` | 5.5 |
| 10 | `logistic1` | logistic | ✅ ok | 1.406e-04 | 6 | `Plus(0.49994370737975763, Times(0.5001565732187...` | 5.7 |
| 11 | `logistic2` | logistic | ✅ ok | 2.705e-04 | 10 | `Plus(Times(1.000053013182566, Sigmoid(Subtract(...` | 14.5 |
| 12 | `tanh_sum` | logistic | ✅ ok | 4.222e-04 | 8 | `Plus(Times(0.9996110013092986, Tanh(Plus(v_{0},...` | 8.5 |
| 13 | `saturation` | exp | ✅ ok | 4.596e-04 | 16 | `Inv(Plus(Plus(Plus(0.2557365639207234, Times(0....` | 14.4 |
| 14 | `exp_decay` | exp | ✅ ok | 3.860e-04 | 7 | `Exp(Plus(1.098639371235853, Times(-0.5002832223...` | 5.4 |
| 15 | `mixed_prod` | mixed | ✅ ok | 5.409e-04 | 9 | `Plus(Times(0.9994507908511425, Divide(v_{0}, Ex...` | 9.9 |
| 16 | `sine_amp` | mixed | ✅ ok | 4.698e-04 | 6 | `Times(Times(0.9996106681919908, v_{0}), Sin(v_{...` | 9.7 |
| 17 | `gauss2d` | exp | ✅ ok | 5.573e-05 | 10 | `Exp(Plus(Times(-0.4999647515999212, Square(v_{1...` | 11.9 |
| 18 | `diff_sq` | polynomial | ✅ ok | 5.482e-03 | 8 | `Plus(Times(0.9996840921188285, Square(Subtract(...` | 7.7 |
| 19 | `cos_diff` | trig-comp | ✅ ok | 7.824e-06 | 6 | `Times(1.9999887196957664, Cos(Subtract(v_{0}, v...` | 10.9 |
| 20 | `sqrt_sum` | mixed | ✅ ok | 3.403e-04 | 10 | `Sqrt(Plus(Times(0.9999186618595259, Square(v_{0...` | 8.6 |
