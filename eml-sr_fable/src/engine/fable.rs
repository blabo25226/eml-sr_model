//! The fable pipeline: Stage A (closed-form power laws) → Stage C
//! (multiplicative decomposition / ratio search) → Stage B (plain EML beam
//! search), with all candidates merged into one Pareto front.

use crate::config::SearchConfig;
use crate::core::build;
use crate::core::expression::Expression;
use crate::core::value::{is_usable, real};
use crate::engine::bfs;
use crate::engine::powerlaw;
use crate::error::EmlError;
use crate::ops::registry::OperatorRegistry;
use crate::result::SearchResult;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Full-data RMSE of an assembled expression.
fn expr_error(
    expr: &Expression,
    inputs: &[Vec<f64>],
    ys: &[f64],
    reg: &OperatorRegistry,
) -> f64 {
    let mut acc = 0.0;
    for (row, &y) in inputs.iter().zip(ys) {
        let vals: Vec<crate::core::value::Value> = row.iter().map(|&v| real(v)).collect();
        match expr.eval(&vals, reg) {
            Some(v) if is_usable(v) && v.im.abs() < 1e-6 * v.re.abs().max(1.0) => {
                let d = v.re - y;
                acc += d * d;
            }
            _ => return f64::INFINITY,
        }
    }
    (acc / ys.len() as f64).sqrt()
}

fn deadline_passed(deadline: Option<Instant>) -> bool {
    deadline.map_or(false, |d| Instant::now() >= d)
}

/// Rebuilds the canonical display string from the RPN nodes (used after
/// parameter materialization, when the stored display would be stale).
fn redisplay(expr: &Expression, reg: &OperatorRegistry) -> String {
    use crate::core::expression::Node;
    let mut stack: Vec<String> = Vec::with_capacity(expr.complexity());
    for node in &expr.nodes {
        match node {
            Node::Const { op_id, .. } => stack.push(reg.meta(*op_id).name.to_string()),
            Node::Num(v) => stack.push(build::fmt_num(*v)),
            Node::Var(i) => stack.push(format!("v_{{{i}}}")),
            Node::Param { initial_value, .. } => stack.push(build::fmt_num(initial_value.re)),
            Node::Op { op_id, arity } => {
                let arity = *arity as usize;
                let start = stack.len().saturating_sub(arity);
                let args: Vec<String> = stack.drain(start..).collect();
                stack.push(format!("{}({})", reg.meta(*op_id).name, args.join(", ")));
            }
        }
    }
    stack.pop().unwrap_or_default()
}

/// Converts every numeric literal (and existing parameter) into a tunable
/// `Param` with node-order ids, so LM can re-optimize all constants of a
/// closed-form candidate against the raw target. Returns `None` when there
/// is nothing to tune or when the parameter count would make LM too slow.
fn literals_to_params(expr: &Expression) -> Option<Expression> {
    use crate::core::expression::Node;
    let mut nodes = expr.nodes.clone();
    let mut count: usize = 0;
    for node in nodes.iter_mut() {
        match node {
            Node::Num(v) => {
                *node = Node::Param {
                    id: count as u8,
                    initial_value: real(*v),
                };
                count += 1;
            }
            Node::Param { id, .. } => {
                *id = count as u8;
                count += 1;
            }
            _ => {}
        }
    }
    if count == 0 || count > 16 {
        return None;
    }
    Some(Expression::new(
        nodes,
        expr.var_count(),
        count as u8,
        expr.display().to_string(),
    ))
}

/// Materializes every `Param` back into a numeric literal and regenerates
/// the display string.
fn params_to_literals(expr: &Expression, reg: &OperatorRegistry) -> Expression {
    use crate::core::expression::Node;
    let nodes: Vec<Node> = expr
        .nodes
        .iter()
        .map(|node| match node {
            Node::Param { initial_value, .. } => Node::Num(initial_value.re),
            other => other.clone(),
        })
        .collect();
    let out = Expression::new(nodes, expr.var_count(), 0, String::new());
    let display = redisplay(&out, reg);
    Expression::new(out.nodes.clone(), out.var_count(), 0, display)
}

