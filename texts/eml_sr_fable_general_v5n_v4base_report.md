# EML-SR Fable — 非 Feynman 一般ベンチ + 1% ノイズ (v5n, tag=v4base)

**結果: 完全回収 16/20、部分 3、失敗 1、総時間 3.4 分**

| # | 式 | カテゴリ | ステータス | test RMSE | 複雑度 | 発見式 | 時間(s) |
|---|-----|---------|-----------|-----------|--------|--------|---------|
| 1 | `poly3` | polynomial | ✅ ok | 3.698e-03 | 13 | `Plus(Plus(Times(0.9996994561826611, Cube(v_{0})...` | 4.8 |
| 2 | `poly_cross` | polynomial | ✅ ok | 9.403e-04 | 12 | `Plus(Times(1.0004090981609668, Times(Square(v_{...` | 11.0 |
| 3 | `nguyen5` | trig-comp | ✅ ok | 5.890e-05 | 10 | `Plus(Times(1.0003304005918414, Times(Cos(v_{0})...` | 4.9 |
| 4 | `nguyen6` | trig-comp | ❌ failed | 1.785e-02 | 39 | `Plus(Plus(Plus(Plus(Plus(Plus(Plus(1541.5281578...` | 6.5 |
| 5 | `nguyen7` | log | 🟡 partial | 3.579e-03 | 20 | `Plus(Plus(Plus(Plus(Times(2.989922396875767, v_...` | 11.1 |
| 6 | `rational1` | rational | 🟡 partial | 2.034e-03 | 15 | `Divide(Plus(Times(0.9975582236401415, v_{1}), T...` | 10.0 |
| 7 | `rational2` | rational | ✅ ok | 1.454e-04 | 8 | `Plus(Times(1.0004386715719567, Square(Cos(ArcTa...` | 5.6 |
| 8 | `rational3` | rational | ✅ ok | 2.818e-04 | 10 | `Plus(Times(1.0001905585077582, Inv(Plus(Inv(v_{...` | 32.3 |
| 9 | `shifted_inv` | rational | ✅ ok | 1.527e-03 | 5 | `Divide(3.0006598876816244, Plus(v_{0}, 2.499394...` | 5.5 |
| 10 | `logistic1` | logistic | ✅ ok | 1.406e-04 | 6 | `Plus(Times(0.5001565732194062, Tanh(v_{0})), 0....` | 6.0 |
| 11 | `logistic2` | logistic | ✅ ok | 2.705e-04 | 10 | `Plus(Times(1.000053013182566, Sigmoid(Plus(v_{0...` | 12.8 |
| 12 | `tanh_sum` | logistic | ✅ ok | 4.222e-04 | 8 | `Plus(Times(0.9996110013092986, Tanh(Plus(v_{0},...` | 9.9 |
| 13 | `saturation` | exp | 🟡 partial | 1.976e-03 | 16 | `Exp(Plus(Plus(Plus(Times(-0.3685827844324172, v...` | 13.5 |
| 14 | `exp_decay` | exp | ✅ ok | 6.025e-04 | 9 | `Plus(Times(3.000831515089337, Sqrt(Inv(Exp(Squa...` | 5.4 |
| 15 | `mixed_prod` | mixed | ✅ ok | 5.409e-04 | 9 | `Plus(Times(0.9994507908511425, Divide(v_{0}, Ex...` | 9.9 |
| 16 | `sine_amp` | mixed | ✅ ok | 4.698e-04 | 6 | `Times(Times(0.9996106681929896, v_{0}), Sin(v_{...` | 11.8 |
| 17 | `gauss2d` | exp | ✅ ok | 1.303e-04 | 10 | `Exp(Plus(Times(-0.5001312016113038, Square(v_{1...` | 11.7 |
| 18 | `diff_sq` | polynomial | ✅ ok | 5.482e-03 | 8 | `Plus(Times(0.9996840921188284, Square(Subtract(...` | 7.5 |
| 19 | `cos_diff` | trig-comp | ✅ ok | 7.824e-06 | 6 | `Times(1.9999887196977608, Cos(Subtract(v_{0}, v...` | 11.0 |
| 20 | `sqrt_sum` | mixed | ✅ ok | 2.931e-04 | 10 | `Plus(Times(0.9997379617559319, Sqrt(Plus(Square...` | 12.6 |
