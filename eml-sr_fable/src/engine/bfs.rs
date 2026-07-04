use crate::config::SearchConfig;
use crate::core::build;
use crate::core::expression::{Expression, Node};
use crate::core::signature::{fingerprint, Fingerprint};
use crate::core::value::{is_usable, real, Value};
use crate::engine::optimizer;
use crate::error::EmlError;
use crate::ops::registry::OperatorRegistry;
use crate::result::SearchResult;
use dashmap::DashMap;
use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// A Pareto-front entry: an unwrapped structure plus its best affine map.
#[derive(Clone)]
pub(crate) struct FrontEntry {
    pub error: f64,
    pub expr: Expression,
    pub affine: Option<(f64, f64)>,
}

type Front = Arc<Mutex<BTreeMap<usize, FrontEntry>>>;

/// Executes a parallel Breadth-First Search (BFS) for symbolic regression.
///
/// Fable-edition upgrades over the cursor engine:
/// - **Affine scoring**: every candidate `f` is ranked by
///   `min_{a,b} RMSE(y, a*f+b)` (closed form), so scale/offset constants are
///   free and the effective expressivity grows by ~3 complexity units.
/// - **Subsampled scoring**: candidates are scored on a deterministic subset
///   of the data; the surviving Pareto front is re-fit on the full data.
/// - **Beam diversity**: at most `affine_class_cap` members of one affine
///   equivalence class survive per level.
/// - **Early exit**: the search stops once a candidate reaches
///   `early_exit_threshold * std(y)` full-data RMSE.
/// - **Deadlines**: expansion halts once the wall-clock budget is exhausted.
pub fn run_bfs(
    inputs: &[Vec<f64>],
    ys: &[f64],
    config: &SearchConfig,
) -> Result<Vec<SearchResult>, EmlError> {
    run_bfs_with(inputs, ys, config, None)
}

pub fn run_bfs_with(
    inputs: &[Vec<f64>],
    ys: &[f64],
    config: &SearchConfig,
    deadline: Option<Instant>,
) -> Result<Vec<SearchResult>, EmlError> {
    let registry = Arc::new(OperatorRegistry::with_builtins());
    let front = run_bfs_front(inputs, ys, config, deadline, &registry)?;
    finalize_result(&front, &registry, config, ys)
}

