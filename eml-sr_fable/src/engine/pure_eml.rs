//! Pure-EML pipeline: the original "combine EML" idea made load-bearing.
//!
//! The only function primitive is EML(x, y) = e^x - ln(y); Plus/Times/Neg
//! and tunable constants are the arithmetic glue. None of the closed-form
//! stages (A/A2/C) is used. Accuracy comes from two mechanisms:
//!
//! 1. **EML basis boosting**: f = c0 + sum_k c_k * EML(P_k, Q_k) where the
//!    arguments are log-augmented affine forms. A single unit's first
//!    argument already spans the whole "monomial x exponential" class
//!    (e^{a0 + sum a_i x_i + sum b_i ln x_i} = C * prod x_i^{b_i} *
//!    e^{sum a_i x_i}), and -ln Q covers logarithmic shapes. Units are
//!    fitted one at a time with multi-start Levenberg-Marquardt in the raw
//!    target space and accepted only when the held-out error improves.
//! 2. **Pure-EML beam search**: the standard beam over the restricted
//!    {EML, Plus, Times, Neg} grammar, with one-step lookahead ranking and
//!    behavioral diversity so building-block subexpressions survive.
//!
//! ln x_i inside the arguments is itself EML-composed
//! (ln v = 1 - EML(0, v)), so the final formulas contain nothing beyond
//! EML, Plus, Times, Neg, variables and numbers.

use crate::config::SearchConfig;
use crate::core::build;
use crate::core::expression::Expression;
use crate::core::value::real;
use crate::engine::bfs;
use crate::engine::fable;
use crate::engine::optimizer;
use crate::error::EmlError;
use crate::ops::registry::OperatorRegistry;
use crate::result::SearchResult;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn std_dev(v: &[f64]) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    (v.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / v.len() as f64).sqrt()
}

/// Dense linear least squares via normal equations (trace-scaled ridge).
fn lstsq(design: &[Vec<f64>], target: &[f64]) -> Option<Vec<f64>> {
    let n = design.len();
    if n == 0 {
        return None;
    }
    let m = design[0].len();
    if n < m {
        return None;
    }
    let mut ata = vec![vec![0.0f64; m]; m];
    let mut atb = vec![0.0f64; m];
    for (row, &t) in design.iter().zip(target) {
        for i in 0..m {
            for j in i..m {
                ata[i][j] += row[i] * row[j];
            }
            atb[i] += row[i] * t;
        }
    }
    let trace: f64 = (0..m).map(|i| ata[i][i]).sum();
    let ridge = 1e-12 * (trace / m as f64).max(1.0);
    for i in 0..m {
        for j in 0..i {
            ata[i][j] = ata[j][i];
        }
        ata[i][i] += ridge;
    }
    // Gauss-Jordan.
    for col in 0..m {
        let mut pivot = col;
        for row in (col + 1)..m {
            if ata[row][col].abs() > ata[pivot][col].abs() {
                pivot = row;
            }
        }
        if ata[pivot][col].abs() < 1e-300 {
            return None;
        }
        ata.swap(col, pivot);
        atb.swap(col, pivot);
        let div = ata[col][col];
        for j in col..m {
            ata[col][j] /= div;
        }
        atb[col] /= div;
        for row in 0..m {
            if row == col {
                continue;
            }
            let factor = ata[row][col];
            if factor == 0.0 {
                continue;
            }
            for j in col..m {
                ata[row][j] -= factor * ata[col][j];
            }
            atb[row] -= factor * atb[col];
        }
    }
    Some(atb)
}

/// ln(v) composed purely from EML: ln v = 1 - EML(0, v).
fn eml_ln(v: Expression, reg: &OperatorRegistry) -> Expression {
    build::binary(
        "Plus",
        build::num(1.0),
        build::unary(
            "Neg",
            build::binary("EML", build::num(0.0), v, reg),
            reg,
        ),
        reg,
    )
}