/// Raw-space LM polish (P1-2): the closed-form stages fit their coefficients
/// in a transformed space (log y, 1/y^2, y*Q = P, ...), where least squares
/// is biased with respect to the raw residual once the target is noisy.
/// Re-optimizing every constant of the leading candidates directly against
/// y removes that bias for whichever stage produced them.
fn polish_pool(
    pool: &mut Vec<(f64, Expression)>,
    inputs: &[Vec<f64>],
    ys: &[f64],
    config: &SearchConfig,
    registry: &Arc<OperatorRegistry>,
) {
    if pool.is_empty() {
        return;
    }
    let mut order: Vec<usize> = (0..pool.len()).collect();
    order.sort_by(|&a, &b| pool[a].0.partial_cmp(&pool[b].0).unwrap());

    let n = ys.len();
    let sub_n = n.min(config.subsample_size.max(64));
    let rows: Vec<usize> = (0..sub_n).map(|i| i * n / sub_n).collect();
    let sub_inputs: Vec<Vec<crate::core::value::Value>> = rows
        .iter()
        .map(|&i| inputs[i].iter().map(|&v| real(v)).collect())
        .collect();
    let sub_targets: Vec<f64> = rows.iter().map(|&i| ys[i]).collect();

    let mut additions: Vec<(f64, Expression)> = Vec::new();
    for &idx in order.iter().take(8) {
        let (err, expr) = &pool[idx];
        if !err.is_finite() {
            continue;
        }
        let pexpr = match literals_to_params(expr) {
            Some(p) => p,
            None => continue,
        };
        let (refined, _sub_err) = crate::engine::optimizer::refine_constants(
            &pexpr,
            &sub_inputs,
            &sub_targets,
            registry,
            60,
        );
        let materialized = params_to_literals(&refined, registry);
        let full = expr_error(&materialized, inputs, ys, registry);
        if full.is_finite() && full < *err * (1.0 - 1e-12) {
            additions.push((full, materialized));
        }
    }
    pool.extend(additions);
}

/// Validation-ranked candidate filter (P4-1, one-standard-error rule).
///
/// The pool is ranked on a held-out 20% of the rows instead of the full
/// (training) RMSE, and a higher-complexity candidate survives only when it
/// improves the held-out error by more than one standard error over every
/// simpler survivor. Overfit candidates — great training error, mediocre
/// held-out error — are dropped here and never reach the caller.
fn val_pareto_filter(
    pool: Vec<(f64, Expression)>,
    inputs: &[Vec<f64>],
    ys: &[f64],
    reg: &OperatorRegistry,
) -> Vec<(f64, Expression)> {
    use std::collections::BTreeMap;
    let n = ys.len();
    let val_rows: Vec<usize> = (0..n).filter(|i| i % 5 == 4).collect();
    if val_rows.len() < 10 {
        return pool;
    }
    let val_inputs: Vec<Vec<f64>> = val_rows.iter().map(|&i| inputs[i].clone()).collect();
    let val_ys: Vec<f64> = val_rows.iter().map(|&i| ys[i]).collect();
    // Relative one-standard-error margin of an RMSE estimate from n_val rows.
    let eps = (1.0 / (2.0 * val_rows.len() as f64).sqrt()).clamp(0.01, 0.15);

    // Best validation error per complexity.
    let mut by_complexity: BTreeMap<usize, (f64, f64, Expression)> = BTreeMap::new();
    for (full_err, expr) in &pool {
        if !full_err.is_finite() {
            continue;
        }
        let val_err = expr_error(expr, &val_inputs, &val_ys, reg);
        if !val_err.is_finite() {
            continue;
        }
        let comp = expr.complexity();
        let insert = match by_complexity.get(&comp) {
            None => true,
            Some((prev, _, _)) => val_err < *prev,
        };
        if insert {
            by_complexity.insert(comp, (val_err, *full_err, expr.clone()));
        }
    }

    let mut survivors: Vec<(f64, Expression)> = Vec::new();
    let mut best_val = f64::INFINITY;
    for (_comp, (val_err, full_err, expr)) in by_complexity {
        if val_err < best_val * (1.0 - eps) || survivors.is_empty() {
            best_val = best_val.min(val_err);
            survivors.push((full_err, expr));
        }
    }
    if survivors.is_empty() {
        return pool;
    }
    survivors
}