/// Same as [`run_bfs_with`] but returns the raw Pareto front (used by the
/// fable pipeline to post-process ratio-search results).
pub(crate) fn run_bfs_front(
    inputs: &[Vec<f64>],
    ys: &[f64],
    config: &SearchConfig,
    deadline: Option<Instant>,
    registry: &Arc<OperatorRegistry>,
) -> Result<Vec<(f64, Expression)>, EmlError> {
    if inputs.is_empty() || ys.is_empty() || inputs.len() != ys.len() {
        return Err(EmlError::invalid(
            "Inputs and target vector must be non-empty and equal length.",
        ));
    }

    let num_vars = inputs[0].len();

    let data_inputs: Vec<Vec<Value>> = inputs
        .iter()
        .map(|row| row.iter().map(|&v| real(v)).collect())
        .collect();
    let targets: Vec<f64> = ys.to_vec();

    // Deterministic evenly-spaced subsample used for scoring during search.
    let n = targets.len();
    let sub_n = if config.subsample_size == 0 {
        n
    } else {
        n.min(config.subsample_size)
    };
    let sub_idx: Vec<usize> = (0..sub_n).map(|i| i * n / sub_n).collect();
    let sub_inputs: Vec<Vec<Value>> = sub_idx.iter().map(|&i| data_inputs[i].clone()).collect();
    let sub_targets: Vec<f64> = sub_idx.iter().map(|&i| targets[i]).collect();

    let y_std = std_dev(&targets).max(1e-30);
    let exit_error = config.early_exit_threshold * y_std;

    let seen: Arc<DashMap<Fingerprint, ()>> = Arc::new(DashMap::new());
    let front: Front = Arc::new(Mutex::new(BTreeMap::new()));

    let mut levels: Vec<Vec<Expression>> = vec![Vec::new(); config.max_complexity + 1];

    for &init in &config.param_seed_inits {
        let expr = Expression::parameter_with(init);
        if let Some(fp) = fingerprint(&expr, registry) {
            if seen.insert(fp, ()).is_none() {
                score_into_front(&expr, &sub_inputs, &sub_targets, config, &front, registry);
                levels[1].push(expr);
            }
        }
    }

    for i in 0..num_vars {
        let expr = Expression::variable(i as u8, format!("v_{{{i}}}"));
        if let Some(fp) = fingerprint(&expr, registry) {
            if seen.insert(fp, ()).is_none() {
                score_into_front(&expr, &sub_inputs, &sub_targets, config, &front, registry);
                levels[1].push(expr);
            }
        }
    }

    if config.verbose {
        println!(
            "[EML-SR-Fable] Search started: {} level-1 seeds, subsample {}/{} points.",
            levels[1].len(),
            sub_n,
            n
        );
    }

    'levels: for k in 2..=config.max_complexity {
        if deadline_hit(deadline) {
            break 'levels;
        }
        let t_level = Instant::now();
        let unary_ids = registry.ids_by_arity(1);
        let reg = Arc::clone(registry);
        let seen_ref = Arc::clone(&seen);

        let new_unary: Vec<Expression> = levels[k - 1]
            .par_iter()
            .flat_map(|child| {
                let mut local = Vec::new();
                for &uid in &unary_ids {
                    let mut nodes = child.nodes.clone();
                    nodes.push(Node::Op {
                        op_id: uid,
                        arity: 1,
                    });
                    let expr = Expression::new(
                        nodes,
                        child.var_count(),
                        child.param_count(),
                        format!("{}({})", reg.meta(uid).name, child.display()),
                    );
                    if let Some(fp) = fingerprint(&expr, &reg) {
                        if seen_ref.insert(fp, ()).is_none() {
                            local.push(expr);
                        }
                    }
                }
                local
            })
            .collect();

        let binary_ids = registry.ids_by_arity(2);
        let mut binary_candidates = Vec::new();
        for lk in 1..k - 1 {
            if deadline_hit(deadline) {
                break;
            }
            let rk = k - 1 - lk;
            let reg_bin = Arc::clone(registry);
            let seen_bin = Arc::clone(&seen);

            let new_binary: Vec<Expression> = levels[lk]
                .par_iter()
                .flat_map(|left| {
                    let mut local = Vec::new();
                    for right in &levels[rk] {
                        for &bid in &binary_ids {
                            let meta = reg_bin.meta(bid);
                            if meta.name == "Pow" && !right.is_scalar_only() {
                                continue;
                            }
                            if meta.is_commutative && lk == rk && left.display() > right.display() {
                                continue;
                            }
                            let mut nodes = left.nodes.clone();
                            let mut right_nodes = right.nodes.clone();
                            for node in &mut right_nodes {
                                if let Node::Param { id, .. } = node {
                                    *id += left.param_count();
                                }
                            }
                            nodes.extend_from_slice(&right_nodes);
                            nodes.push(Node::Op {
                                op_id: bid,
                                arity: 2,
                            });

                            let expr = Expression::new(
                                nodes,
                                left.var_count().max(right.var_count()),
                                left.param_count() + right.param_count(),
                                format!("{}({}, {})", meta.name, left.display(), right.display()),
                            );
                            if let Some(fp) = fingerprint(&expr, &reg_bin) {
                                if seen_bin.insert(fp, ()).is_none() {
                                    local.push(expr);
                                }
                            }
                        }
                    }
                    local
                })
                .collect();
            binary_candidates.extend(new_binary);
        }

        let mut all_candidates = new_unary;
        all_candidates.extend(binary_candidates);
        if all_candidates.len() > config.max_pool_size {
            all_candidates.truncate(config.max_pool_size);
        }

        // Score every admitted candidate on the subsample.
        let scored: Vec<(Expression, Option<(f64, f64)>, f64)> = all_candidates
            .into_par_iter()
            .filter_map(|mut expr| {
                let mut preds = predict(&expr, &sub_inputs, registry)?;

                if config.optimize_constants && expr.param_count() > 0 {
                    let raw = rmse(&preds, &sub_targets);
                    let (refined_expr, refined_error) = optimizer::refine_constants(
                        &expr,
                        &sub_inputs,
                        &sub_targets,
                        registry,
                        config.optimizer_max_iters,
                    );
                    if refined_error < raw {
                        if let Some(p) = predict(&refined_expr, &sub_inputs, registry) {
                            expr = refined_expr;
                            preds = p;
                        }
                    }
                }

                let raw_error = rmse(&preds, &sub_targets);
                let (affine, aff_error) = if config.affine_scaling {
                    affine_fit(&preds, &sub_targets)
                } else {
                    (None, f64::INFINITY)
                };
                let best_error = raw_error.min(aff_error);
                if !best_error.is_finite() {
                    return None;
                }
                let affine = if aff_error < raw_error * (1.0 - 1e-12) {
                    affine
                } else {
                    None
                };
                Some((expr, affine, best_error))
            })
            .collect();

        for (expr, affine, error) in &scored {
            update_pareto(&front, expr, *error, *affine);
        }

        // Beam selection with affine-class diversity.
        let mut ranked: Vec<(Expression, f64)> = scored
            .into_iter()
            .map(|(expr, _aff, err)| {
                let score = err * (1.0 + expr.complexity() as f64 * config.complexity_penalty);
                (expr, score)
            })
            .collect();
        ranked.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let mut selected: Vec<Expression> = Vec::with_capacity(config.beam_width.min(ranked.len()));
        if config.affine_class_cap > 0 && config.affine_scaling {
            let mut class_counts: HashMap<Vec<i64>, usize> = HashMap::new();
            for (expr, _) in ranked {
                if selected.len() >= config.beam_width {
                    break;
                }
                let key = match predict(&expr, &sub_inputs, registry) {
                    Some(p) => affine_class_key(&p),
                    None => continue,
                };
                let count = class_counts.entry(key).or_insert(0);
                if *count >= config.affine_class_cap {
                    continue;
                }
                *count += 1;
                selected.push(expr);
            }
        } else {
            selected = ranked
                .into_iter()
                .take(config.beam_width)
                .map(|(e, _)| e)
                .collect();
        }
        levels[k] = selected;

        if config.verbose {
            println!(
                "[EML-SR-Fable]   -> Level {} in {:?}: beam holds {} candidates.",
                k,
                t_level.elapsed(),
                levels[k].len()
            );
        }

        // Early exit on verified full-data accuracy.
        if best_full_error(&front, &data_inputs, &targets, registry) <= exit_error {
            if config.verbose {
                println!("[EML-SR-Fable]   -> Early exit: target precision reached.");
            }
            break 'levels;
        }
    }

    refine_front(
        &front,
        &data_inputs,
        &targets,
        registry,
        config,
    );

    let guard = front.lock().unwrap();
    if guard.is_empty() {
        return Err(EmlError::NotFound {
            max_complexity: config.max_complexity,
        });
    }
    let y_scale = std_dev(&targets).max(1e-30);
    let entries: Vec<(f64, Expression)> = guard
        .values()
        .filter(|e| e.error.is_finite())
        .map(|e| {
            let (a, b) = e.affine.unwrap_or((1.0, 0.0));
            let wrapped = build::affine_wrap(e.expr.clone(), a, b, y_scale, registry);
            (e.error, wrapped)
        })
        .collect();
    Ok(entries)
}

