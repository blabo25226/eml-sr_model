use crate::core::expression::{Expression, Node};
use crate::core::value::{real, Value};
use crate::ops::registry::OperatorRegistry;
use std::sync::Arc;

/// A structural template for numerical optimization of expression constants.
pub struct Template {
    pub expression: Expression,
    pub param_indices: Vec<usize>,
}

impl Template {
    /// Extracts parameter locations and initial values from an expression.
    pub fn from_expression(expr: &Expression) -> (Self, Vec<f64>) {
        let mut nodes = expr.nodes.clone();
        let mut param_indices = Vec::new();
        let mut initial_values = Vec::new();

        for (i, node) in nodes.iter_mut().enumerate() {
            if let Node::Param { initial_value, .. } = node {
                param_indices.push(i);
                initial_values.push(initial_value.re);
            }
        }

        (
            Self {
                expression: Expression::new(
                    nodes,
                    expr.var_count(),
                    expr.param_count(),
                    expr.display().to_string(),
                ),
                param_indices,
            },
            initial_values,
        )
    }
}

/// Refines numerical constants using Levenberg-Marquardt with optional multi-start.
pub fn refine_constants(
    expr: &Expression,
    inputs: &[Vec<Value>],
    targets: &[f64],
    registry: &Arc<OperatorRegistry>,
    max_iters: usize,
) -> (Expression, f64) {
    refine_constants_multi(expr, inputs, targets, registry, max_iters, 1)
}

/// Multi-start LM: perturb initial parameters and keep the best result.
pub fn refine_constants_multi(
    expr: &Expression,
    inputs: &[Vec<Value>],
    targets: &[f64],
    registry: &Arc<OperatorRegistry>,
    max_iters: usize,
    n_starts: usize,
) -> (Expression, f64) {
    let (template, base_params) = Template::from_expression(expr);
    if base_params.is_empty() {
        return (expr.clone(), compute_error(expr, inputs, targets, registry));
    }

    let mut best_expr = expr.clone();
    let mut best_error = f64::INFINITY;

    let starts = n_starts.max(1);
    for s in 0..starts {
        let mut params: Vec<f64> = base_params
            .iter()
            .map(|&p| {
                if s == 0 {
                    p
                } else {
                    let scale = 1.0 + 0.1 * (s as f64) * if s % 2 == 0 { 1.0 } else { -1.0 };
                    p * scale
                }
            })
            .collect();

        let (refined_expr, error) = lm_optimize(&template, &mut params, inputs, targets, registry, max_iters);
        if error < best_error {
            best_error = error;
            best_expr = refined_expr;
        }
    }

    (best_expr, best_error)
}

/// Snap-candidate values for one fitted constant: integers, simple rationals,
/// and multiples of pi / (2 pi) / (4 pi) — the constants physics actually uses.
fn snap_candidates(p: f64) -> Vec<f64> {
    use std::f64::consts::PI;
    let mut cands = Vec::with_capacity(12);
    for den in [1.0f64, 2.0, 3.0, 4.0] {
        cands.push((p * den).round() / den);
    }
    for unit in [PI, PI / 2.0, 2.0 * PI, 4.0 * PI] {
        cands.push((p / unit).round() * unit);
        if p.abs() > 1e-300 {
            let k = (p * unit).round();
            if k != 0.0 {
                cands.push(k / unit);
            }
        }
    }
    cands.retain(|c| c.is_finite() && (c - p).abs() > 0.0);
    cands.dedup();
    cands
}

/// Greedily snaps each fitted `Param` to a nearby "nice" constant, keeping a
/// snap only when the full-data RMSE does not get worse.
pub fn snap_constants(
    expr: &Expression,
    inputs: &[Vec<Value>],
    targets: &[f64],
    registry: &Arc<OperatorRegistry>,
) -> Expression {
    let (template, mut params) = Template::from_expression(expr);
    if params.is_empty() {
        return expr.clone();
    }
    let mut best_error =
        compute_error_with_params(&template.expression, &params, inputs, targets, registry);
    if !best_error.is_finite() {
        return expr.clone();
    }

    for i in 0..params.len() {
        let original = params[i];
        let mut best_val = original;
        for cand in snap_candidates(original) {
            params[i] = cand;
            let err = compute_error_with_params(
                &template.expression,
                &params,
                inputs,
                targets,
                registry,
            );
            if err <= best_error * (1.0 + 1e-9) && err.is_finite() {
                best_error = err.min(best_error);
                best_val = cand;
            }
        }
        params[i] = best_val;
    }

    let mut final_nodes = template.expression.nodes.clone();
    for (i, &v) in params.iter().enumerate() {
        let node_idx = template.param_indices[i];
        if let Node::Param { id, .. } = final_nodes[node_idx] {
            final_nodes[node_idx] = Node::Param {
                id,
                initial_value: real(v),
            };
        }
    }
    Expression::new(
        final_nodes,
        template.expression.var_count(),
        template.expression.param_count(),
        template.expression.display().to_string(),
    )
}