/// Final pool handling shared by every pipeline exit: raw-space LM polish of
/// the leading candidates, validation-ranked filtering, Pareto merge.
fn finalize_pool(
    mut pool: Vec<(f64, Expression)>,
    inputs: &[Vec<f64>],
    ys: &[f64],
    config: &SearchConfig,
    registry: &Arc<OperatorRegistry>,
) -> Vec<SearchResult> {
    let y_std = std_dev(ys).max(1e-30);
    let best = pool.iter().map(|(e, _)| *e).fold(f64::INFINITY, f64::min);
    // Skip the polish when a candidate is already numerically exact.
    if best > 1e-12 * y_std {
        polish_pool(&mut pool, inputs, ys, config, registry);
    }
    let filtered = val_pareto_filter(pool, inputs, ys, registry);
    bfs::merge_pareto(filtered, registry)
}

fn std_dev(v: &[f64]) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    (v.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / v.len() as f64).sqrt()
}

/// Runs the complete fable search pipeline.
pub fn run_fable(
    inputs: &[Vec<f64>],
    ys: &[f64],
    config: &SearchConfig,
) -> Result<Vec<SearchResult>, EmlError> {
    if inputs.is_empty() || ys.is_empty() || inputs.len() != ys.len() {
        return Err(EmlError::invalid(
            "Inputs and target vector must be non-empty and equal length.",
        ));
    }

    let registry = Arc::new(OperatorRegistry::with_builtins());
    let start = Instant::now();
    let deadline = if config.time_budget_s > 0.0 {
        Some(start + Duration::from_secs_f64(config.time_budget_s))
    } else {
        None
    };
    let y_std = std_dev(ys).max(1e-30);
    let solved = |err: f64| err <= config.early_exit_threshold * y_std;

    // Pool of (full-data error, assembled expression) across all stages.
    let mut pool: Vec<(f64, Expression)> = Vec::new();

    // ---- Stage A: closed-form power-law / monomial-sum solver ----
    if config.powerlaw_stage {
        let t0 = Instant::now();
        let stage_a_deadline = Some(
            deadline
                .unwrap_or(t0 + Duration::from_secs(30))
                .min(t0 + Duration::from_secs(30)),
        );
        let fits = powerlaw::run_powerlaw(inputs, ys, config, &registry, stage_a_deadline);
        if config.verbose && !fits.is_empty() {
            println!(
                "[EML-SR-Fable] Stage A produced {} candidates in {:?} (best RMSE {:.3e}).",
                fits.len(),
                t0.elapsed(),
                fits[0].error
            );
        }
        for fit in fits {
            pool.push((fit.error, fit.expression));
        }
    }

    let best_so_far = pool
        .iter()
        .map(|(e, _)| *e)
        .fold(f64::INFINITY, f64::min);
    if solved(best_so_far) {
        if config.verbose {
            println!("[EML-SR-Fable] Stage A solved the dataset; skipping beam search.");
        }
        return Ok(finalize_pool(pool, inputs, ys, config, &registry));
    }

    // ---- Stage C: multiplicative decomposition (ratio search) ----
    // The log-fit whitener is ambiguous when the non-monomial factor leaks
    // exponent mass onto its own variables, so probe the leading candidates
    // with the cheap closed-form Stage A before spending beam-search budget.
    let mut ratio_solved = false;
    if config.ratio_search && !inputs[0].is_empty() {
        let candidates = powerlaw::monomial_whitener_candidates(inputs, ys);
        let ratio_deadline = deadline.map(|d| {
            let remaining = d.saturating_duration_since(Instant::now());
            Instant::now() + remaining / 2
        });

        let mut probes: Vec<(crate::core::build::Monomial, Vec<f64>)> = Vec::new();
        let mut seen_exps: Vec<Vec<f64>> = Vec::new();
        for whitener in candidates {
            if probes.len() >= 5 {
                break;
            }
            if !whitener.exponents.iter().any(|&e| e != 0.0) {
                continue;
            }
            if seen_exps.contains(&whitener.exponents) {
                continue;
            }
            seen_exps.push(whitener.exponents.clone());
            let m_vals = whitener.eval_rows(inputs);
            if !m_vals.iter().all(|v| v.is_finite() && v.abs() > 1e-300) {
                continue;
            }
            let ratios: Vec<f64> = ys.iter().zip(&m_vals).map(|(y, m)| y / m).collect();
            if ratios.iter().all(|r| r.is_finite()) {
                probes.push((whitener, ratios));
            }
        }

        // Stage A on each candidate ratio: corrections like exp(-monomial)
        // or exp(monomial)-1 exceed the beam complexity budget but
        // linearize under the power-law transforms (Log / Log1p).
        if config.powerlaw_stage {
            for (whitener, ratios) in &probes {
                if ratio_solved || deadline_passed(ratio_deadline) {
                    break;
                }
                let t0 = Instant::now();
                let a_deadline = Some(
                    ratio_deadline
                        .unwrap_or(t0 + Duration::from_secs(20))
                        .min(t0 + Duration::from_secs(20)),
                );
                if config.verbose {
                    println!(
                        "[EML-SR-Fable] Stage C: powerlaw probe of ratio vs {}.",
                        whitener.to_expression(&registry).display()
                    );
                }
                let m_expr = whitener.to_expression(&registry);
                let a_fits =
                    powerlaw::run_powerlaw(inputs, ratios, config, &registry, a_deadline);
                for fit in a_fits {
                    let combined =
                        build::binary("Times", m_expr.clone(), fit.expression, &registry);
                    let err = expr_error(&combined, inputs, ys, &registry);
                    if err.is_finite() {
                        if solved(err) {
                            ratio_solved = true;
                        }
                        pool.push((err, combined));
                    }
                }
            }
        }

        // Beam search on the best whitener's ratio when still unsolved.
        if !ratio_solved {
            if let Some((whitener, ratios)) = probes.first() {
                let mut sub_config = config.clone();
                sub_config.verbose = false;
                if config.verbose {
                    println!(
                        "[EML-SR-Fable] Stage C: ratio beam search vs {}.",
                        whitener.to_expression(&registry).display()
                    );
                }
                let m_expr = whitener.to_expression(&registry);
                if let Ok(entries) =
                    bfs::run_bfs_front(inputs, ratios, &sub_config, ratio_deadline, &registry)
                {
                    for (_, g_expr) in entries {
                        let combined =
                            build::binary("Times", m_expr.clone(), g_expr, &registry);
                        let err = expr_error(&combined, inputs, ys, &registry);
                        if err.is_finite() {
                            if solved(err) {
                                ratio_solved = true;
                            }
                            pool.push((err, combined));
                        }
                    }
                }
            }
        }
    }

    // ---- Stage A2: rational-function fit y ~= P(x)/Q(x) ----
    let mut rational_solved = false;
    if config.rational_stage && !ratio_solved {
        let t0 = Instant::now();
        let r_deadline = Some(
            deadline
                .unwrap_or(t0 + Duration::from_secs(25))
                .min(t0 + Duration::from_secs(25)),
        );
        let r_fits = powerlaw::run_rational(inputs, ys, config, &registry, r_deadline);
        if config.verbose && !r_fits.is_empty() {
            println!(
                "[EML-SR-Fable] Rational stage produced {} candidates (best RMSE {:.3e}).",
                r_fits.len(),
                r_fits[0].error
            );
        }
        for fit in r_fits {
            if solved(fit.error) {
                rational_solved = true;
            }
            pool.push((fit.error, fit.expression));
        }
    }

    // ---- Stage B: plain beam search on the raw target ----
    if !ratio_solved && !rational_solved {
        let mut sub_config = config.clone();
        sub_config.verbose = config.verbose;
        match bfs::run_bfs_front(inputs, ys, &sub_config, deadline, &registry) {
            Ok(entries) => {
                for (err, expr) in entries {
                    // run_bfs_front errors are already full-data after refinement.
                    pool.push((err, expr));
                }
            }
            Err(e) => {
                if pool.is_empty() {
                    return Err(e);
                }
            }
        }
    } else if config.verbose {
        println!("[EML-SR-Fable] Stage C solved the dataset; skipping plain beam search.");
    }

    if pool.is_empty() {
        return Err(EmlError::NotFound {
            max_complexity: config.max_complexity,
        });
    }

    // ---- Residual boosting (P3-1) ----
    // When the best candidate sits in the "close but not exact" band, the
    // structure is usually right but a smaller additive component is missing
    // (and often beyond the complexity budget of a single search). One extra
    // closed-form pass on the residual y - f1 recovers it. The combined
    // candidate is accepted only when the held-out error clearly improves,
    // so noise-chasing additions never make it into the pool.
    if config.powerlaw_stage {
        residual_boost(&mut pool, inputs, ys, config, &registry, deadline);
    }

    if config.verbose {
        let best = pool
            .iter()
            .map(|(e, _)| *e)
            .fold(f64::INFINITY, f64::min);
        println!(
            "[EML-SR-Fable] Pipeline finished in {:?}; best full-data RMSE {:.3e}.",
            start.elapsed(),
            best
        );
    }

    Ok(finalize_pool(pool, inputs, ys, config, &registry))
}