fn deadline_hit(deadline: Option<Instant>) -> bool {
    deadline.map_or(false, |d| Instant::now() >= d)
}

fn std_dev(v: &[f64]) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    (v.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / v.len() as f64).sqrt()
}

fn rmse(pred: &[f64], target: &[f64]) -> f64 {
    let mut acc = 0.0;
    for (p, t) in pred.iter().zip(target) {
        let d = p - t;
        acc += d * d;
    }
    if acc.is_finite() {
        (acc / target.len() as f64).sqrt()
    } else {
        f64::INFINITY
    }
}

/// Evaluates an expression on the given rows; None when any point fails.
fn predict(
    expr: &Expression,
    inputs: &[Vec<Value>],
    reg: &OperatorRegistry,
) -> Option<Vec<f64>> {
    let mut preds = Vec::with_capacity(inputs.len());
    for row in inputs {
        let v = expr.eval(row, reg)?;
        if !is_usable(v) || v.im.abs() > 1e-6 * v.re.abs().max(1.0) {
            return None;
        }
        preds.push(v.re);
    }
    Some(preds)
}

/// Closed-form least-squares fit of `y ≈ a*p + b`. Returns ((a, b), rmse).
fn affine_fit(preds: &[f64], targets: &[f64]) -> (Option<(f64, f64)>, f64) {
    let n = preds.len() as f64;
    if n < 2.0 {
        return (None, f64::INFINITY);
    }
    let mean_p = preds.iter().sum::<f64>() / n;
    let mean_y = targets.iter().sum::<f64>() / n;
    let mut cov = 0.0;
    let mut var = 0.0;
    for (p, y) in preds.iter().zip(targets) {
        cov += (p - mean_p) * (y - mean_y);
        var += (p - mean_p) * (p - mean_p);
    }
    let p_scale = preds.iter().fold(0.0f64, |m, v| m.max(v.abs())).max(1.0);
    if var <= 1e-24 * p_scale * p_scale || !var.is_finite() || !cov.is_finite() {
        return (None, f64::INFINITY);
    }
    let a = cov / var;
    let b = mean_y - a * mean_p;
    if !a.is_finite() || !b.is_finite() {
        return (None, f64::INFINITY);
    }
    let mut acc = 0.0;
    for (p, y) in preds.iter().zip(targets) {
        let d = a * p + b - y;
        acc += d * d;
    }
    ((Some((a, b))), (acc / n).sqrt())
}