/// Log-augmented affine argument with tunable parameters:
/// P(x) = p0 + sum_{i in xs} p_i x_i + sum_{j in lns} q_j ln(x_j).
/// The ln factors are EML-composed so the tree stays inside the pure
/// grammar. Parameter initial values come from the caller.
fn affine_arg(
    xs: &[usize],
    lns: &[usize],
    inits: &AffineInits,
    reg: &OperatorRegistry,
) -> Expression {
    let mut acc = Expression::parameter_with(inits.intercept);
    for (k, &i) in xs.iter().enumerate() {
        let term = build::binary(
            "Times",
            Expression::parameter_with(inits.x_coeffs.get(k).copied().unwrap_or(0.0)),
            build::var(i),
            reg,
        );
        acc = build::binary("Plus", acc, term, reg);
    }
    for (k, &j) in lns.iter().enumerate() {
        let term = build::binary(
            "Times",
            Expression::parameter_with(inits.ln_coeffs.get(k).copied().unwrap_or(0.0)),
            eml_ln(build::var(j), reg),
            reg,
        );
        acc = build::binary("Plus", acc, term, reg);
    }
    acc
}

#[derive(Clone, Default)]
struct AffineInits {
    intercept: f64,
    x_coeffs: Vec<f64>,
    ln_coeffs: Vec<f64>,
}

/// The unit templates the basis boosting can fit.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Template {
    /// c * EML(P(x), 1) = c * e^{P}: monomial x exponential class.
    ExpUnit,
    /// c * EML(0, 1 + A(x)*B(x)) = c * (1 - ln(1 + A*B)): logarithmic class.
    LogUnit,
    /// c * EML(P(x), 1 + A(x)*B(x)): both at once.
    FullUnit,
}

/// Builds one parametric unit (leading scale included as a parameter).
fn build_unit(
    t: Template,
    xs: &[usize],
    lns: &[usize],
    scale_init: f64,
    p_inits: &AffineInits,
    q_inits: &AffineInits,
    reg: &OperatorRegistry,
) -> Expression {
    let one_plus_prod = |reg: &OperatorRegistry| {
        let a = affine_arg(xs, &[], q_inits, reg);
        let b = affine_arg(xs, &[], q_inits, reg);
        build::binary(
            "Plus",
            build::num(1.0),
            build::binary("Times", a, b, reg),
            reg,
        )
    };
    let core = match t {
        Template::ExpUnit => build::binary(
            "EML",
            affine_arg(xs, lns, p_inits, reg),
            build::num(1.0),
            reg,
        ),
        Template::LogUnit => build::binary("EML", build::num(0.0), one_plus_prod(reg), reg),
        Template::FullUnit => build::binary(
            "EML",
            affine_arg(xs, lns, p_inits, reg),
            one_plus_prod(reg),
            reg,
        ),
    };
    build::binary("Times", Expression::parameter_with(scale_init), core, reg)
}

/// Ranks variables by relevance to the residual so the unit arguments stay
/// small in high-dimensional data: max of |corr(x_i, r)| and, when defined,
/// |corr(ln x_i, ln |r|)|.
fn select_variables(inputs: &[Vec<f64>], residual: &[f64], max_vars: usize) -> Vec<usize> {
    let d = inputs.first().map_or(0, |r| r.len());
    let n = residual.len();
    if d == 0 || n < 4 {
        return Vec::new();
    }
    let corr = |a: &[f64], b: &[f64]| -> f64 {
        let nf = a.len() as f64;
        let ma = a.iter().sum::<f64>() / nf;
        let mb = b.iter().sum::<f64>() / nf;
        let mut cov = 0.0;
        let mut va = 0.0;
        let mut vb = 0.0;
        for (x, y) in a.iter().zip(b) {
            cov += (x - ma) * (y - mb);
            va += (x - ma) * (x - ma);
            vb += (y - mb) * (y - mb);
        }
        if va <= 0.0 || vb <= 0.0 {
            0.0
        } else {
            (cov / (va.sqrt() * vb.sqrt())).abs()
        }
    };
    let r_scale = residual.iter().fold(0.0f64, |m, v| m.max(v.abs()));
    let ln_r: Vec<f64> = residual
        .iter()
        .map(|&v| v.abs().max(1e-13 * r_scale.max(1e-300)).ln())
        .collect();
    let mut ranked: Vec<(usize, f64)> = (0..d)
        .map(|i| {
            let col: Vec<f64> = inputs.iter().map(|row| row[i]).collect();
            let mut score = corr(&col, residual);
            if col.iter().all(|&v| v > 0.0) {
                let ln_col: Vec<f64> = col.iter().map(|v| v.ln()).collect();
                score = score.max(corr(&ln_col, &ln_r));
            }
            (i, score)
        })
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let mut vars: Vec<usize> = ranked.into_iter().take(max_vars).map(|(i, _)| i).collect();
    vars.sort_unstable();
    vars
}

