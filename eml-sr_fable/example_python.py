"""
eml-sr_fable Python インターフェースのデモ (v5)

ビルド:
  cd eml-sr_fable
  maturin develop --release --features python,full-math

実行:
  python example_python.py
"""

import numpy as np
import eml_sr_fable


def rmse(y_true, y_pred):
    y_true = np.asarray(y_true, dtype=float)
    y_pred = np.asarray(y_pred, dtype=float)
    if not np.all(np.isfinite(y_pred)):
        return float("inf")
    return float(np.sqrt(np.mean((y_true - y_pred) ** 2)))


def pick_conservative(candidates, X, y, band_ratio=1.05):
    """誤差が最良の band_ratio 倍以内で、複雑度が最小の候補を選ぶ (ベンチと同一方針)。"""
    scored = []
    for c in candidates:
        preds = np.asarray(c.predict(X.tolist()), dtype=float)
        scored.append((c, rmse(y, preds)))
    finite = [(c, r) for c, r in scored if np.isfinite(r)]
    if not finite:
        raise RuntimeError("no finite candidates")
    best_rmse = min(r for _, r in finite)
    band = [(c, r) for c, r in finite if r <= best_rmse * band_ratio]
    return min(band, key=lambda cr: (cr[0].complexity, cr[1]))[0]


def main():
    print("=" * 63)
    print("  eml-sr_fable Python Interface Demo (v5)")
    print("=" * 63)

    # 1. Searcher 初期化 (Feynman 1.test 相当の高速設定)
    searcher = eml_sr_fable.Searcher(
        max_complexity=6,
        complexity_penalty=0.08,
        beam_width=1000,
        time_budget_s=60.0,
        subsample_size=256,
        early_exit_threshold=1e-9,
        refinement_top_k=8,
        snap_constants=True,
        powerlaw_stage=True,
        ratio_search=True,
        rational_stage=True,
        affine_scaling=True,
        max_boost_terms=6,
        verbose=False,
    )

    # 2. 定数の閉形式同定
    print("\n[1] recognize_constant(π)")
    r = searcher.recognize_constant(np.pi)
    print(f"    formula:    {r.formula}")
    print(f"    to_python:  {r.to_python()}")
    print(f"    to_latex:   {r.to_latex()}")
    print(f"    complexity: {r.complexity}")

    # 3. 1 変数: f(x) = sin(x) + 1
    print("\n[2] find_function: f(x) = sin(x) + 1")
    xs = np.linspace(0.1, 2 * np.pi, 80)
    ys = np.sin(xs) + 1.0
    r = searcher.find_function(xs, ys)
    print(f"    formula:    {r.formula}")
    print(f"    to_python:  {r.to_python()}")
    print(f"    RMSE:       {rmse(ys, [r.eval(float(x)) for x in xs]):.3e}")
    print(f"    complexity: {r.complexity}")

    # 4. 多変数 + Pareto front + 節約的タイブレーク
    print("\n[3] find_candidates: y = x0^2 + 3*x1 (節約的タイブレーク)")
    rng = np.random.default_rng(0)
    X = rng.uniform(0.5, 3.0, size=(500, 2))
    y = X[:, 0] ** 2 + 3.0 * X[:, 1]

    candidates = searcher.find_candidates(X.tolist(), y.tolist())
    best = pick_conservative(candidates, X, y)
    preds = np.asarray(best.predict(X.tolist()), dtype=float)
    print(f"    pareto size: {len(candidates)}")
    print(f"    best formula: {best.formula}")
    print(f"    best python:  {best.to_python()}")
    print(f"    RMSE:         {rmse(y, preds):.3e}")
    print(f"    complexity:   {best.complexity}")

    # 5. sklearn 風 API
    print("\n[4] fit / predict: f(x0, x1) = x0 * x1 + 0.5")
    inputs = np.array([[1.0, 2.0], [2.0, 3.0], [3.0, 4.0], [0.5, 0.5]])
    targets = inputs[:, 0] * inputs[:, 1] + 0.5
    r = searcher.fit(inputs, targets)
    print(f"    formula:     {r.formula}")
    print(f"    predictions: {r.predict(inputs)}")

    # 6. EML 演算子の例: f(x) = exp(x) - ln(x + 5)
    print("\n[5] EML challenge: f(x) = exp(x) - ln(x + 5)")
    xs_eml = np.linspace(1.0, 10.0, 100)
    ys_eml = np.exp(xs_eml) - np.log(xs_eml + 5.0)
    r = searcher.find_function(xs_eml, ys_eml)
    print(f"    formula:    {r.formula}")
    print(f"    to_python:  {r.to_python()}")
    print(f"    complexity: {r.complexity}")

    # 7. eval_batch
    print("\n[6] eval_batch on held-out points")
    X_test = rng.uniform(0.5, 3.0, size=(20, 2))
    y_test = X_test[:, 0] ** 2 + 3.0 * X_test[:, 1]
    y_hat = np.asarray(best.eval_batch(X_test.tolist()), dtype=float)
    print(f"    hold-out RMSE: {rmse(y_test, y_hat):.3e}")

    print("\n" + "=" * 63)
    print("  Done. See manual_eml-sr_fable.md for full API and benchmarks.")
    print("=" * 63)


if __name__ == "__main__":
    main()