/// Canonical key of the affine equivalence class of a prediction vector.
fn affine_class_key(preds: &[f64]) -> Vec<i64> {
    let n = preds.len() as f64;
    let mean = preds.iter().sum::<f64>() / n;
    let norm = preds
        .iter()
        .map(|p| (p - mean) * (p - mean))
        .sum::<f64>()
        .sqrt();
    let scale = preds.iter().fold(0.0f64, |m, v| m.max(v.abs())).max(1.0);
    if norm <= 1e-12 * scale {
        return Vec::new(); // all constants share one class
    }
    let mut flip = 0.0;
    let mut key = Vec::with_capacity(preds.len());
    for p in preds {
        let mut w = (p - mean) / norm;
        if flip == 0.0 && w.abs() > 1e-9 {
            flip = if w < 0.0 { -1.0 } else { 1.0 };
        }
        if flip != 0.0 {
            w *= flip;
        }
        key.push((w * 1e8).round() as i64);
    }
    key
}

fn update_pareto(front: &Front, expr: &Expression, error: f64, affine: Option<(f64, f64)>) {
    let mut guard = front.lock().unwrap();
    let comp = expr.complexity();
    let should_insert = match guard.get(&comp) {
        None => true,
        Some(prev) => error < prev.error,
    };
    if should_insert {
        guard.insert(
            comp,
            FrontEntry {
                error,
                expr: expr.clone(),
                affine,
            },
        );
    }
}

fn score_into_front(
    expr: &Expression,
    inputs: &[Vec<Value>],
    targets: &[f64],
    config: &SearchConfig,
    front: &Front,
    reg: &OperatorRegistry,
) {
    if let Some(preds) = predict(expr, inputs, reg) {
        let raw = rmse(&preds, targets);
        let (affine, aff_err) = if config.affine_scaling {
            affine_fit(&preds, targets)
        } else {
            (None, f64::INFINITY)
        };
        let best = raw.min(aff_err);
        if best.is_finite() {
            let affine = if aff_err < raw * (1.0 - 1e-12) {
                affine
            } else {
                None
            };
            update_pareto(front, expr, best, affine);
        }
    }
}

/// Full-data error of the best front entry (affine re-fit on full data).
fn best_full_error(
    front: &Front,
    inputs: &[Vec<Value>],
    targets: &[f64],
    reg: &OperatorRegistry,
) -> f64 {
    let best = {
        let guard = front.lock().unwrap();
        guard
            .values()
            .min_by(|a, b| a.error.partial_cmp(&b.error).unwrap())
            .map(|e| e.expr.clone())
    };
    match best {
        Some(expr) => match predict(&expr, inputs, reg) {
            Some(preds) => {
                let raw = rmse(&preds, targets);
                let (_, aff) = affine_fit(&preds, targets);
                raw.min(aff)
            }
            None => f64::INFINITY,
        },
        None => f64::INFINITY,
    }
}