/// Log-space initialisation for the ExpUnit: fits
/// ln |r| ~ p0 + sum a_i x_i + sum b_j ln x_j on the rows where the
/// residual is meaningfully nonzero, exactly the structure e^{P} produces.
fn exp_unit_inits(
    inputs: &[Vec<f64>],
    residual: &[f64],
    xs: &[usize],
    lns: &[usize],
) -> (f64, AffineInits) {
    let r_scale = residual.iter().fold(0.0f64, |m, v| m.max(v.abs()));
    let rows: Vec<usize> = (0..residual.len())
        .filter(|&i| residual[i].abs() > 1e-10 * r_scale.max(1e-300))
        .collect();
    let sign = if rows.iter().map(|&i| residual[i].signum()).sum::<f64>() >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let mut inits = AffineInits {
        intercept: 0.0,
        x_coeffs: vec![0.0; xs.len()],
        ln_coeffs: vec![0.0; lns.len()],
    };
    if rows.len() >= xs.len() + lns.len() + 2 {
        let design: Vec<Vec<f64>> = rows
            .iter()
            .map(|&i| {
                let mut row = Vec::with_capacity(1 + xs.len() + lns.len());
                row.push(1.0);
                for &v in xs {
                    row.push(inputs[i][v]);
                }
                for &v in lns {
                    row.push(inputs[i][v].max(1e-300).ln());
                }
                row
            })
            .collect();
        let target: Vec<f64> = rows.iter().map(|&i| residual[i].abs().ln()).collect();
        if let Some(sol) = lstsq(&design, &target) {
            if sol.iter().all(|v| v.is_finite()) {
                inits.intercept = sol[0];
                inits.x_coeffs = sol[1..1 + xs.len()].to_vec();
                inits.ln_coeffs = sol[1 + xs.len()..].to_vec();
            }
        }
    }
    (sign, inits)
}