/// One round of closed-form residual boosting: fit Stage A on y - f1 for the
/// current best candidate f1 and add f1 + g when validation improves >= 10%.
fn residual_boost(
    pool: &mut Vec<(f64, Expression)>,
    inputs: &[Vec<f64>],
    ys: &[f64],
    config: &SearchConfig,
    registry: &Arc<OperatorRegistry>,
    deadline: Option<Instant>,
) {
    let y_std = std_dev(ys).max(1e-30);
    let (best_err, best_expr) = match pool
        .iter()
        .filter(|(e, _)| e.is_finite())
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
    {
        Some((e, x)) => (*e, x.clone()),
        None => return,
    };
    // Only the "close but not exact" band benefits: exact fits need nothing,
    // and a badly wrong f1 would just seed a junk correction.
    if !(best_err > 1e-4 * y_std && best_err < 0.2 * y_std) {
        return;
    }

    // Residual of f1 on all rows.
    let mut residual = Vec::with_capacity(ys.len());
    for (row, &y) in inputs.iter().zip(ys) {
        let vals: Vec<crate::core::value::Value> = row.iter().map(|&v| real(v)).collect();
        match best_expr.eval(&vals, registry) {
            Some(v) if is_usable(v) && v.im.abs() < 1e-6 * v.re.abs().max(1.0) => {
                residual.push(y - v.re);
            }
            _ => return,
        }
    }

    let t0 = Instant::now();
    let boost_deadline = Some(
        deadline
            .unwrap_or(t0 + Duration::from_secs(10))
            .min(t0 + Duration::from_secs(10)),
    );
    let fits = powerlaw::run_powerlaw(inputs, &residual, config, registry, boost_deadline);

    let n = ys.len();
    let val_rows: Vec<usize> = (0..n).filter(|i| i % 5 == 4).collect();
    if val_rows.len() < 10 {
        return;
    }
    let val_inputs: Vec<Vec<f64>> = val_rows.iter().map(|&i| inputs[i].clone()).collect();
    let val_ys: Vec<f64> = val_rows.iter().map(|&i| ys[i]).collect();
    let base_val = expr_error(&best_expr, &val_inputs, &val_ys, registry);
    if !base_val.is_finite() {
        return;
    }

    for fit in fits.into_iter().take(5) {
        let combined = build::binary("Plus", best_expr.clone(), fit.expression, registry);
        let comb_val = expr_error(&combined, &val_inputs, &val_ys, registry);
        if comb_val < 0.9 * base_val {
            let full = expr_error(&combined, inputs, ys, registry);
            if full.is_finite() && full < best_err {
                pool.push((full, combined));
            }
        }
    }
}