/// Re-fits every Pareto entry on the full data: LM for the top-k
/// param-bearing structures, affine refit and constant snapping for all.
fn refine_front(
    front: &Front,
    inputs: &[Vec<Value>],
    targets: &[f64],
    registry: &Arc<OperatorRegistry>,
    config: &SearchConfig,
) {
    let entries: Vec<(usize, FrontEntry)> = {
        let guard = front.lock().unwrap();
        guard.iter().map(|(k, v)| (*k, v.clone())).collect()
    };

    // Rank by search-time error to decide who deserves the expensive LM pass.
    let mut order: Vec<usize> = (0..entries.len()).collect();
    order.sort_by(|&a, &b| entries[a].1.error.partial_cmp(&entries[b].1.error).unwrap());
    let lm_set: Vec<usize> = order.into_iter().take(config.refinement_top_k).collect();

    let refined: Vec<(usize, FrontEntry)> = entries
        .par_iter()
        .enumerate()
        .map(|(idx, (comp, entry))| {
            let mut expr = entry.expr.clone();

            if expr.param_count() > 0 && config.optimize_constants && lm_set.contains(&idx) {
                let (better, err) = optimizer::refine_constants_multi(
                    &expr,
                    inputs,
                    targets,
                    registry,
                    config.refine_max_iters,
                    3,
                );
                if err.is_finite() {
                    expr = better;
                }
                if config.snap_constants {
                    expr = optimizer::snap_constants(&expr, inputs, targets, registry);
                }
            }

            let (error, affine) = match predict(&expr, inputs, registry) {
                Some(preds) => {
                    let raw = rmse(&preds, targets);
                    let (aff, aff_err) = if config.affine_scaling {
                        affine_fit(&preds, targets)
                    } else {
                        (None, f64::INFINITY)
                    };
                    if aff_err < raw * (1.0 - 1e-12) {
                        (aff_err, aff)
                    } else {
                        (raw, None)
                    }
                }
                None => (f64::INFINITY, None),
            };

            (
                *comp,
                FrontEntry {
                    error,
                    expr,
                    affine,
                },
            )
        })
        .collect();

    let mut guard = front.lock().unwrap();
    guard.clear();
    for (comp, entry) in refined {
        if !entry.error.is_finite() {
            continue;
        }
        let should_insert = match guard.get(&comp) {
            None => true,
            Some(prev) => entry.error < prev.error,
        };
        if should_insert {
            guard.insert(comp, entry);
        }
    }
}

fn finalize_result(
    front: &[(f64, Expression)],
    reg: &Arc<OperatorRegistry>,
    config: &SearchConfig,
    _ys: &[f64],
) -> Result<Vec<SearchResult>, EmlError> {
    if front.is_empty() {
        return Err(EmlError::NotFound {
            max_complexity: config.max_complexity,
        });
    }
    Ok(merge_pareto(front.to_vec(), reg))
}

/// Merges (error, expression) pairs into a strict Pareto front ordered
/// best-error-first (index 0 = lowest error).
pub(crate) fn merge_pareto(
    entries: Vec<(f64, Expression)>,
    reg: &Arc<OperatorRegistry>,
) -> Vec<SearchResult> {
    let mut by_complexity: BTreeMap<usize, (f64, Expression)> = BTreeMap::new();
    for (error, expr) in entries {
        if !error.is_finite() {
            continue;
        }
        let comp = expr.complexity();
        let insert = match by_complexity.get(&comp) {
            None => true,
            Some((prev, _)) => error < *prev,
        };
        if insert {
            by_complexity.insert(comp, (error, expr));
        }
    }

    let mut results = Vec::new();
    let mut best = f64::INFINITY;
    for (_comp, (error, expr)) in by_complexity {
        if error < best {
            results.push(SearchResult::new(expr, error, Arc::clone(reg)));
            best = error;
        }
    }
    results.reverse();
    results
}