/// Fits `f = c0 + sum_k c_k EML(P_k, Q_k)` by greedy residual boosting.
/// Returns Pareto-ready (full-data error, expression) snapshots, one per
/// accepted unit count. Everything is expressed in the pure-EML grammar.
pub(crate) fn eml_basis_boost(
    inputs: &[Vec<f64>],
    ys: &[f64],
    config: &SearchConfig,
    reg: &Arc<OperatorRegistry>,
    deadline: Option<Instant>,
) -> Vec<(f64, Expression)> {
    let n = ys.len();
    let mut results: Vec<(f64, Expression)> = Vec::new();
    if n < 20 || inputs.is_empty() || inputs[0].is_empty() {
        return results;
    }
    let y_std = std_dev(ys).max(1e-30);

    // Deterministic subsample for LM; deterministic 80/20 split for gating.
    let sub_n = n.min(config.subsample_size.max(64));
    let sub_rows: Vec<usize> = (0..sub_n).map(|i| i * n / sub_n).collect();
    let sub_inputs: Vec<Vec<crate::core::value::Value>> = sub_rows
        .iter()
        .map(|&i| inputs[i].iter().map(|&v| real(v)).collect())
        .collect();
    let val_rows: Vec<usize> = (0..n).filter(|i| i % 5 == 4).collect();

    let positive: Vec<usize> = (0..inputs[0].len())
        .filter(|&j| inputs.iter().all(|row| row[j] > 1e-12 && row[j].is_finite()))
        .collect();

    // Model state: fitted unit expressions and their full-data predictions.
    let mut unit_exprs: Vec<Expression> = Vec::new();
    let mut unit_preds: Vec<Vec<f64>> = Vec::new();

    // Assembles c0 + sum c_k u_k with OLS outer coefficients; returns the
    // materialized expression, its predictions and the validation RMSE.
    let assemble = |unit_exprs: &[Expression],
                    unit_preds: &[Vec<f64>]|
     -> Option<(Expression, Vec<f64>, f64, f64)> {
        let design: Vec<Vec<f64>> = (0..n)
            .map(|i| {
                let mut row = Vec::with_capacity(unit_preds.len() + 1);
                row.push(1.0);
                for u in unit_preds {
                    row.push(u[i]);
                }
                row
            })
            .collect();
        let coeffs = lstsq(&design, ys)?;
        if coeffs.iter().any(|c| !c.is_finite()) {
            return None;
        }
        let preds: Vec<f64> = (0..n)
            .map(|i| {
                coeffs[0]
                    + unit_preds
                        .iter()
                        .zip(&coeffs[1..])
                        .map(|(u, c)| c * u[i])
                        .sum::<f64>()
            })
            .collect();
        let mut expr = build::num(coeffs[0]);
        for (u, &c) in unit_exprs.iter().zip(&coeffs[1..]) {
            expr = build::binary(
                "Plus",
                expr,
                build::binary("Times", build::num(c), u.clone(), reg),
                reg,
            );
        }
        let full_err = {
            let mut acc = 0.0;
            for (p, &y) in preds.iter().zip(ys) {
                let d = p - y;
                acc += d * d;
            }
            (acc / n as f64).sqrt()
        };
        let val_err = {
            let mut acc = 0.0;
            for &i in &val_rows {
                let d = preds[i] - ys[i];
                acc += d * d;
            }
            (acc / val_rows.len().max(1) as f64).sqrt()
        };
        Some((expr, preds, full_err, val_err))
    };

    let mut residual: Vec<f64> = ys.to_vec();
    let mut current_val = {
        let mean = ys.iter().sum::<f64>() / n as f64;
        let mut acc = 0.0;
        for &i in &val_rows {
            let d = ys[i] - mean;
            acc += d * d;
        }
        (acc / val_rows.len().max(1) as f64).sqrt()
    };

    for _k in 0..4 {
        if deadline.map_or(false, |d| Instant::now() >= d) {
            break;
        }
        // Variable subsets sized to keep the LM parameter count modest.
        let vars = select_variables(inputs, &residual, 5);
        if vars.is_empty() {
            break;
        }
        let lns: Vec<usize> = vars
            .iter()
            .copied()
            .filter(|v| positive.contains(v))
            .collect();
        let sub_residual: Vec<f64> = sub_rows.iter().map(|&i| residual[i]).collect();

        // Candidate templates with data-driven initialisation.
        let (sign, exp_inits) = exp_unit_inits(inputs, &residual, &vars, &lns);
        let r_std = std_dev(&residual).max(1e-30);
        let q_inits = AffineInits {
            intercept: 0.1,
            x_coeffs: vars
                .iter()
                .map(|&i| {
                    let col_std =
                        std_dev(&inputs.iter().map(|row| row[i]).collect::<Vec<f64>>());
                    if col_std > 0.0 {
                        1.0 / col_std
                    } else {
                        0.0
                    }
                })
                .collect(),
            ln_coeffs: Vec::new(),
        };
        let mut candidates = vec![
            build_unit(Template::ExpUnit, &vars, &lns, sign, &exp_inits, &q_inits, reg),
            build_unit(
                Template::ExpUnit,
                &vars,
                &[],
                sign,
                &AffineInits {
                    intercept: exp_inits.intercept,
                    x_coeffs: exp_inits.x_coeffs.clone(),
                    ln_coeffs: Vec::new(),
                },
                &q_inits,
                reg,
            ),
            build_unit(Template::LogUnit, &vars, &[], r_std, &exp_inits, &q_inits, reg),
            build_unit(Template::FullUnit, &vars, &lns, sign, &exp_inits, &q_inits, reg),
        ];
        // Single-variable ExpUnits: the full-variable log-space fit often
        // locks onto a "compromise" monomial when the target is a sum of
        // structurally different terms; per-variable starts escape it
        // (the same multi-start idea Stage A uses in the fable pipeline).
        for &v in &vars {
            let single = [v];
            let single_ln: Vec<usize> = if positive.contains(&v) { vec![v] } else { vec![] };
            let (s_sign, s_inits) = exp_unit_inits(inputs, &residual, &single, &single_ln);
            candidates.push(build_unit(
                Template::ExpUnit,
                &single,
                &single_ln,
                s_sign,
                &s_inits,
                &q_inits,
                reg,
            ));
        }

        // Fit each template to the residual with multi-start LM; pick the
        // winner by the HELD-OUT error of the tentatively assembled model
        // (training error alone favors compromise fits).
        let mut best: Option<(f64, f64, Expression, Vec<f64>)> = None;
        for cand in candidates {
            if deadline.map_or(false, |d| Instant::now() >= d) {
                break;
            }
            let (fitted, err) = optimizer::refine_constants_multi(
                &cand,
                &sub_inputs,
                &sub_residual,
                reg,
                config.refine_max_iters.max(60),
                3,
            );
            if !err.is_finite() {
                continue;
            }
            let snapped = if config.snap_constants {
                optimizer::snap_constants(&fitted, &sub_inputs, &sub_residual, reg)
            } else {
                fitted
            };
            let unit = fable::params_to_literals(&snapped, reg);

            // Full-data predictions of the raw unit.
            let mut preds = Vec::with_capacity(n);
            let mut ok = true;
            for row in inputs {
                let vals: Vec<crate::core::value::Value> =
                    row.iter().map(|&v| real(v)).collect();
                match unit.eval(&vals, reg) {
                    Some(v)
                        if crate::core::value::is_usable(v)
                            && v.im.abs() < 1e-6 * v.re.abs().max(1.0) =>
                    {
                        preds.push(v.re)
                    }
                    _ => {
                        ok = false;
                        break;
                    }
                }
            }
            if !ok {
                continue;
            }
            unit_preds.push(preds);
            let scored = assemble(&unit_exprs_with(&unit_exprs, &unit), &unit_preds);
            let preds = unit_preds.pop().unwrap();
            if let Some((_, _, full_err, val_err)) = scored {
                if best
                    .as_ref()
                    .map_or(true, |(bv, _, _, _)| val_err < *bv)
                    && full_err.is_finite()
                {
                    best = Some((val_err, full_err, unit, preds));
                }
            }
        }
        let (best_val, _, unit, preds) = match best {
            Some(b) => b,
            None => break,
        };
        // Accept only on clear held-out improvement.
        if best_val >= current_val * 0.98 {
            break;
        }
        unit_exprs.push(unit);
        unit_preds.push(preds);
        match assemble(&unit_exprs, &unit_preds) {
            Some((expr, preds_all, full_err, val_err)) => {
                current_val = val_err;
                residual = ys.iter().zip(&preds_all).map(|(y, p)| y - p).collect();
                results.push((full_err, expr));
                if full_err < config.early_exit_threshold.max(1e-12) * y_std {
                    break;
                }
            }
            None => {
                unit_exprs.pop();
                unit_preds.pop();
                break;
            }
        }
    }

    results
}