#[cfg(test)]
mod v5_tests {
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

    /// P1-2 mechanism: the literal->param->LM->literal round trip must
    /// re-optimize a slightly-off coefficient directly against y.
    #[test]
    fn polish_refits_coefficients() {
        let registry = Arc::new(OperatorRegistry::with_builtins());
        let mut seed = 5u64;
        let mut inputs = Vec::new();
        for _ in 0..300 {
            inputs.push(vec![-3.0 + 6.0 * lcg(&mut seed)]);
        }
        let ys: Vec<f64> = inputs.iter().map(|r| 2.0 * r[0]).collect();

        // Candidate with a deliberately wrong coefficient.
        let off = build::binary("Times", build::num(2.05), build::var(0), &registry);
        let err = expr_error(&off, &inputs, &ys, &registry);
        let mut pool = vec![(err, off)];
        let config = SearchConfig::fable_default();
        polish_pool(&mut pool, &inputs, &ys, &config, &registry);

        let best = pool
            .iter()
            .map(|(e, _)| *e)
            .fold(f64::INFINITY, f64::min);
        assert!(
            best < err * 1e-3,
            "polish failed to refit the coefficient: {} -> {}",
            err,
            best
        );
    }

    /// P1-2 end-to-end: noisy rational target. The y*Q = P linearization is
    /// biased under noise (y appears in the design columns); the raw-space
    /// polish must pull the recovered coefficients back to the truth.
    #[test]
    fn pipeline_recovers_noisy_rational() {
        let mut seed = 31u64;
        let mut inputs = Vec::new();
        for _ in 0..750 {
            inputs.push(vec![-3.0 + 6.0 * lcg(&mut seed)]);
        }
        let clean: Vec<f64> = inputs
            .iter()
            .map(|r| (r[0] + 2.0) / (r[0] * r[0] + 1.0))
            .collect();
        let y_std = std_dev(&clean).max(1e-30);
        let sigma = 0.01 * y_std;
        let ys: Vec<f64> = clean.iter().map(|&c| c + sigma * gauss(&mut seed)).collect();

        let mut config = SearchConfig::fable_default();
        config.time_budget_s = 40.0;
        config.beam_width = 200;
        config.early_exit_threshold = 9e-3;
        let results = run_fable(&inputs, &ys, &config).expect("search");
        assert!(!results.is_empty());

        let best_clean = results
            .iter()
            .map(|r| {
                let mut acc = 0.0;
                for (row, &c) in inputs.iter().zip(&clean) {
                    let p = r.eval_multi(row);
                    if !p.is_finite() {
                        return f64::INFINITY;
                    }
                    let d = p - c;
                    acc += d * d;
                }
                (acc / clean.len() as f64).sqrt()
            })
            .fold(f64::INFINITY, f64::min);
        assert!(
            best_clean < 0.5 * sigma,
            "rational not recovered under noise: clean rmse {} vs sigma {}",
            best_clean,
            sigma
        );
    }
}

