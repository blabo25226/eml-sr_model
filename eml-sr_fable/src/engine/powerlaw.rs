//! Stage A: closed-form power-law (monomial-sum) solver.
//!
//! A large fraction of physical laws are sums of a few monomials
//! `c * prod_i x_i^{a_i}` — possibly after a simple invertible transform of
//! the target (log y, 1/y, 1/y^2, y^2). This module recovers such structures
//! in closed form via least squares in log space (greedy residual boosting)
//! and orthogonal matching pursuit over a dictionary of standard-exponent
//! monomials. It costs milliseconds-to-seconds, so it runs before the beam
//! search and short-circuits it entirely when the data is monomial-shaped.

use crate::config::SearchConfig;
use crate::core::build::{self, Feature, Monomial, Term};
use crate::core::expression::Expression;
use crate::ops::registry::OperatorRegistry;
use rayon::prelude::*;
use std::time::Instant;

/// A candidate produced by the power-law stage, with its full-data RMSE.
pub struct PowerlawFit {
    pub expression: Expression,
    pub error: f64,
}

/// Invertible target transforms attempted by Stage A.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Transform {
    Id,
    Log,
    InvY,
    InvY2,
    Y2,
    /// t = ln(1 + y), defined for y > -1. Linearizes `exp(m) - 1` shapes
    /// (diode law, Bose-Einstein denominators after whitening).
    Log1p,
}

const TRANSFORMS: &[Transform] = &[
    Transform::Id,
    Transform::Log,
    Transform::InvY,
    Transform::InvY2,
    Transform::Y2,
    Transform::Log1p,
];

/// Solves a dense linear least-squares system via normal equations.
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
    for i in 0..m {
        for j in 0..i {
            ata[i][j] = ata[j][i];
        }
        // Tiny Tikhonov ridge keeps near-collinear systems stable.
        ata[i][i] += 1e-12;
    }
    gaussian_solve(&mut ata, &mut atb)
}

fn gaussian_solve(a: &mut [Vec<f64>], b: &mut [f64]) -> Option<Vec<f64>> {
    let n = b.len();
    for col in 0..n {
        let mut pivot = col;
        for row in (col + 1)..n {
            if a[row][col].abs() > a[pivot][col].abs() {
                pivot = row;
            }
        }
        if a[pivot][col].abs() < 1e-300 {
            return None;
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
            if factor == 0.0 {
                continue;
            }
            for j in col..n {
                a[row][j] -= factor * a[col][j];
            }
            b[row] -= factor * b[col];
        }
    }
    Some(b.to_vec())
}

fn rms(v: &[f64]) -> f64 {
    if v.is_empty() {
        return f64::INFINITY;
    }
    (v.iter().map(|x| x * x).sum::<f64>() / v.len() as f64).sqrt()
}

fn rmse(pred: &[f64], target: &[f64]) -> f64 {
    if pred.len() != target.len() || pred.is_empty() {
        return f64::INFINITY;
    }
    let mut acc = 0.0;
    for (p, t) in pred.iter().zip(target) {
        let d = p - t;
        if !d.is_finite() {
            return f64::INFINITY;
        }
        acc += d * d;
    }
    (acc / target.len() as f64).sqrt()
}

/// Variables usable inside fractional-power monomials (strictly positive data).
fn usable_variables(inputs: &[Vec<f64>]) -> Vec<usize> {
    if inputs.is_empty() {
        return Vec::new();
    }
    let d = inputs[0].len();
    (0..d)
        .filter(|&j| inputs.iter().all(|row| row[j] > 1e-12 && row[j].is_finite()))
        .collect()
}

/// Fits `r ≈ c * prod x^a` in log space. Returns (raw exponents, coeff).
fn fit_monomial_log(
    inputs: &[Vec<f64>],
    residual: &[f64],
    usable: &[usize],
) -> Option<(Vec<f64>, f64)> {
    let scale = residual.iter().fold(0.0f64, |m, v| m.max(v.abs()));
    if !(scale.is_finite()) || scale <= 0.0 {
        return None;
    }
    let mask: Vec<usize> = (0..residual.len())
        .filter(|&i| residual[i].abs() > 1e-13 * scale)
        .collect();
    let d = inputs[0].len();
    if mask.len() < usable.len() + 2 {
        return None;
    }
    let sign = residual[mask[0]].signum();
    if mask.iter().any(|&i| residual[i].signum() != sign) {
        return None;
    }

    let design: Vec<Vec<f64>> = mask
        .iter()
        .map(|&i| {
            let mut row = Vec::with_capacity(usable.len() + 1);
            row.push(1.0);
            for &j in usable {
                row.push(inputs[i][j].ln());
            }
            row
        })
        .collect();
    let target: Vec<f64> = mask.iter().map(|&i| residual[i].abs().ln()).collect();
    let sol = lstsq(&design, &target)?;

    let mut exps = vec![0.0; d];
    for (k, &j) in usable.iter().enumerate() {
        exps[j] = sol[k + 1];
    }
    let coeff = sign * sol[0].exp();
    if !coeff.is_finite() {
        return None;
    }
    Some((exps, coeff))
}

/// Generates rounded exponent-vector candidates (rationals with small denominators).
fn exponent_candidates(raw: &[f64]) -> Vec<Vec<f64>> {
    let mut cands: Vec<Vec<f64>> = Vec::new();
    for den in [1.0f64, 2.0, 3.0, 4.0] {
        let rounded: Vec<f64> = raw
            .iter()
            .map(|&e| {
                let r = (e * den).round() / den;
                if r.abs() < 1e-9 {
                    0.0
                } else {
                    r
                }
            })
            .collect();
        if !cands.contains(&rounded) {
            cands.push(rounded);
        }
    }
    cands.push(raw.to_vec());
    cands
}

/// Least-squares coefficient for a single basis column against a residual.
fn refit_coeff(basis: &[f64], residual: &[f64]) -> Option<f64> {
    let mut num = 0.0;
    let mut den = 0.0;
    for (b, r) in basis.iter().zip(residual) {
        if !b.is_finite() {
            return None;
        }
        num += b * r;
        den += b * b;
    }
    if den <= 0.0 || !den.is_finite() {
        return None;
    }
    Some(num / den)
}