/// `existing + [unit]` without cloning the whole list twice.
fn unit_exprs_with(existing: &[Expression], unit: &Expression) -> Vec<Expression> {
    let mut v = Vec::with_capacity(existing.len() + 1);
    v.extend_from_slice(existing);
    v.push(unit.clone());
    v
}

/// Runs the complete pure-EML search pipeline.
pub fn run_pure_eml(
    inputs: &[Vec<f64>],
    ys: &[f64],
    config: &SearchConfig,
) -> Result<Vec<SearchResult>, EmlError> {
    if inputs.is_empty() || ys.is_empty() || inputs.len() != ys.len() {
        return Err(EmlError::invalid(
            "Inputs and target vector must be non-empty and equal length.",
        ));
    }
    let registry = Arc::new(OperatorRegistry::pure_eml());
    let start = Instant::now();
    let deadline = if config.time_budget_s > 0.0 {
        Some(start + Duration::from_secs_f64(config.time_budget_s))
    } else {
        None
    };
    let y_std = std_dev(ys).max(1e-30);
    let solved = |err: f64| err <= config.early_exit_threshold * y_std;

    let mut pool: Vec<(f64, Expression)> = Vec::new();

    // ---- Stage P1: EML basis boosting ----
    if config.pure_basis_stage {
    let t0 = Instant::now();
    let p1_deadline = Some(
        deadline
            .unwrap_or(t0 + Duration::from_secs(30))
            .min(t0 + Duration::from_secs(30)),
    );
    let fits = eml_basis_boost(inputs, ys, config, &registry, p1_deadline);
    if config.verbose && !fits.is_empty() {
        let best = fits.iter().map(|(e, _)| *e).fold(f64::INFINITY, f64::min);
        println!(
            "[EML-SR-Pure] Basis boosting produced {} candidates (best RMSE {:.3e}).",
            fits.len(),
            best
        );
    }
    pool.extend(fits);
    }

    let best_so_far = pool.iter().map(|(e, _)| *e).fold(f64::INFINITY, f64::min);
    if solved(best_so_far) {
        if config.verbose {
            println!("[EML-SR-Pure] Basis boosting solved the dataset; skipping beam search.");
        }
        return Ok(fable::finalize_pool(pool, inputs, ys, config, &registry));
    }

    // ---- Stage P2: pure-EML beam search (lookahead + behavior diversity) ----
    let mut sub_config = config.clone();
    sub_config.lookahead_scoring = true;
    if sub_config.behavior_cap == 0 {
        sub_config.behavior_cap = 32;
    }
    match bfs::run_bfs_front(inputs, ys, &sub_config, deadline, &registry) {
        Ok(entries) => pool.extend(entries),
        Err(e) => {
            if pool.is_empty() {
                return Err(e);
            }
        }
    }

    // ---- Width re-expansion: if plenty of budget remains and the target is
    // not yet reached, restart the beam twice as wide. ----
    if let Some(dl) = deadline {
        let best = pool.iter().map(|(e, _)| *e).fold(f64::INFINITY, f64::min);
        let remaining = dl.saturating_duration_since(Instant::now());
        if !solved(best) && remaining.as_secs_f64() > 0.3 * config.time_budget_s {
            let mut wide = sub_config.clone();
            wide.beam_width = sub_config.beam_width * 2;
            if config.verbose {
                println!(
                    "[EML-SR-Pure] Budget remains; retrying with beam width {}.",
                    wide.beam_width
                );
            }
            if let Ok(entries) = bfs::run_bfs_front(inputs, ys, &wide, deadline, &registry) {
                pool.extend(entries);
            }
        }
    }

    if pool.is_empty() {
        return Err(EmlError::NotFound {
            max_complexity: config.max_complexity,
        });
    }

    // ---- Residual boosting: one more basis pass on the residual of the
    // best candidate (validation-gated inside the assembler). ----
    let (best_err, best_expr) = pool
        .iter()
        .filter(|(e, _)| e.is_finite())
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .map(|(e, x)| (*e, x.clone()))
        .unwrap_or((f64::INFINITY, build::num(0.0)));
    if config.pure_basis_stage && best_err.is_finite() && best_err > 1e-4 * y_std && best_err < 0.2 * y_std {
        let mut res = Vec::with_capacity(ys.len());
        let mut ok = true;
        for (row, &y) in inputs.iter().zip(ys) {
            let vals: Vec<crate::core::value::Value> = row.iter().map(|&v| real(v)).collect();
            match best_expr.eval(&vals, &registry) {
                Some(v)
                    if crate::core::value::is_usable(v)
                        && v.im.abs() < 1e-6 * v.re.abs().max(1.0) =>
                {
                    res.push(y - v.re)
                }
                _ => {
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            let t1 = Instant::now();
            let boost_deadline = Some(
                deadline
                    .unwrap_or(t1 + Duration::from_secs(10))
                    .min(t1 + Duration::from_secs(10)),
            );
            for (_, g) in eml_basis_boost(inputs, &res, config, &registry, boost_deadline) {
                let combined = build::binary("Plus", best_expr.clone(), g, &registry);
                let full = fable::expr_error(&combined, inputs, ys, &registry);
                if full.is_finite() && full < best_err {
                    pool.push((full, combined));
                }
            }
        }
    }

    if config.verbose {
        let best = pool.iter().map(|(e, _)| *e).fold(f64::INFINITY, f64::min);
        println!(
            "[EML-SR-Pure] Pipeline finished in {:?}; best full-data RMSE {:.3e}.",
            start.elapsed(),
            best
        );
    }

    Ok(fable::finalize_pool(pool, inputs, ys, config, &registry))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lcg(seed: &mut u64) -> f64 {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((*seed >> 33) as f64) / (u64::MAX >> 33) as f64
    }

    fn gauss(seed: &mut u64) -> f64 {
        let u1 = lcg(seed).max(1e-12);
        let u2 = lcg(seed);
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }

    fn noisy(clean: &[f64], seed: &mut u64) -> (Vec<f64>, f64) {
        let sigma = 0.01 * std_dev(clean);
        (
            clean.iter().map(|&c| c + sigma * gauss(seed)).collect(),
            sigma,
        )
    }

    fn clean_rmse(results: &[SearchResult], inputs: &[Vec<f64>], clean: &[f64]) -> f64 {
        results
            .iter()
            .map(|r| {
                let mut acc = 0.0;
                for (row, &c) in inputs.iter().zip(clean) {
                    let p = r.eval_multi(row);
                    if !p.is_finite() {
                        return f64::INFINITY;
                    }
                    let d = p - c;
                    acc += d * d;
                }
                (acc / clean.len() as f64).sqrt()
            })
            .fold(f64::INFINITY, f64::min)
    }

    fn pure_config() -> SearchConfig {
        let mut c = SearchConfig::fable_default();
        c.pure_eml = true;
        c.max_complexity = 8;
        c.beam_width = 400;
        c.time_budget_s = 40.0;
        c.early_exit_threshold = 9e-3;
        c.verbose = false;
        c
    }

    /// (a) exponential decay: one ExpUnit.
    #[test]
    fn pure_recovers_exp_decay() {
        let mut seed = 41u64;
        let inputs: Vec<Vec<f64>> = (0..500).map(|_| vec![3.0 * lcg(&mut seed)]).collect();
        let clean: Vec<f64> = inputs.iter().map(|r| 3.0 * (-2.0 * r[0]).exp() + 1.0).collect();
        let (ys, sigma) = noisy(&clean, &mut seed);
        let results = run_pure_eml(&inputs, &ys, &pure_config()).expect("search");
        let err = clean_rmse(&results, &inputs, &clean);
        assert!(err < 0.5 * sigma, "exp decay not recovered: {} vs sigma {}", err, sigma);
    }

    /// (b) monomial sum via the log-augmented first argument.
    #[test]
    fn pure_recovers_monomial_sum() {
        let mut seed = 43u64;
        let inputs: Vec<Vec<f64>> = (0..600)
            .map(|_| {
                vec![
                    0.5 + 2.5 * lcg(&mut seed),
                    0.5 + 2.5 * lcg(&mut seed),
                    0.5 + 2.5 * lcg(&mut seed),
                ]
            })
            .collect();
        let clean: Vec<f64> = inputs.iter().map(|r| r[0] * r[0] * r[1] + 2.0 * r[2]).collect();
        let (ys, sigma) = noisy(&clean, &mut seed);
        let results = run_pure_eml(&inputs, &ys, &pure_config()).expect("search");
        let err = clean_rmse(&results, &inputs, &clean);
        assert!(err < 0.5 * sigma, "monomial sum not recovered: {} vs sigma {}", err, sigma);
    }

    /// (c) logarithmic shape via the LogUnit.
    #[test]
    fn pure_recovers_log_shape() {
        let mut seed = 47u64;
        let inputs: Vec<Vec<f64>> = (0..500).map(|_| vec![-2.0 + 4.0 * lcg(&mut seed)]).collect();
        let clean: Vec<f64> = inputs.iter().map(|r| 2.0 - (1.0 + r[0] * r[0]).ln()).collect();
        let (ys, sigma) = noisy(&clean, &mut seed);
        let results = run_pure_eml(&inputs, &ys, &pure_config()).expect("search");
        let err = clean_rmse(&results, &inputs, &clean);
        assert!(err < 0.5 * sigma, "log shape not recovered: {} vs sigma {}", err, sigma);
    }

    /// (d) plain product via the pure beam.
    #[test]
    fn pure_beam_recovers_product() {
        let mut seed = 53u64;
        let inputs: Vec<Vec<f64>> = (0..400)
            .map(|_| vec![-2.0 + 4.0 * lcg(&mut seed), -2.0 + 4.0 * lcg(&mut seed)])
            .collect();
        let clean: Vec<f64> = inputs.iter().map(|r| r[0] * r[1]).collect();
        let (ys, sigma) = noisy(&clean, &mut seed);
        let results = run_pure_eml(&inputs, &ys, &pure_config()).expect("search");
        let err = clean_rmse(&results, &inputs, &clean);
        assert!(err < 0.5 * sigma, "product not recovered: {} vs sigma {}", err, sigma);
    }

    /// (e) the output grammar is pure: nothing beyond EML + arithmetic glue.
    #[test]
    fn pure_output_grammar_is_pure() {
        let mut seed = 59u64;
        let inputs: Vec<Vec<f64>> = (0..400)
            .map(|_| vec![0.5 + 2.0 * lcg(&mut seed), 0.5 + 2.0 * lcg(&mut seed)])
            .collect();
        let clean: Vec<f64> = inputs.iter().map(|r| r[0] / r[1]).collect();
        let (ys, _) = noisy(&clean, &mut seed);
        let results = run_pure_eml(&inputs, &ys, &pure_config()).expect("search");
        for r in &results {
            let f = r.formula();
            for forbidden in [
                "Sin", "Cos", "Tan", "Exp(", "Log(", "Sqrt", "Square", "Cube", "Pow(",
                "Divide", "Inv(", "Abs", "Sigmoid", "Min(", "Max(", "Subtract",
            ] {
                assert!(
                    !f.contains(forbidden),
                    "non-pure operator {} in formula: {}",
                    forbidden,
                    f
                );
            }
        }
    }

    /// (f) lookahead ranking: a hidden building block scores well once a
    /// wrapper is considered, while its direct affine fit is poor.
    #[test]
    fn lookahead_rescues_building_block() {
        let mut seed = 61u64;
        let mut preds = Vec::new();
        let mut targets = Vec::new();
        for _ in 0..200 {
            let s = 0.5 + 3.0 * lcg(&mut seed); // building block f = x0 + x1 > 0
            preds.push(s);
            targets.push(1.0 / s); // y = 1/(x0 + x1)
        }
        let direct = {
            let (_, e) = {
                // affine fit of raw preds
                let n = preds.len() as f64;
                let mp = preds.iter().sum::<f64>() / n;
                let my = targets.iter().sum::<f64>() / n;
                let mut cov = 0.0;
                let mut var = 0.0;
                for (p, y) in preds.iter().zip(&targets) {
                    cov += (p - mp) * (y - my);
                    var += (p - mp) * (p - mp);
                }
                let a = cov / var;
                let b = my - a * mp;
                let mut acc = 0.0;
                for (p, y) in preds.iter().zip(&targets) {
                    let d = a * p + b - y;
                    acc += d * d;
                }
                ((), (acc / n).sqrt())
            };
            e
        };
        let look = bfs::lookahead_error(&preds, &targets);
        assert!(
            look < 1e-10 && direct > 1e-3,
            "lookahead failed: look={} direct={}",
            look,
            direct
        );
    }

    /// (g) behavior keys separate shapes but unify affine variants.
    #[test]
    fn behavior_key_buckets() {
        let xs: Vec<f64> = (0..64).map(|i| -2.0 + 4.0 * (i as f64) / 63.0).collect();
        let f1: Vec<f64> = xs.iter().map(|&x| x * x).collect();
        let f1_affine: Vec<f64> = xs.iter().map(|&x| -3.0 * x * x + 7.0).collect();
        let f2: Vec<f64> = xs.iter().map(|&x| x * x * x).collect();
        assert_eq!(bfs::behavior_key(&f1), bfs::behavior_key(&f1_affine));
        assert_ne!(bfs::behavior_key(&f1), bfs::behavior_key(&f2));
    }
}