#[cfg(test)]
mod v5_boost_tests {
    use super::*;

    fn lcg(seed: &mut u64) -> f64 {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((*seed >> 33) as f64) / (u64::MAX >> 33) as f64
    }

    /// P3-1: with a partially-correct f1 in the pool, one closed-form pass on
    /// the residual recovers the missing additive component.
    #[test]
    fn residual_boost_completes_partial_fit() {
        let registry = Arc::new(OperatorRegistry::with_builtins());
        let mut seed = 3u64;
        let mut inputs = Vec::new();
        for _ in 0..500 {
            let x = -2.0 + 4.0 * lcg(&mut seed);
            let z = -3.0 + 6.0 * lcg(&mut seed);
            inputs.push(vec![x, z]);
        }
        // y = x^2 + 0.05 z: the second term is ~5% of the signal, i.e. the
        // "close but not exact" band residual boosting is built for.
        let ys: Vec<f64> = inputs.iter().map(|r| r[0] * r[0] + 0.05 * r[1]).collect();

        let f1 = build::unary("Square", build::var(0), &registry);
        let f1_err = expr_error(&f1, &inputs, &ys, &registry);
        assert!(f1_err.is_finite() && f1_err > 0.0);

        let config = SearchConfig::fable_default();
        let mut pool = vec![(f1_err, f1)];
        residual_boost(&mut pool, &inputs, &ys, &config, &registry, None);

        let best = pool
            .iter()
            .map(|(e, _)| *e)
            .fold(f64::INFINITY, f64::min);
        assert!(
            best < 1e-3 * f1_err,
            "residual boost failed: {} -> {}",
            f1_err,
            best
        );
    }
}