/// Jointly refits the coefficients of `terms` against `target` (coeffs folded in).
fn joint_refit(terms: &mut [Term], columns: &[Vec<f64>], target: &[f64]) -> Option<Vec<f64>> {
    let n = target.len();
    let design: Vec<Vec<f64>> = (0..n)
        .map(|i| columns.iter().map(|c| c[i]).collect())
        .collect();
    let coeffs = lstsq(&design, target)?;
    if coeffs.iter().any(|c| !c.is_finite()) {
        return None;
    }
    for (term, &c) in terms.iter_mut().zip(&coeffs) {
        term.coeff = c;
    }
    let pred: Vec<f64> = (0..n)
        .map(|i| {
            columns
                .iter()
                .zip(&coeffs)
                .map(|(col, c)| col[i] * c)
                .sum::<f64>()
        })
        .collect();
    Some(pred)
}

/// Greedy residual boosting with log-space monomial fits.
fn greedy_monomial_fit(
    inputs: &[Vec<f64>],
    target: &[f64],
    usable: &[usize],
    max_terms: usize,
) -> Option<Vec<Term>> {
    let mut terms: Vec<Term> = Vec::new();
    let mut unit_columns: Vec<Vec<f64>> = Vec::new();
    let mut residual = target.to_vec();
    let mut current_rmse = rms(&residual);

    for _ in 0..max_terms {
        let (raw_exps, _) = match fit_monomial_log(inputs, &residual, usable) {
            Some(f) => f,
            None => break,
        };

        // Pick the exponent rounding whose refitted single term reduces the
        // residual the most.
        let mut best: Option<(Term, Vec<f64>, f64)> = None;
        for exps in exponent_candidates(&raw_exps) {
            let unit = Monomial {
                coeff: 1.0,
                exponents: exps.clone(),
            };
            let column = unit.eval_rows(inputs);
            if column.iter().any(|v| !v.is_finite()) {
                continue;
            }
            let coeff = match refit_coeff(&column, &residual) {
                Some(c) => c,
                None => continue,
            };
            let new_res: Vec<f64> = residual
                .iter()
                .zip(&column)
                .map(|(r, c)| r - coeff * c)
                .collect();
            let err = rms(&new_res);
            if best.as_ref().map_or(true, |(_, _, e)| err < *e) {
                best = Some((
                    Term {
                        coeff,
                        exponents: exps,
                        feature: Feature::None,
                    },
                    column,
                    err,
                ));
            }
        }
        let (term, column, err) = best?;

        // Require a meaningful reduction to keep adding terms.
        if err > 0.85 * current_rmse {
            break;
        }
        terms.push(term);
        unit_columns.push(column);

        // Joint refit of all coefficients keeps the greedy path numerically exact.
        match joint_refit(&mut terms, &unit_columns, target) {
            Some(pred) => {
                residual = target.iter().zip(&pred).map(|(t, p)| t - p).collect();
                current_rmse = rms(&residual);
            }
            None => break,
        }
        let scale = rms(target).max(1e-300);
        if current_rmse < 1e-13 * scale {
            break;
        }
    }

    if terms.is_empty() {
        None
    } else {
        Some(terms)
    }
}

/// Enumerates monomial exponent vectors over `usable` variables, with a
/// per-term active-variable cap chosen so the dictionary stays small.
fn enumerate_exponents(
    d: usize,
    usable: &[usize],
    exponent_set: &[f64],
    cap: usize,
) -> Vec<Vec<f64>> {
    let mut vectors: Vec<Vec<f64>> = vec![vec![0.0; d]]; // constant
    let mut stack: Vec<(usize, Vec<(usize, f64)>)> = vec![(0, Vec::new())];
    while let Some((start, active)) = stack.pop() {
        if !active.is_empty() {
            let mut exps = vec![0.0; d];
            for &(j, e) in &active {
                exps[j] = e;
            }
            vectors.push(exps);
        }
        if active.len() >= cap {
            continue;
        }
        for (uidx, &j) in usable.iter().enumerate().skip(start) {
            for &e in exponent_set {
                let mut next = active.clone();
                next.push((j, e));
                stack.push((uidx + 1, next));
            }
        }
    }
    vectors
}

fn dict_size_estimate(n_usable: usize, n_exps: usize, cap: usize) -> usize {
    let mut total = 1usize;
    let mut choose = 1usize;
    for k in 1..=cap.min(n_usable) {
        choose = choose * (n_usable - k + 1) / k;
        total = total.saturating_add(choose.saturating_mul(n_exps.pow(k as u32)));
    }
    total
}

fn build_dictionary(
    d: usize,
    usable: &[usize],
    exponent_set: &[f64],
    max_active: usize,
    size_limit: usize,
) -> Vec<Term> {
    let mut cap = usable.len().min(max_active);
    while cap > 1 && dict_size_estimate(usable.len(), exponent_set.len(), cap) > size_limit {
        cap -= 1;
    }
    enumerate_exponents(d, usable, exponent_set, cap)
        .into_iter()
        .map(|exponents| Term {
            coeff: 1.0,
            exponents,
            feature: Feature::None,
        })
        .collect()
}