fn lm_optimize(
    template: &Template,
    params: &mut [f64],
    inputs: &[Vec<Value>],
    targets: &[f64],
    registry: &Arc<OperatorRegistry>,
    max_iters: usize,
) -> (Expression, f64) {
    let mut lambda = 1e-2;
    let mut best_error =
        compute_error_with_params(&template.expression, params, inputs, targets, registry);
    let mut best_params = params.to_vec();

    for _ in 0..max_iters {
        let (jacobian, residuals) = compute_jacobian_and_residuals(
            &template.expression,
            params,
            inputs,
            targets,
            registry,
        );

        let delta = solve_lm_delta(&jacobian, &residuals, lambda);
        if delta.is_empty() {
            break;
        }

        let mut trial = params.to_vec();
        for (p, d) in trial.iter_mut().zip(delta.iter()) {
            *p -= d;
        }

        let trial_error = compute_error_with_params(
            &template.expression,
            &trial,
            inputs,
            targets,
            registry,
        );

        if trial_error < best_error {
            best_error = trial_error;
            best_params = trial.clone();
            for (p, t) in params.iter_mut().zip(trial.iter()) {
                *p = *t;
            }
            lambda = (lambda * 0.1).max(1e-12);
            if best_error < 1e-14 {
                break;
            }
        } else {
            lambda = (lambda * 10.0).min(1e12);
        }
    }

    let mut final_nodes = template.expression.nodes.clone();
    for (i, &v) in best_params.iter().enumerate() {
        let node_idx = template.param_indices[i];
        if let Node::Param { id, .. } = final_nodes[node_idx] {
            final_nodes[node_idx] = Node::Param {
                id,
                initial_value: real(v),
            };
        }
    }

    (
        Expression::new(
            final_nodes,
            template.expression.var_count(),
            template.expression.param_count(),
            template.expression.display().to_string(),
        ),
        best_error,
    )
}

/// Solve $(J^T J + \lambda I)\,\delta = J^T r$ for $\delta$ (residuals $r = f - y$).
fn solve_lm_delta(jacobian: &[Vec<f64>], residuals: &[f64], lambda: f64) -> Vec<f64> {
    let n_params = jacobian.first().map(|r| r.len()).unwrap_or(0);
    if n_params == 0 {
        return Vec::new();
    }

    let mut jtj = vec![vec![0.0f64; n_params]; n_params];
    let mut jtr = vec![0.0f64; n_params];

    for (row, res) in jacobian.iter().zip(residuals.iter()) {
        for i in 0..n_params {
            for j in 0..n_params {
                jtj[i][j] += row[i] * row[j];
            }
            jtr[i] += row[i] * res;
        }
    }

    for i in 0..n_params {
        jtj[i][i] += lambda;
    }

    gaussian_elimination(&mut jtj, &mut jtr)
}

fn gaussian_elimination(a: &mut [Vec<f64>], b: &mut [f64]) -> Vec<f64> {
    let n = b.len();
    if n == 0 {
        return Vec::new();
    }

    for col in 0..n {
        let mut pivot = col;
        for row in (col + 1)..n {
            if a[row][col].abs() > a[pivot][col].abs() {
                pivot = row;
            }
        }
        if a[pivot][col].abs() < 1e-15 {
            continue;
        }
        a.swap(col, pivot);
        b.swap(col, pivot);

        let div = a[col][col];
        for j in col..n {
            a[col][j] /= div;
        }
        b[col] /= div;

        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = a[row][col];
            for j in col..n {
                a[row][j] -= factor * a[col][j];
            }
            b[row] -= factor * b[col];
        }
    }

    b.to_vec()
}

fn compute_error(
    expr: &Expression,
    inputs: &[Vec<Value>],
    targets: &[f64],
    reg: &OperatorRegistry,
) -> f64 {
    let mut total = 0.0;
    for (i, row) in inputs.iter().enumerate() {
        if let Some(val) = expr.eval(row, reg) {
            total += (val.re - targets[i]).powi(2);
        } else {
            return f64::INFINITY;
        }
    }
    (total / inputs.len() as f64).sqrt()
}

fn compute_error_with_params(
    expr: &Expression,
    params: &[f64],
    inputs: &[Vec<Value>],
    targets: &[f64],
    reg: &OperatorRegistry,
) -> f64 {
    let param_values: Vec<Value> = params.iter().map(|&v| real(v)).collect();
    let mut total = 0.0;
    for (i, row) in inputs.iter().enumerate() {
        if let Some(val) = expr.eval_with_params(row, &param_values, reg) {
            total += (val.re - targets[i]).powi(2);
        } else {
            return f64::INFINITY;
        }
    }
    (total / inputs.len() as f64).sqrt()
}

fn compute_jacobian_and_residuals(
    expr: &Expression,
    params: &[f64],
    inputs: &[Vec<Value>],
    targets: &[f64],
    reg: &OperatorRegistry,
) -> (Vec<Vec<f64>>, Vec<f64>) {
    let n = targets.len();
    let m = params.len();
    let epsilon = 1e-6;
    let mut jacobian = vec![vec![0.0; m]; n];
    let mut residuals = vec![0.0; n];

    let base_v: Vec<Value> = params.iter().map(|&v| real(v)).collect();

    for i in 0..n {
        let f0 = expr
            .eval_with_params(&inputs[i], &base_v, reg)
            .map(|v| v.re)
            .unwrap_or(0.0);
        residuals[i] = f0 - targets[i];

        for j in 0..m {
            let mut shifted_params = params.to_vec();
            shifted_params[j] += epsilon;
            let shifted_v: Vec<Value> = shifted_params.iter().map(|&v| real(v)).collect();
            let f1 = expr
                .eval_with_params(&inputs[i], &shifted_v, reg)
                .map(|v| v.re)
                .unwrap_or(0.0);
            jacobian[i][j] = (f1 - f0) / epsilon;
        }
    }

    (jacobian, residuals)
}
