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
        return Ok(bfs::merge_pareto(pool, &registry));
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

    Ok(bfs::merge_pareto(pool, &registry))
}