/// Builds the feature-augmented dictionary: `monomial × feature` where the
/// feature is a single-variable trig/log factor or a pairwise
/// difference/product factor, and the monomial avoids the feature's own
/// variables. The exponent set and active-variable cap of the monomial part
/// are chosen adaptively to keep the dictionary under `size_limit` columns.
fn build_feature_dictionary(d: usize, usable: &[usize], size_limit: usize) -> Vec<Term> {
    let mut features: Vec<Feature> = Vec::new();
    for &i in usable {
        features.push(Feature::Sin(i));
        features.push(Feature::Cos(i));
        features.push(Feature::Sin2x(i));
        features.push(Feature::Cos2x(i));
        features.push(Feature::Ln(i));
    }
    for (a, &i) in usable.iter().enumerate() {
        for &j in usable.iter().skip(a + 1) {
            features.push(Feature::DiffSq(i, j));
            features.push(Feature::CosDiff(i, j));
            features.push(Feature::CosProd(i, j));
            features.push(Feature::SinProd(i, j));
        }
    }
    if features.is_empty() {
        return Vec::new();
    }

    // Pick the richest monomial configuration that fits the budget.
    const CONFIGS: &[(&[f64], usize)] = &[
        (&[-3.0, -2.0, -1.0, 1.0, 2.0, 3.0], 3),
        (&[-2.0, -1.0, 1.0, 2.0], 3),
        (&[-2.0, -1.0, 1.0, 2.0], 2),
        (&[-1.0, 1.0], 3),
        (&[-1.0, 1.0], 2),
        (&[-1.0, 1.0], 1),
    ];
    let per_feature_budget = size_limit / features.len();
    let (exps, cap) = CONFIGS
        .iter()
        .find(|(e, c)| dict_size_estimate(usable.len(), e.len(), *c) <= per_feature_budget)
        .copied()
        .unwrap_or((&[-1.0, 1.0], 1));

    let base = enumerate_exponents(d, usable, exps, cap.min(usable.len()));
    let mut dictionary: Vec<Term> = Vec::with_capacity(base.len() * (features.len() + 1));
    // Plain monomial columns must coexist with the augmented ones — mixed
    // targets like q*Ef + q*B*v*sin(theta) need both kinds of terms.
    for exponents in &base {
        dictionary.push(Term {
            coeff: 1.0,
            exponents: exponents.clone(),
            feature: Feature::None,
        });
    }
    for feature in &features {
        let fvars = feature.vars();
        for exponents in &base {
            if fvars.iter().any(|&v| exponents[v] != 0.0) {
                continue; // keep the feature variable out of the monomial part
            }
            dictionary.push(Term {
                coeff: 1.0,
                exponents: exponents.clone(),
                feature: feature.clone(),
            });
        }
    }
    dictionary
}

/// Orthogonal-least-squares pursuit over dictionaries of standard-exponent
/// monomials. Basic pass: a fractional-rich dictionary (covers sqrt/cube
/// laws) plus an integer-only one — fractional "compromise" columns like
/// x^0.5*y^0.5 otherwise trap the greedy selection when the target is a sum
/// of several same-magnitude integer terms (e.g. x1*y1 + x2*y2 + x3*y3).
/// Feature pass: the `monomial × feature` dictionary for targets the basic
/// dictionaries cannot represent (trig/log/difference factors).
fn omp_monomial_fit(
    inputs: &[Vec<f64>],
    target: &[f64],
    usable: &[usize],
    max_terms: usize,
    deadline: Option<Instant>,
    with_features: bool,
) -> Vec<Vec<Term>> {
    const FRACTIONAL: &[f64] = &[-3.0, -2.0, -1.0, -0.5, 0.5, 1.0, 2.0, 3.0];
    const INTEGER: &[f64] = &[-2.0, -1.0, 1.0, 2.0];
    let d = match inputs.first() {
        Some(row) => row.len(),
        None => return Vec::new(),
    };
    if usable.is_empty() {
        return Vec::new();
    }

    let mut results: Vec<Vec<Term>> = Vec::new();
    if with_features {
        let dictionary = build_feature_dictionary(d, usable, 150_000);
        if !dictionary.is_empty() {
            results.extend(pursue_dictionary(
                inputs,
                target,
                &dictionary,
                max_terms,
                deadline,
            ));
        }
    } else {
        for (exps, max_active) in [(FRACTIONAL, 4usize), (INTEGER, 5usize)] {
            let mut dictionary = build_dictionary(d, usable, exps, max_active, 70_000);
            if exps == INTEGER {
                // Bare single-variable features ride along with the integer
                // dictionary: mixed supports like ln(n0) - m*g*x/(kb*T)
                // need a deep monomial AND a lone feature in one pursuit.
                for &i in usable {
                    for feature in [
                        Feature::Sin(i),
                        Feature::Cos(i),
                        Feature::Sin2x(i),
                        Feature::Cos2x(i),
                        Feature::Ln(i),
                    ] {
                        dictionary.push(Term {
                            coeff: 1.0,
                            exponents: vec![0.0; d],
                            feature,
                        });
                    }
                }
            }
            results.extend(pursue_dictionary(
                inputs,
                target,
                &dictionary,
                max_terms,
                deadline,
            ));
        }
    }
    results
}

/// Runs constant-seeded and unseeded OLS pursuits over one dictionary,
/// returning every finalized candidate.
fn pursue_dictionary(
    inputs: &[Vec<f64>],
    target: &[f64],
    dictionary: &[Term],
    max_terms: usize,
    deadline: Option<Instant>,
) -> Vec<Vec<Term>> {
    // Deterministic subsample for the pursuit itself.
    let n = target.len();
    let sub_n = n.min(200);
    let sub_idx: Vec<usize> = (0..sub_n).map(|i| i * n / sub_n).collect();

    // Precompute normalized dictionary columns on the subsample.
    let columns: Vec<Option<(Vec<f64>, f64)>> = dictionary
        .par_iter()
        .map(|term| {
            let mut col = Vec::with_capacity(sub_idx.len());
            for &i in &sub_idx {
                let row = &inputs[i];
                let mut acc = 1.0f64;
                for (x, &e) in row.iter().zip(&term.exponents) {
                    if e != 0.0 {
                        acc *= x.powf(e);
                    }
                }
                acc *= term.feature.eval_row(row);
                if !acc.is_finite() {
                    return None;
                }
                col.push(acc);
            }
            let norm = col.iter().map(|v| v * v).sum::<f64>().sqrt();
            if norm < 1e-300 || !norm.is_finite() {
                None
            } else {
                Some((col, norm))
            }
        })
        .collect();

    let sub_target: Vec<f64> = sub_idx.iter().map(|&i| target[i]).collect();
    let scale = rms(&sub_target).max(1e-300);

    // Orthogonalizes `col` against `basis`; returns (q, q_norm).
    let orthogonalize = |col: &[f64], basis: &[Vec<f64>]| -> (Vec<f64>, f64) {
        let mut q = col.to_vec();
        for b in basis {
            let proj: f64 = col.iter().zip(b).map(|(a, x)| a * x).sum();
            for (qi, bi) in q.iter_mut().zip(b) {
                *qi -= proj * bi;
            }
        }
        let nq = q.iter().map(|v| v * v).sum::<f64>().sqrt();
        (q, nq)
    };

    // Orthogonal Least Squares greedy selection: at each step pick the
    // dictionary column whose component orthogonal to the already-chosen
    // basis best explains the current residual. This is robust against the
    // highly collinear monomial dictionary (plain OMP gets stuck on
    // "compromise" columns like x^1 * y^0.5 for the target x^2 + 3y).
    // With `seed_constant`, the constant column is pre-selected: for a
    // near-constant target the first free pick otherwise locks onto an
    // arbitrary smooth monotone column.
    let select = |seed_constant: bool| -> Vec<usize> {
        let mut ortho_basis: Vec<Vec<f64>> = Vec::new();
        let mut residual = sub_target.clone();
        let mut chosen: Vec<usize> = Vec::new();
        let mut current_rmse = rms(&residual);

        if seed_constant {
            if let Some((col, norm)) = columns[0].as_ref() {
                let qn: Vec<f64> = col.iter().map(|v| v / norm).collect();
                let r_dot: f64 = residual.iter().zip(&qn).map(|(a, b)| a * b).sum();
                for (ri, bi) in residual.iter_mut().zip(&qn) {
                    *ri -= r_dot * bi;
                }
                ortho_basis.push(qn);
                chosen.push(0);
                current_rmse = rms(&residual);
            }
        }

        for _ in 0..max_terms {
            if deadline.map_or(false, |dl| Instant::now() >= dl) {
                break;
            }
            let best = columns
                .par_iter()
                .enumerate()
                .filter_map(|(idx, c)| {
                    if chosen.contains(&idx) {
                        return None;
                    }
                    let (col, norm) = c.as_ref()?;
                    let (q, q_norm) = orthogonalize(col, &ortho_basis);
                    if q_norm < 1e-8 * norm || !q_norm.is_finite() {
                        return None; // collinear with the chosen set
                    }
                    let dot: f64 = q.iter().zip(&residual).map(|(a, b)| a * b).sum();
                    let score = (dot / q_norm).abs();
                    if score.is_finite() {
                        Some((idx, score, q, q_norm))
                    } else {
                        None
                    }
                })
                .reduce_with(|a, b| if a.1 >= b.1 { a } else { b });
            let (best_idx, best_score, q, q_norm) = match best {
                Some(v) => v,
                None => break,
            };
            if best_score < 1e-12 * scale {
                break;
            }

            let qn: Vec<f64> = q.iter().map(|v| v / q_norm).collect();
            let r_dot: f64 = residual.iter().zip(&qn).map(|(a, b)| a * b).sum();
            for (ri, bi) in residual.iter_mut().zip(&qn) {
                *ri -= r_dot * bi;
            }
            ortho_basis.push(qn);
            chosen.push(best_idx);

            let new_rmse = rms(&residual);
            if new_rmse > 0.95 * current_rmse && new_rmse > 1e-12 * scale && chosen.len() > 1 {
                chosen.pop();
                ortho_basis.pop();
                break;
            }
            current_rmse = new_rmse;
            if current_rmse < 1e-13 * scale {
                break;
            }
        }
        chosen
    };

    // Orthonormal basis of a support set plus the deflated target residual.
    let residual_after = |support: &[usize]| -> (Vec<Vec<f64>>, Vec<f64>) {
        let mut basis: Vec<Vec<f64>> = Vec::with_capacity(support.len());
        for &c in support {
            let col = &columns[c].as_ref().unwrap().0;
            let (q, nq) = orthogonalize(col, &basis);
            if nq > 1e-12 {
                basis.push(q.into_iter().map(|v| v / nq).collect());
            }
        }
        let mut r = sub_target.clone();
        for b in &basis {
            let proj: f64 = r.iter().zip(b).map(|(a, x)| a * x).sum();
            for (ri, bi) in r.iter_mut().zip(b) {
                *ri -= proj * bi;
            }
        }
        (basis, r)
    };

    // Backfitting: cyclically try to replace each chosen column with the
    // dictionary column that best explains the residual left by the others.
    // The initial greedy pick often locks onto a "compromise" column;
    // swapping columns one at a time escapes it.
    let backfit = |mut chosen: Vec<usize>| -> Vec<usize> {
        for _pass in 0..3 {
            if deadline.map_or(false, |dl| Instant::now() >= dl) {
                break;
            }
            let mut changed = false;
            for slot in 0..chosen.len() {
                let others: Vec<usize> = chosen
                    .iter()
                    .enumerate()
                    .filter(|(s, _)| *s != slot)
                    .map(|(_, &c)| c)
                    .collect();
                let (basis, r) = residual_after(&others);
                let best = columns
                    .par_iter()
                    .enumerate()
                    .filter_map(|(idx, c)| {
                        if others.contains(&idx) {
                            return None;
                        }
                        let (col, norm) = c.as_ref()?;
                        let (q, q_norm) = orthogonalize(col, &basis);
                        if q_norm < 1e-8 * norm || !q_norm.is_finite() {
                            return None;
                        }
                        let dot: f64 = q.iter().zip(&r).map(|(a, b2)| a * b2).sum();
                        let score = (dot / q_norm).abs();
                        if score.is_finite() {
                            Some((idx, score))
                        } else {
                            None
                        }
                    })
                    .reduce_with(|a, b| if a.1 >= b.1 { a } else { b });
                if let Some((best_idx, best_score)) = best {
                    if best_idx != chosen[slot] {
                        let cur_score = {
                            let (col, norm) = columns[chosen[slot]].as_ref().unwrap();
                            let (q, q_norm) = orthogonalize(col, &basis);
                            if q_norm < 1e-8 * norm {
                                0.0
                            } else {
                                let dot: f64 =
                                    q.iter().zip(&r).map(|(a, b2)| a * b2).sum();
                                (dot / q_norm).abs()
                            }
                        };
                        if best_score > cur_score * (1.0 + 1e-9) {
                            chosen[slot] = best_idx;
                            changed = true;
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
        chosen
    };

    // Full-data joint refit of a support set, followed by backward pruning:
    // drop every term whose removal costs less than 25% extra RMSE (with an
    // absolute floor at exactness level). True structural terms are
    // catastrophic to remove, while the "compromise"/junk columns the greedy
    // pursuit picked up barely matter — this is what keeps the output from
    // degenerating into long Plus chains of near-useless monomials.
    let finalize = |chosen: &[usize]| -> Option<(Vec<Term>, Option<Vec<Term>>)> {
        let mut terms: Vec<Term> = chosen
            .iter()
            .map(|&c| {
                let mut t = dictionary[c].clone();
                t.coeff = 1.0;
                t
            })
            .collect();
        let full_columns: Vec<Vec<f64>> = terms.iter().map(|t| t.eval_rows(inputs)).collect();
        if full_columns
            .iter()
            .any(|col| col.iter().any(|v| !v.is_finite()))
        {
            return None;
        }
        let pred = joint_refit(&mut terms, &full_columns, target)?;
        let full_scale = rms(target).max(1e-300);
        let initial_rmse = rmse(&pred, target);
        // A term survives only if removing it pushes the error above this.
        let removal_tolerance = (initial_rmse * 1.25).max(1e-10 * full_scale);

        let mut keep: Vec<bool> = vec![true; terms.len()];
        for i in 0..terms.len() {
            if keep.iter().filter(|&&k| k).count() <= 1 {
                break;
            }
            keep[i] = false;
            let mut trial: Vec<Term> = terms
                .iter()
                .zip(&keep)
                .filter(|(_, &k)| k)
                .map(|(t, _)| t.clone())
                .collect();
            let trial_cols: Vec<Vec<f64>> = full_columns
                .iter()
                .zip(&keep)
                .filter(|(_, &k)| k)
                .map(|(c, _)| c.clone())
                .collect();
            match joint_refit(&mut trial, &trial_cols, target) {
                Some(pred) => {
                    let err = rmse(&pred, target);
                    if err > removal_tolerance {
                        keep[i] = true;
                    }
                }
                None => keep[i] = true,
            }
        }
        let mut pruned: Vec<Term> = terms
            .iter()
            .zip(&keep)
            .filter(|(_, &k)| k)
            .map(|(t, _)| t.clone())
            .collect();
        // Refit against UNIT columns (coeff folded out), otherwise the
        // previously fitted coefficients would be applied twice.
        let pruned_cols: Vec<Vec<f64>> = full_columns
            .iter()
            .zip(&keep)
            .filter(|(_, &k)| k)
            .map(|(c, _)| c.clone())
            .collect();
        let pruned_pred = joint_refit(&mut pruned, &pruned_cols, target)?;
        // If pruning measurably worsened the fit, also keep the unpruned
        // version so the best-RMSE candidate is never lost — the pruned one
        // still wins on readability whenever the errors tie.
        let pruned_rmse = rmse(&pruned_pred, target);
        let fallback = if pruned.len() < terms.len() && pruned_rmse > initial_rmse * 1.01 {
            Some(terms)
        } else {
            None
        };
        Some((pruned, fallback))
    };

    // Try both seeding strategies and keep every distinct outcome: sums of
    // structural terms favor the unseeded start, near-constant targets
    // (offsets, Lorentz-style 1 - x) need the constant-seeded start.
    let mut results: Vec<Vec<Term>> = Vec::new();
    for seed_constant in [false, true] {
        let chosen = select(seed_constant);
        if chosen.is_empty() {
            continue;
        }
        let chosen = backfit(chosen);
        if let Some((pruned, fallback)) = finalize(&chosen) {
            results.push(pruned);
            if let Some(unpruned) = fallback {
                results.push(unpruned);
            }
        }
    }
    results
}

/// Applies a transform to |y| (sign folded separately). Returns None when the
/// transform is undefined for this data.
fn transform_target(t: Transform, ys_abs: &[f64]) -> Option<Vec<f64>> {
    let min_abs = ys_abs.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_abs = ys_abs.iter().cloned().fold(0.0f64, f64::max);
    if !max_abs.is_finite() || max_abs <= 0.0 {
        return None;
    }
    let needs_nonzero = !matches!(t, Transform::Id);
    if needs_nonzero && min_abs < 1e-12 * max_abs {
        return None;
    }
    let vals: Vec<f64> = match t {
        Transform::Id | Transform::Log1p => return Some(ys_abs.to_vec()),
        Transform::Log => ys_abs.iter().map(|&y| y.ln()).collect(),
        Transform::InvY => ys_abs.iter().map(|&y| 1.0 / y).collect(),
        Transform::InvY2 => ys_abs.iter().map(|&y| 1.0 / (y * y)).collect(),
        Transform::Y2 => ys_abs.iter().map(|&y| y * y).collect(),
    };
    if vals.iter().all(|v| v.is_finite()) {
        Some(vals)
    } else {
        None
    }
}

/// Numerically inverts a transform applied to the monomial-sum prediction.
fn invert_prediction(t: Transform, m: f64) -> f64 {
    match t {
        Transform::Id => m,
        Transform::Log => m.exp(),
        Transform::InvY => 1.0 / m,
        Transform::InvY2 => {
            if m > 0.0 {
                1.0 / m.sqrt()
            } else {
                f64::NAN
            }
        }
        Transform::Y2 => {
            if m >= 0.0 {
                m.sqrt()
            } else {
                f64::NAN
            }
        }
        Transform::Log1p => m.exp_m1(),
    }
}

/// Wraps the monomial-sum expression `m_expr` with the transform inverse.
fn invert_expression(
    t: Transform,
    m_expr: Expression,
    negate: bool,
    reg: &OperatorRegistry,
) -> Expression {
    let inner = match t {
        Transform::Id => m_expr,
        Transform::Log => build::unary("Exp", m_expr, reg),
        Transform::InvY => build::unary("Inv", m_expr, reg),
        Transform::InvY2 => build::unary("Inv", build::unary("Sqrt", m_expr, reg), reg),
        Transform::Y2 => build::unary("Sqrt", m_expr, reg),
        Transform::Log1p => build::binary(
            "Subtract",
            build::unary("Exp", m_expr, reg),
            build::num(1.0),
            reg,
        ),
    };
    if negate {
        build::unary("Neg", inner, reg)
    } else {
        inner
    }
}

/// Full-data RMSE of the assembled expression, evaluated exactly like the
/// downstream consumers will (complex stack machine over the registry).
fn expression_rmse(
    expr: &Expression,
    inputs: &[Vec<f64>],
    ys: &[f64],
    reg: &OperatorRegistry,
) -> f64 {
    use crate::core::value::{is_usable, real};
    let mut acc = 0.0;
    for (row, &y) in inputs.iter().zip(ys) {
        let vals: Vec<crate::core::value::Value> = row.iter().map(|&v| real(v)).collect();
        match expr.eval(&vals, reg) {
            Some(v) if is_usable(v) && v.im.abs() < 1e-6 * v.re.abs().max(1.0) => {
                let dfit = v.re - y;
                acc += dfit * dfit;
            }
            _ => return f64::INFINITY,
        }
    }
    (acc / ys.len() as f64).sqrt()
}

/// Prepares the (transform, target, negate) triples valid for this dataset.
fn prepare_targets(ys: &[f64]) -> Vec<(Transform, Vec<f64>, bool)> {
    let all_pos = ys.iter().all(|&y| y > 0.0);
    let all_neg = ys.iter().all(|&y| y < 0.0);
    let ys_abs: Vec<f64> = ys.iter().map(|y| y.abs()).collect();

    let mut out = Vec::new();
    for &t in TRANSFORMS {
        match t {
            Transform::Id => out.push((t, ys.to_vec(), false)),
            Transform::Log1p => {
                // Signed target; defined for 1 + y bounded away from zero.
                let shifted_ok = ys.iter().all(|&y| 1.0 + y > 1e-9);
                if shifted_ok {
                    let vals: Vec<f64> = ys.iter().map(|&y| y.ln_1p()).collect();
                    if vals.iter().all(|v| v.is_finite()) {
                        out.push((t, vals, false));
                    }
                }
            }
            _ => {
                if all_pos || all_neg {
                    if let Some(vals) = transform_target(t, &ys_abs) {
                        out.push((t, vals, all_neg));
                    }
                }
            }
        }
    }
    out
}

/// Validates a term-sum candidate on the full data and assembles the tree.
fn validate_and_assemble(
    t: Transform,
    negate: bool,
    terms: &[Term],
    inputs: &[Vec<f64>],
    ys: &[f64],
    reg: &OperatorRegistry,
    fits: &mut Vec<PowerlawFit>,
) {
    // Quick numeric validation before assembling the tree.
    let m_pred: Vec<f64> = {
        let cols: Vec<Vec<f64>> = terms.iter().map(|t| t.eval_rows(inputs)).collect();
        (0..ys.len())
            .map(|i| cols.iter().map(|c| c[i]).sum::<f64>())
            .collect()
    };
    let y_pred: Vec<f64> = m_pred
        .iter()
        .map(|&m| {
            let v = invert_prediction(t, m);
            if negate {
                -v
            } else {
                v
            }
        })
        .collect();
    if !rmse(&y_pred, ys).is_finite() {
        return;
    }

    let expr = invert_expression(t, build::term_sum_expression(terms, reg), negate, reg);
    let err = expression_rmse(&expr, inputs, ys, reg);
    if err.is_finite() {
        fits.push(PowerlawFit {
            expression: expr,
            error: err,
        });
    }
}

/// Runs the full Stage A pipeline. Returns candidate fits sorted by error.
///
/// Two passes: the basic monomial dictionaries run first (they solve the
/// bulk of physics formulas in milliseconds); the much larger
/// monomial-times-feature dictionary only runs when the basic pass failed to
/// reach exactness.
pub fn run_powerlaw(
    inputs: &[Vec<f64>],
    ys: &[f64],
    config: &SearchConfig,
    reg: &OperatorRegistry,
    deadline: Option<Instant>,
) -> Vec<PowerlawFit> {
    let mut fits: Vec<PowerlawFit> = Vec::new();
    if inputs.len() < 10 || inputs[0].is_empty() {
        return fits;
    }
    let usable = usable_variables(inputs);
    let targets = prepare_targets(ys);
    let y_std = {
        let mean = ys.iter().sum::<f64>() / ys.len() as f64;
        (ys.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / ys.len() as f64).sqrt()
    }
    .max(1e-30);
    let exact = |err: f64| err <= 1e-9 * y_std;

    // ---- Pass 1: greedy log-fit boosting + basic monomial dictionaries ----
    for (t, t_target, negate) in &targets {
        if deadline.map_or(false, |dl| Instant::now() >= dl) {
            break;
        }
        let mut candidates: Vec<Vec<Term>> = Vec::new();
        if let Some(terms) = greedy_monomial_fit(inputs, t_target, &usable, config.max_boost_terms)
        {
            candidates.push(terms);
        }
        candidates.extend(omp_monomial_fit(
            inputs,
            t_target,
            &usable,
            config.max_boost_terms,
            deadline,
            false,
        ));
        for terms in candidates {
            validate_and_assemble(*t, *negate, &terms, inputs, ys, reg, &mut fits);
        }
    }

    // ---- Pass 2: feature-augmented dictionary, only when still unsolved ----
    let best_so_far = fits.iter().map(|f| f.error).fold(f64::INFINITY, f64::min);
    if !exact(best_so_far) {
        for (t, t_target, negate) in &targets {
            if deadline.map_or(false, |dl| Instant::now() >= dl) {
                break;
            }
            for terms in omp_monomial_fit(
                inputs,
                t_target,
                &usable,
                config.max_boost_terms,
                deadline,
                true,
            ) {
                validate_and_assemble(*t, *negate, &terms, inputs, ys, reg, &mut fits);
            }
        }
    }

    fits.sort_by(|a, b| a.error.partial_cmp(&b.error).unwrap());
    fits
}

/// Best single-monomial "whitener" for Stage C: fits `|y| ≈ c * prod x^a`
/// and returns the monomial (rounded exponents preferred when they fit as
/// well). Working on |y| deliberately supports mixed-sign targets like
/// G*m1*m2*(1/r2 - 1/r1): the sign structure stays in the ratio, which the
/// downstream searches handle natively.
pub fn best_monomial_whitener(inputs: &[Vec<f64>], ys: &[f64]) -> Option<Monomial> {
    monomial_whitener_candidates(inputs, ys).into_iter().next()
}

/// All plausible whitener monomials, best whitening score first. The log-fit
/// exponents are ambiguous when the non-monomial factor correlates with some
/// variables (e.g. n*kb*T*ln(V2/V1) leaks exponent mass onto V1/V2), so the
/// caller should probe the leading few candidates rather than trust one.
pub fn monomial_whitener_candidates(inputs: &[Vec<f64>], ys: &[f64]) -> Vec<Monomial> {
    let mut scored = whitener_candidates_scored(inputs, ys);
    scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    scored.into_iter().map(|(m, _)| m).collect()
}

fn whitener_candidates_scored(inputs: &[Vec<f64>], ys: &[f64]) -> Vec<(Monomial, f64)> {
    if inputs.len() < 10 || inputs[0].is_empty() {
        return Vec::new();
    }
    let usable = usable_variables(inputs);
    if usable.is_empty() {
        return Vec::new();
    }
    let ys_abs: Vec<f64> = ys.iter().map(|y| y.abs()).collect();
    let raw_exps = match fit_monomial_log(inputs, &ys_abs, &usable) {
        Some((e, _)) => e,
        None => return Vec::new(),
    };

    // Rounded variants of the raw fit, plus per-variable zeroing of the
    // integer rounding: a non-monomial factor often leaks fractional
    // exponent mass onto its own variables, and zeroing them one at a time
    // recovers the clean whitener.
    let mut exps_candidates = exponent_candidates(&raw_exps);
    let den1: Vec<f64> = raw_exps
        .iter()
        .map(|&e| {
            let r = e.round();
            if r.abs() < 1e-9 {
                0.0
            } else {
                r
            }
        })
        .collect();
    for j in 0..den1.len() {
        if den1[j] != 0.0 {
            let mut z = den1.clone();
            z[j] = 0.0;
            if !exps_candidates.contains(&z) {
                exps_candidates.push(z);
            }
        }
    }
    // Zero every |exponent| < 1 in the raw fit (keeps only confident vars).
    let confident: Vec<f64> = raw_exps
        .iter()
        .map(|&e| {
            let r = e.round();
            if e.abs() < 0.6 || r.abs() < 1e-9 {
                0.0
            } else {
                r
            }
        })
        .collect();
    if !exps_candidates.contains(&confident) {
        exps_candidates.push(confident);
    }
    // Keep only near-unit exponents: a log-like factor ln(V2/V1) leaks large
    // symmetric exponents onto its own variables while the true monomial
    // factor keeps clean ±1 entries (e.g. n*kb*T*ln(V2/V1)).
    let unit_only: Vec<f64> = raw_exps
        .iter()
        .map(|&e| {
            if (e.abs() - 1.0).abs() <= 0.35 {
                e.signum()
            } else {
                0.0
            }
        })
        .collect();
    if !exps_candidates.contains(&unit_only) {
        exps_candidates.push(unit_only);
    }

    let mut scored: Vec<(Monomial, f64)> = Vec::new();
    for exps in exps_candidates {
        if exps.iter().all(|&e| e == 0.0) {
            continue;
        }
        let unit = Monomial {
            coeff: 1.0,
            exponents: exps.clone(),
        };
        let column = unit.eval_rows(inputs);
        if column.iter().any(|v| !v.is_finite() || v.abs() < 1e-300) {
            continue;
        }
        let coeff = match refit_coeff(&column, &ys_abs) {
            Some(c) => c,
            None => continue,
        };
        if coeff.abs() < 1e-300 {
            continue;
        }
        // Judge whitening quality by the spread of the log-ratio.
        let log_ratios: Vec<f64> = ys
            .iter()
            .zip(&column)
            .filter(|(y, c)| y.abs() > 1e-300 && c.abs() > 1e-300)
            .map(|(y, c)| (y.abs() / (coeff.abs() * c.abs())).ln())
            .collect();
        if log_ratios.len() < ys.len() / 2 {
            continue;
        }
        let mean = log_ratios.iter().sum::<f64>() / log_ratios.len() as f64;
        let var = log_ratios
            .iter()
            .map(|v| (v - mean) * (v - mean))
            .sum::<f64>()
            / log_ratios.len() as f64;
        // Prefer simpler (more-rounded) exponents on near-ties.
        let complexity_bonus = exps
            .iter()
            .map(|e| if (e * 2.0).fract().abs() < 1e-9 { 0.0 } else { 0.05 })
            .sum::<f64>();
        let score = var.sqrt() + complexity_bonus;
        scored.push((
            Monomial {
                coeff,
                exponents: exps,
            },
            score,
        ));
    }
    scored
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::registry::OperatorRegistry;

    fn grid2() -> (Vec<Vec<f64>>, OperatorRegistry) {
        let mut inputs = Vec::new();
        for i in 1..=20 {
            for j in 1..=20 {
                inputs.push(vec![1.0 + i as f64 * 0.2, 1.0 + j as f64 * 0.15]);
            }
        }
        (inputs, OperatorRegistry::with_builtins())
    }

    #[test]
    fn recovers_pure_monomial() {
        let (inputs, reg) = grid2();
        let ys: Vec<f64> = inputs
            .iter()
            .map(|r| 2.5 * r[0] * r[0] / r[1])
            .collect();
        let cfg = SearchConfig::fable_default();
        let fits = run_powerlaw(&inputs, &ys, &cfg, &reg, None);
        assert!(!fits.is_empty());
        let scale = rms(&ys);
        assert!(
            fits[0].error < 1e-8 * scale,
            "error too large: {}",
            fits[0].error
        );
    }

    #[test]
    fn recovers_sum_of_monomials() {
        let (inputs, reg) = grid2();
        let ys: Vec<f64> = inputs.iter().map(|r| r[0] * r[0] + 3.0 * r[1]).collect();
        let cfg = SearchConfig::fable_default();
        let fits = run_powerlaw(&inputs, &ys, &cfg, &reg, None);
        assert!(!fits.is_empty());
        let scale = rms(&ys);
        assert!(
            fits[0].error < 1e-8 * scale,
            "error too large: {}",
            fits[0].error
        );
    }

    #[test]
    fn recovers_exponential_of_monomial() {
        let (inputs, reg) = grid2();
        // y = 5 * exp(-0.5 * x0 * x1)
        let ys: Vec<f64> = inputs
            .iter()
            .map(|r| 5.0 * (-0.5 * r[0] * r[1]).exp())
            .collect();
        let cfg = SearchConfig::fable_default();
        let fits = run_powerlaw(&inputs, &ys, &cfg, &reg, None);
        assert!(!fits.is_empty());
        let scale = rms(&ys).max(1e-12);
        assert!(
            fits[0].error < 1e-7 * scale,
            "error too large: {}",
            fits[0].error
        );
    }

    #[test]
    fn recovers_lorentz_factor_via_inv_y2() {
        let mut inputs = Vec::new();
        for i in 1..=30 {
            for j in 1..=30 {
                // v in (1,3), c in (4,10) so v/c < 1
                inputs.push(vec![1.0 + i as f64 / 15.0, 4.0 + j as f64 / 5.0]);
            }
        }
        let reg = OperatorRegistry::with_builtins();
        // y = 1/sqrt(1 - v^2/c^2)
        let ys: Vec<f64> = inputs
            .iter()
            .map(|r| 1.0 / (1.0 - r[0] * r[0] / (r[1] * r[1])).sqrt())
            .collect();
        let cfg = SearchConfig::fable_default();
        let fits = run_powerlaw(&inputs, &ys, &cfg, &reg, None);
        assert!(!fits.is_empty());
        let scale = rms(&ys);
        assert!(
            fits[0].error < 1e-8 * scale,
            "error too large: {}",
            fits[0].error
        );
    }

    #[test]
    fn whitener_finds_monomial_factor() {
        let (inputs, _reg) = grid2();
        // y = 3 * x0^2 * sin-ish correction ~ 1 (pure monomial here)
        let ys: Vec<f64> = inputs.iter().map(|r| 3.0 * r[0] * r[0]).collect();
        let m = best_monomial_whitener(&inputs, &ys).expect("whitener");
        assert!((m.exponents[0] - 2.0).abs() < 1e-6);
        assert!(m.exponents[1].abs() < 1e-6);
    }
}



#[cfg(test)]
mod feature_tests {
    use super::*;
    use crate::ops::registry::OperatorRegistry;

    fn grid2() -> (Vec<Vec<f64>>, OperatorRegistry) {
        let mut inputs = Vec::new();
        for i in 1..=20 {
            for j in 1..=20 {
                inputs.push(vec![1.0 + i as f64 * 0.2, 1.0 + j as f64 * 0.15]);
            }
        }
        (inputs, OperatorRegistry::with_builtins())
    }

    fn best_error(inputs: &[Vec<f64>], ys: &[f64]) -> f64 {
        let reg = OperatorRegistry::with_builtins();
        let cfg = SearchConfig::fable_default();
        let fits = run_powerlaw(inputs, ys, &cfg, &reg, None);
        fits.first().map(|f| f.error).unwrap_or(f64::INFINITY)
    }

    #[test]
    fn recovers_monomial_times_sin() {
        // y = q*Ef + q*B*v*sin(theta) shape (I.12.11): x0*3 + x0*x1*sin(x1)?
        // keep it 2-var: y = 2*x0 + 0.5*x0*x1*... use sin on x1.
        let (inputs, _reg) = grid2();
        let ys: Vec<f64> = inputs
            .iter()
            .map(|r| 2.0 * r[0] + 1.5 * r[0] * r[1].sin())
            .collect();
        let scale = rms(&ys);
        let err = best_error(&inputs, &ys);
        assert!(err < 1e-8 * scale, "error too large: {err}");
    }

    #[test]
    fn recovers_sqrt_of_diff_squares() {
        // y = sqrt((x0-x1)^2 + 4) shape via Y2 transform + DiffSq feature.
        let (inputs, _reg) = grid2();
        let ys: Vec<f64> = inputs
            .iter()
            .map(|r| ((r[0] - r[1]).powi(2) + 4.0).sqrt())
            .collect();
        let scale = rms(&ys);
        let err = best_error(&inputs, &ys);
        assert!(err < 1e-8 * scale, "error too large: {err}");
    }

    #[test]
    fn recovers_monomial_times_ln() {
        // y = 3*x0*ln(x1) (I.44.4 shape).
        let (inputs, _reg) = grid2();
        let ys: Vec<f64> = inputs.iter().map(|r| 3.0 * r[0] * r[1].ln()).collect();
        let scale = rms(&ys);
        let err = best_error(&inputs, &ys);
        assert!(err < 1e-8 * scale, "error too large: {err}");
    }

    #[test]
    fn recovers_exp_minus_one_via_log1p() {
        // y = exp(0.5*x0*x1) - 1 (III.14.14 ratio shape).
        let mut inputs = Vec::new();
        for i in 1..=20 {
            for j in 1..=20 {
                inputs.push(vec![0.1 + i as f64 * 0.05, 0.1 + j as f64 * 0.05]);
            }
        }
        let ys: Vec<f64> = inputs
            .iter()
            .map(|r| (0.5 * r[0] * r[1]).exp_m1())
            .collect();
        let scale = rms(&ys);
        let err = best_error(&inputs, &ys);
        assert!(err < 1e-8 * scale, "error too large: {err}");
    }
}
