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
    /// t = ln((1 - y)/y), defined for y in (0, 1). Linearizes the logistic
    /// family y = 1/(1 + exp(m)) common in biology/chemistry/economics.
    Logit,
    /// t = asin(sqrt(y)), defined for y in (0, 1). Linearizes y = sin^2(m)
    /// (transition probabilities, interference fringes) as long as the
    /// argument stays within the principal branch.
    AsinSqrt,
    /// t = atanh(y), defined for |y| < 1. Linearizes y = tanh(m)
    /// (saturation laws with unit amplitude).
    Atanh,
}

const TRANSFORMS: &[Transform] = &[
    Transform::Id,
    Transform::Log,
    Transform::InvY,
    Transform::InvY2,
    Transform::Y2,
    Transform::Log1p,
    Transform::Logit,
    Transform::AsinSqrt,
    Transform::Atanh,
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
    // Tiny Tikhonov ridge keeps near-collinear systems stable. Scaled by the
    // mean diagonal so huge-magnitude columns (common in raw physical data)
    // get the same relative damping as O(1) columns.
    let trace: f64 = (0..m).map(|i| ata[i][i]).sum();
    let ridge = 1e-12 * (trace / m as f64).max(1.0);
    for i in 0..m {
        for j in 0..i {
            ata[i][j] = ata[j][i];
        }
        ata[i][i] += ridge;
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

/// Weighted RMS: rms of `v` with each entry scaled by the matching
/// sqrt-weight. Falls back to plain rms when no weights are given.
fn wrms(v: &[f64], sw: Option<&[f64]>) -> f64 {
    match sw {
        None => rms(v),
        Some(w) => {
            if v.is_empty() {
                return f64::INFINITY;
            }
            (v.iter()
                .zip(w)
                .map(|(x, s)| {
                    let y = x * s;
                    y * y
                })
                .sum::<f64>()
                / v.len() as f64)
                .sqrt()
        }
    }
}

#[inline]
fn sw_at(sw: Option<&[f64]>, i: usize) -> f64 {
    sw.map_or(1.0, |w| w[i])
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

/// Variables usable inside fractional-power monomials and Ln features
/// (strictly positive data).
fn usable_variables(inputs: &[Vec<f64>]) -> Vec<usize> {
    if inputs.is_empty() {
        return Vec::new();
    }
    let d = inputs[0].len();
    (0..d)
        .filter(|&j| inputs.iter().all(|row| row[j] > 1e-12 && row[j].is_finite()))
        .collect()
}

/// Variables usable inside integer-exponent monomials and trig/difference
/// features: any finite data, including negative and mixed-sign variables.
/// (Feynman happens to be all-positive; general data is not.)
fn real_variables(inputs: &[Vec<f64>]) -> Vec<usize> {
    if inputs.is_empty() {
        return Vec::new();
    }
    let d = inputs[0].len();
    (0..d)
        .filter(|&j| inputs.iter().all(|row| row[j].is_finite()))
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
    joint_refit_weighted(terms, columns, target, None)
}

/// Weighted joint refit: the least-squares fit runs on rows scaled by the
/// sqrt-weights (WLS), while the returned prediction stays in the unscaled
/// space so residual bookkeeping is unchanged.
fn joint_refit_weighted(
    terms: &mut [Term],
    columns: &[Vec<f64>],
    target: &[f64],
    sw: Option<&[f64]>,
) -> Option<Vec<f64>> {
    let n = target.len();
    let design: Vec<Vec<f64>> = (0..n)
        .map(|i| {
            let s = sw_at(sw, i);
            columns.iter().map(|c| c[i] * s).collect()
        })
        .collect();
    let scaled_target: Vec<f64> = (0..n).map(|i| target[i] * sw_at(sw, i)).collect();
    let coeffs = lstsq(&design, &scaled_target)?;
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
///
/// `sw` are optional per-row sqrt-weights (WLS in a transformed space): the
/// exponent proposal still uses the raw residual (log-space structure), but
/// every least-squares fit and every progress decision runs weighted.
fn greedy_monomial_fit(
    inputs: &[Vec<f64>],
    target: &[f64],
    usable: &[usize],
    max_terms: usize,
    sw: Option<&[f64]>,
) -> Option<Vec<Term>> {
    let mut terms: Vec<Term> = Vec::new();
    let mut unit_columns: Vec<Vec<f64>> = Vec::new();
    let mut residual = target.to_vec();
    let mut current_rmse = wrms(&residual, sw);
    let n = target.len();

    // Weighted intercept+slope fit of `col` against `res` restricted to
    // `rows`; returns (weighted rms over `err_rows`, coeff, intercept).
    let fit2 = |col: &[f64], res: &[f64], rows: &[usize], err_rows: &[usize]| -> Option<(f64, f64, f64)> {
        let design: Vec<Vec<f64>> = rows
            .iter()
            .map(|&i| {
                let s = sw_at(sw, i);
                vec![s, s * col[i]]
            })
            .collect();
        let t: Vec<f64> = rows.iter().map(|&i| res[i] * sw_at(sw, i)).collect();
        let sol = lstsq(&design, &t)?;
        let mut acc = 0.0;
        for &i in err_rows {
            let d = (res[i] - sol[0] - sol[1] * col[i]) * sw_at(sw, i);
            if !d.is_finite() {
                return None;
            }
            acc += d * d;
        }
        Some(((acc / err_rows.len().max(1) as f64).sqrt(), sol[1], sol[0]))
    };
    let all_rows: Vec<usize> = (0..n).collect();
    // Deterministic 80/20 split for the continuous-exponent refinement: the
    // refined exponent must win on held-out rows, otherwise it is chasing
    // the noise floor and the rounded exponent is kept.
    let train_rows: Vec<usize> = (0..n).filter(|i| i % 5 != 4).collect();
    let val_rows: Vec<usize> = (0..n).filter(|i| i % 5 == 4).collect();

    for _ in 0..max_terms {
        let (raw_exps, _) = match fit_monomial_log(inputs, &residual, usable) {
            Some(f) => f,
            None => break,
        };

        // Pick the exponent rounding whose refitted term reduces the residual
        // the most. The fit includes an intercept so that a constant offset
        // (e.g. 1/y = 1/E + (K^n/E) x^{-n} in a Hill law) does not drag the
        // exponent estimate toward a compromise value.
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
            let (err, coeff, _c0) = match fit2(&column, &residual, &all_rows, &all_rows) {
                Some(v) => v,
                None => continue,
            };
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
        let (mut term, mut column, mut err) = best?;

        // Single-variable continuous-exponent refinement: when the term uses
        // exactly one variable, the true exponent may be far from every
        // rounding candidate (the raw log-fit is dragged by an intercept,
        // e.g. Hill laws). Ternary-search the exponent, scoring on the
        // held-out 20% with coefficients fit on the other 80% so a noisy
        // dataset cannot drag the exponent off a clean rational value.
        let active: Vec<usize> = (0..term.exponents.len())
            .filter(|&j| term.exponents[j] != 0.0)
            .collect();
        if active.len() == 1 && !val_rows.is_empty() {
            let j = active[0];
            let col_for_exp = |e: f64| -> Option<Vec<f64>> {
                let mut exps = vec![0.0; term.exponents.len()];
                exps[j] = e;
                let col = Monomial {
                    coeff: 1.0,
                    exponents: exps,
                }
                .eval_rows(inputs);
                if col.iter().any(|v| !v.is_finite()) {
                    None
                } else {
                    Some(col)
                }
            };
            let val_score = |e: f64| -> f64 {
                col_for_exp(e)
                    .and_then(|col| fit2(&col, &residual, &train_rows, &val_rows))
                    .map(|v| v.0)
                    .unwrap_or(f64::INFINITY)
            };
            let (mut lo, mut hi) = (term.exponents[j] - 2.0, term.exponents[j] + 2.0);
            for _ in 0..40 {
                let m1 = lo + (hi - lo) / 3.0;
                let m2 = hi - (hi - lo) / 3.0;
                if val_score(m1) <= val_score(m2) {
                    hi = m2;
                } else {
                    lo = m1;
                }
            }
            let e_star = (lo + hi) / 2.0;
            // Keep the rounded exponent unless the refined one is a clear
            // (>2%) validation improvement.
            let val_rounded = val_score(term.exponents[j]);
            let val_refined = val_score(e_star);
            if val_refined < val_rounded * 0.98 {
                if let Some(col) = col_for_exp(e_star) {
                    if let Some((e_err, coeff, _c0)) = fit2(&col, &residual, &all_rows, &all_rows)
                    {
                        if e_err < err {
                            term.exponents[j] = e_star;
                            term.coeff = coeff;
                            column = col;
                            err = e_err;
                        }
                    }
                }
            }
        }

        // Require a meaningful reduction to keep adding terms.
        if err > 0.85 * current_rmse {
            break;
        }
        let has_const = terms
            .iter()
            .any(|t| t.exponents.iter().all(|&e| e == 0.0) && t.feature == Feature::None);
        if !has_const && !term.exponents.iter().all(|&e| e == 0.0) {
            terms.push(Term {
                coeff: 0.0,
                exponents: vec![0.0; inputs[0].len()],
                feature: Feature::None,
            });
            unit_columns.push(vec![1.0; target.len()]);
        }
        terms.push(term);
        unit_columns.push(column);

        // Joint refit of all coefficients keeps the greedy path numerically exact.
        match joint_refit_weighted(&mut terms, &unit_columns, target, sw) {
            Some(pred) => {
                residual = target.iter().zip(&pred).map(|(t, p)| t - p).collect();
                current_rmse = wrms(&residual, sw);
            }
            None => break,
        }
        let scale = wrms(target, sw).max(1e-300);
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
fn build_feature_dictionary(
    d: usize,
    positive: &[usize],
    real: &[usize],
    size_limit: usize,
) -> Vec<Term> {
    let mut features: Vec<Feature> = Vec::new();
    for &i in real {
        features.push(Feature::Sin(i));
        features.push(Feature::Cos(i));
        features.push(Feature::Sin2x(i));
        features.push(Feature::Cos2x(i));
        features.push(Feature::Sigmoid(i));
        features.push(Feature::TanhVar(i));
    }
    for &i in positive {
        features.push(Feature::Ln(i));
    }
    for (a, &i) in real.iter().enumerate() {
        for &j in real.iter().skip(a + 1) {
            features.push(Feature::DiffSq(i, j));
            features.push(Feature::AbsDiff(i, j));
            // The sigmoid gate is not symmetric: keep both orientations.
            features.push(Feature::SigmoidDiff(i, j));
            features.push(Feature::SigmoidDiff(j, i));
            features.push(Feature::CosDiff(i, j));
            features.push(Feature::CosProd(i, j));
            features.push(Feature::SinProd(i, j));
            features.push(Feature::Cos2Prod(i, j));
            features.push(Feature::Sin2Prod(i, j));
        }
    }
    if features.is_empty() {
        return Vec::new();
    }

    // Pick the richest monomial configuration that fits the budget. Halves
    // come first: interference terms like sqrt(I1)*sqrt(I2)*cos(delta) need
    // fractional exponents next to the feature factor.
    const CONFIGS: &[(&[f64], usize)] = &[
        (&[-2.0, -1.0, -0.5, 0.5, 1.0, 2.0], 3),
        (&[-2.0, -1.0, 1.0, 2.0], 3),
        (&[-2.0, -1.0, 1.0, 2.0], 2),
        (&[-1.0, 1.0], 3),
        (&[-1.0, 1.0], 2),
        (&[-1.0, 1.0], 1),
    ];
    let per_feature_budget = size_limit / features.len();
    // Fractional exponents only make sense on positive variables; when no
    // variable is strictly positive, fractional configurations would leave
    // the monomial part empty, so they are skipped outright.
    let (exps, cap) = CONFIGS
        .iter()
        .filter(|(e, _)| positive.is_empty() == false || e.iter().all(|x| x.fract() == 0.0))
        .find(|(e, c)| dict_size_estimate(real.len(), e.len(), *c) <= per_feature_budget)
        .copied()
        .unwrap_or((&[-1.0, 1.0], 1));

    let has_fractional = exps.iter().any(|e| e.fract() != 0.0);
    let mono_vars: &[usize] = if has_fractional { positive } else { real };
    let base = enumerate_exponents(d, mono_vars, exps, cap.min(mono_vars.len().max(1)));
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
    positive: &[usize],
    real: &[usize],
    max_terms: usize,
    deadline: Option<Instant>,
    with_features: bool,
    sw: Option<&[f64]>,
) -> Vec<Vec<Term>> {
    const FRACTIONAL: &[f64] = &[-3.0, -2.0, -1.0, -0.5, 0.5, 1.0, 2.0, 3.0];
    // Two integer dictionaries: the compact +-2 one keeps the selection
    // clean for feature-mixed supports (e.g. ln(n0) - m*g*x/(kb*T)), while
    // the +-3 one covers cubes on mixed-sign variables (x^3 on (-3,3)
    // cannot come from the positive-only fractional dictionary).
    const INTEGER2: &[f64] = &[-2.0, -1.0, 1.0, 2.0];
    const INTEGER3: &[f64] = &[-3.0, -2.0, -1.0, 1.0, 2.0, 3.0];
    let d = match inputs.first() {
        Some(row) => row.len(),
        None => return Vec::new(),
    };
    if real.is_empty() {
        return Vec::new();
    }

    let mut results: Vec<Vec<Term>> = Vec::new();
    if with_features {
        let dictionary = build_feature_dictionary(d, positive, real, 150_000);
        if !dictionary.is_empty() {
            results.extend(pursue_dictionary(
                inputs,
                target,
                &dictionary,
                max_terms,
                deadline,
                sw,
            ));
        }
    } else {
        for (exps, vars, max_active) in [
            (FRACTIONAL, positive, 4usize),
            (INTEGER3, real, 5usize),
            (INTEGER2, real, 5usize),
        ] {
            if vars.is_empty() {
                continue;
            }
            let mut dictionary = build_dictionary(d, vars, exps, max_active, 70_000);
            if exps == INTEGER2 {
                // Bare single-variable features ride along with the integer
                // dictionary: mixed supports like ln(n0) - m*g*x/(kb*T)
                // need a deep monomial AND a lone feature in one pursuit.
                for &i in real {
                    for feature in [
                        Feature::Sin(i),
                        Feature::Cos(i),
                        Feature::Sin2x(i),
                        Feature::Cos2x(i),
                        Feature::Sigmoid(i),
                        Feature::TanhVar(i),
                    ] {
                        dictionary.push(Term {
                            coeff: 1.0,
                            exponents: vec![0.0; d],
                            feature,
                        });
                    }
                }
                for &i in positive {
                    dictionary.push(Term {
                        coeff: 1.0,
                        exponents: vec![0.0; d],
                        feature: Feature::Ln(i),
                    });
                }
            }
            results.extend(pursue_dictionary(
                inputs,
                target,
                &dictionary,
                max_terms,
                deadline,
                sw,
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
    sw: Option<&[f64]>,
) -> Vec<Vec<Term>> {
    pursue_dictionary_with_starts(inputs, target, dictionary, max_terms, deadline, &[], sw)
}

fn pursue_dictionary_with_starts(
    inputs: &[Vec<f64>],
    target: &[f64],
    dictionary: &[Term],
    max_terms: usize,
    deadline: Option<Instant>,
    extra_starts: &[usize],
    sw: Option<&[f64]>,
) -> Vec<Vec<Term>> {
    // Deterministic subsample, split 80/20 into a fit part (used for
    // selection and coefficients) and a validation part (used to decide
    // whether a term genuinely helps — under noise the fit residual keeps
    // shrinking with junk terms while the validation error does not).
    let n = target.len();
    let sub_n = n.min(250);
    let all_idx: Vec<usize> = (0..sub_n).map(|i| i * n / sub_n).collect();
    let mut fit_rows: Vec<usize> = Vec::with_capacity(sub_n);
    let mut val_rows: Vec<usize> = Vec::with_capacity(sub_n / 5 + 1);
    for (pos, &i) in all_idx.iter().enumerate() {
        if pos % 5 == 4 && sub_n >= 25 {
            val_rows.push(i);
        } else {
            fit_rows.push(i);
        }
    }

    // Precompute normalized dictionary columns (fit part + validation part).
    // With sqrt-weights the whole pursuit runs in the row-scaled space: the
    // fitted coefficients are then exactly the WLS estimates for the
    // unscaled features.
    let eval_term = |term: &Term, rows: &[usize]| -> Option<Vec<f64>> {
        let mut col = Vec::with_capacity(rows.len());
        for &i in rows {
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
            col.push(acc * sw_at(sw, i));
        }
        Some(col)
    };
    let columns: Vec<Option<(Vec<f64>, f64, Vec<f64>)>> = dictionary
        .par_iter()
        .map(|term| {
            let col = eval_term(term, &fit_rows)?;
            let val_col = eval_term(term, &val_rows)?;
            let norm = col.iter().map(|v| v * v).sum::<f64>().sqrt();
            if norm < 1e-300 || !norm.is_finite() {
                None
            } else {
                Some((col, norm, val_col))
            }
        })
        .collect();

    let sub_target: Vec<f64> = fit_rows.iter().map(|&i| target[i] * sw_at(sw, i)).collect();
    let val_target: Vec<f64> = val_rows.iter().map(|&i| target[i] * sw_at(sw, i)).collect();
    let scale = rms(&sub_target).max(1e-300);

    // Validation RMSE of a support, with coefficients fit on the fit part.
    let validation_error = |chosen: &[usize]| -> f64 {
        if chosen.is_empty() || val_target.is_empty() {
            return f64::INFINITY;
        }
        let design: Vec<Vec<f64>> = (0..sub_target.len())
            .map(|i| {
                chosen
                    .iter()
                    .map(|&c| columns[c].as_ref().unwrap().0[i])
                    .collect()
            })
            .collect();
        let coeffs = match lstsq(&design, &sub_target) {
            Some(c) => c,
            None => return f64::INFINITY,
        };
        let pred: Vec<f64> = (0..val_target.len())
            .map(|i| {
                chosen
                    .iter()
                    .zip(&coeffs)
                    .map(|(&c, k)| columns[c].as_ref().unwrap().2[i] * k)
                    .sum::<f64>()
            })
            .collect();
        rmse(&pred, &val_target)
    };

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
    let select = |seed_constant: bool, forced_first: Option<usize>| -> Vec<usize> {
        let mut ortho_basis: Vec<Vec<f64>> = Vec::new();
        let mut residual = sub_target.clone();
        let mut chosen: Vec<usize> = Vec::new();
        let mut current_rmse = rms(&residual);
        let mut current_val = f64::INFINITY;

        if seed_constant {
            if let Some((col, norm, _)) = columns[0].as_ref() {
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

        // Optional forced first pick (multi-start escape from compromise
        // columns that dominate the initial correlation ranking).
        if let Some(idx) = forced_first {
            if !chosen.contains(&idx) {
                if let Some((col, norm, _)) = columns[idx].as_ref() {
                    let mut q = col.clone();
                    for b in &ortho_basis {
                        let proj: f64 = col.iter().zip(b).map(|(a, x)| a * x).sum();
                        for (qi, bi) in q.iter_mut().zip(b) {
                            *qi -= proj * bi;
                        }
                    }
                    let q_norm = q.iter().map(|v| v * v).sum::<f64>().sqrt();
                    if q_norm > 1e-8 * norm && q_norm.is_finite() {
                        let qn: Vec<f64> = q.iter().map(|v| v / q_norm).collect();
                        let r_dot: f64 =
                            residual.iter().zip(&qn).map(|(a, b)| a * b).sum();
                        for (ri, bi) in residual.iter_mut().zip(&qn) {
                            *ri -= r_dot * bi;
                        }
                        ortho_basis.push(qn);
                        chosen.push(idx);
                        current_rmse = rms(&residual);
                    }
                }
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
                    let (col, norm, _) = c.as_ref()?;
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
            // Validation gate: reject terms that only fit the noise.
            let new_val = validation_error(&chosen);
            if new_val > current_val * (1.0 + 1e-9) && new_val > 1e-12 * scale && chosen.len() > 1
            {
                chosen.pop();
                ortho_basis.pop();
                break;
            }
            current_val = new_val;
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
        for _pass in 0..6 {
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
                        let (col, norm, _) = c.as_ref()?;
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
                            let (col, norm, _) = columns[chosen[slot]].as_ref().unwrap();
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
        let pred = joint_refit_weighted(&mut terms, &full_columns, target, sw)?;
        let full_scale = wrms(target, sw).max(1e-300);
        let initial_rmse = {
            let diff: Vec<f64> = pred.iter().zip(target).map(|(p, t)| p - t).collect();
            wrms(&diff, sw)
        };

        // Pruning decisions run on a held-out 20% of the data with
        // coefficients fit on the other 80%: junk terms that only chase the
        // noise do not survive a validation check.
        let train_rows: Vec<usize> = (0..target.len()).filter(|i| i % 5 != 4).collect();
        let val_rows: Vec<usize> = (0..target.len()).filter(|i| i % 5 == 4).collect();
        let holdout_error = |keep: &[bool]| -> f64 {
            let active: Vec<usize> = (0..keep.len()).filter(|&i| keep[i]).collect();
            if active.is_empty() || val_rows.is_empty() {
                return f64::INFINITY;
            }
            let design: Vec<Vec<f64>> = train_rows
                .iter()
                .map(|&r| {
                    let s = sw_at(sw, r);
                    active.iter().map(|&c| full_columns[c][r] * s).collect()
                })
                .collect();
            let train_t: Vec<f64> = train_rows.iter().map(|&r| target[r] * sw_at(sw, r)).collect();
            let coeffs = match lstsq(&design, &train_t) {
                Some(c) => c,
                None => return f64::INFINITY,
            };
            let pred: Vec<f64> = val_rows
                .iter()
                .map(|&r| {
                    let s = sw_at(sw, r);
                    active
                        .iter()
                        .zip(&coeffs)
                        .map(|(&c, k)| full_columns[c][r] * k * s)
                        .sum::<f64>()
                })
                .collect();
            let val_t: Vec<f64> = val_rows.iter().map(|&r| target[r] * sw_at(sw, r)).collect();
            rmse(&pred, &val_t)
        };

        let mut keep: Vec<bool> = vec![true; terms.len()];
        let initial_val = holdout_error(&keep);
        // A term survives only if removing it pushes the validation error
        // above this.
        let removal_tolerance = (initial_val * 1.25).max(1e-10 * full_scale);

        for i in 0..terms.len() {
            if keep.iter().filter(|&&k| k).count() <= 1 {
                break;
            }
            keep[i] = false;
            if holdout_error(&keep) > removal_tolerance {
                keep[i] = true;
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
        let pruned_pred = joint_refit_weighted(&mut pruned, &pruned_cols, target, sw)?;
        // If pruning measurably worsened the fit, also keep the unpruned
        // version so the best-RMSE candidate is never lost — the pruned one
        // still wins on readability whenever the errors tie.
        let pruned_rmse = {
            let diff: Vec<f64> = pruned_pred.iter().zip(target).map(|(p, t)| p - t).collect();
            wrms(&diff, sw)
        };
        let fallback = if pruned.len() < terms.len() && pruned_rmse > initial_rmse * 1.01 {
            Some(terms)
        } else {
            None
        };

        // P5-1: robust (Huber-IRLS) refit of the pruned support. Real data
        // carries occasional outliers (sensor glitches, recording errors)
        // that drag the least-squares coefficients; IRLS caps their
        // influence. The robust coefficients are kept only when the
        // held-out error improves.
        if !val_rows.is_empty() && !pruned.is_empty() {
            let val_err_for = |coeffs: &[f64]| -> f64 {
                let mut acc = 0.0;
                for &r in &val_rows {
                    let pred: f64 = coeffs
                        .iter()
                        .zip(&pruned_cols)
                        .map(|(c, col)| c * col[r])
                        .sum();
                    let d = (pred - target[r]) * sw_at(sw, r);
                    if !d.is_finite() {
                        return f64::INFINITY;
                    }
                    acc += d * d;
                }
                (acc / val_rows.len() as f64).sqrt()
            };
            let ols_coeffs: Vec<f64> = pruned.iter().map(|t| t.coeff).collect();
            let base_val = val_err_for(&ols_coeffs);
            let mut hcoeffs = ols_coeffs.clone();
            for _ in 0..3 {
                let resid: Vec<f64> = (0..target.len())
                    .map(|r| {
                        let pred: f64 = hcoeffs
                            .iter()
                            .zip(&pruned_cols)
                            .map(|(c, col)| c * col[r])
                            .sum();
                        (target[r] - pred) * sw_at(sw, r)
                    })
                    .collect();
                let mut abs: Vec<f64> = resid.iter().map(|v| v.abs()).collect();
                abs.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let mad = abs[abs.len() / 2] * 1.4826;
                if !(mad.is_finite()) || mad < 1e-14 * full_scale {
                    break;
                }
                let cutoff = 1.345 * mad;
                let design: Vec<Vec<f64>> = train_rows
                    .iter()
                    .map(|&r| {
                        let hw = (cutoff / resid[r].abs().max(1e-300)).min(1.0).sqrt();
                        let s = sw_at(sw, r) * hw;
                        pruned_cols.iter().map(|col| col[r] * s).collect()
                    })
                    .collect();
                let t_w: Vec<f64> = train_rows
                    .iter()
                    .map(|&r| {
                        let hw = (cutoff / resid[r].abs().max(1e-300)).min(1.0).sqrt();
                        target[r] * sw_at(sw, r) * hw
                    })
                    .collect();
                match lstsq(&design, &t_w) {
                    Some(c) if c.iter().all(|v| v.is_finite()) => hcoeffs = c,
                    _ => break,
                }
            }
            if val_err_for(&hcoeffs) < base_val * 0.999 {
                for (t, &c) in pruned.iter_mut().zip(&hcoeffs) {
                    t.coeff = c;
                }
            }
        }

        Some((pruned, fallback))
    };

    // Try both seeding strategies and keep every distinct outcome: sums of
    // structural terms favor the unseeded start, near-constant targets
    // (offsets, Lorentz-style 1 - x) need the constant-seeded start.
    // For small dictionaries, additionally multi-start on the top initial
    // correlation columns: the greedy first pick sometimes locks onto a
    // "compromise" column (e.g. sqrt(I1*I2) for I1 + I2 + interference)
    // that single-column backfitting cannot undo.
    let mut starts: Vec<Option<usize>> = vec![None];
    if dictionary.len() <= 20_000 {
        let mut ranking: Vec<(usize, f64)> = columns
            .par_iter()
            .enumerate()
            .filter_map(|(idx, c)| {
                let (col, norm, _) = c.as_ref()?;
                let dot: f64 = col.iter().zip(&sub_target).map(|(a, b)| a * b).sum();
                let score = (dot / norm).abs();
                if score.is_finite() {
                    Some((idx, score))
                } else {
                    None
                }
            })
            .collect();
        ranking.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        starts.extend(ranking.into_iter().take(4).map(|(idx, _)| Some(idx)));
    }
    for &s in extra_starts {
        if s < dictionary.len() && !starts.contains(&Some(s)) {
            starts.push(Some(s));
        }
    }

    let mut results: Vec<Vec<Term>> = Vec::new();
    let mut seen_supports: Vec<Vec<usize>> = Vec::new();
    for seed_constant in [false, true] {
        for &forced in &starts {
            if deadline.map_or(false, |dl| Instant::now() >= dl) {
                break;
            }
            let chosen = select(seed_constant, forced);
            if chosen.is_empty() {
                continue;
            }
            let chosen = backfit(chosen);
            let mut support = chosen.clone();
            support.sort_unstable();
            if seen_supports.contains(&support) {
                continue;
            }
            seen_supports.push(support);
            if let Some((pruned, fallback)) = finalize(&chosen) {
                results.push(pruned);
                if let Some(unpruned) = fallback {
                    results.push(unpruned);
                }
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
    // Only the reciprocal transforms blow up near zero. Log handles tiny
    // positive values fine — exponential laws legitimately span dozens of
    // orders of magnitude (e.g. n0*exp(-m*g*x/(kb*T)) over wide ranges).
    let needs_nonzero = matches!(t, Transform::InvY | Transform::InvY2);
    if needs_nonzero && min_abs < 1e-12 * max_abs {
        return None;
    }
    if matches!(t, Transform::Log) && min_abs < 1e-300 {
        return None;
    }
    let vals: Vec<f64> = match t {
        Transform::Id | Transform::Log1p | Transform::Logit | Transform::AsinSqrt
        | Transform::Atanh => return Some(ys_abs.to_vec()),
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
        Transform::Logit => 1.0 / (1.0 + m.exp()),
        Transform::AsinSqrt => {
            let s = m.sin();
            s * s
        }
        Transform::Atanh => m.tanh(),
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
        Transform::Logit => build::unary(
            "Inv",
            build::binary(
                "Plus",
                build::num(1.0),
                build::unary("Exp", m_expr, reg),
                reg,
            ),
            reg,
        ),
        Transform::AsinSqrt => build::unary("Square", build::unary("Sin", m_expr, reg), reg),
        Transform::Atanh => build::unary("Tanh", m_expr, reg),
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

/// Per-row sqrt-weights for weighted least squares in a transformed space.
///
/// Gaussian noise of size sigma on y becomes non-uniform noise of size
/// sigma * |T'(y_i)| on the transformed target — OLS then over-fits the
/// rows where the transform amplifies the noise (e.g. tiny y under 1/y^2).
/// Weighting each row by w_i = 1 / |T'(y_i)| restores equal noise variance.
/// Returns sqrt-weights (design rows and targets get scaled by them), or
/// `None` when the transform does not distort the noise (identity).
fn transform_sqrt_weights(t: Transform, ys: &[f64]) -> Option<Vec<f64>> {
    let raw: Vec<f64> = match t {
        Transform::Id => return None,
        // |T'(y)| = 1/y            -> w = |y|
        Transform::Log => ys.iter().map(|&y| y.abs()).collect(),
        // |T'(y)| = 1/y^2          -> w = y^2
        Transform::InvY => ys.iter().map(|&y| y * y).collect(),
        // |T'(y)| = 2/|y|^3        -> w = |y|^3 / 2
        Transform::InvY2 => ys.iter().map(|&y| y.abs().powi(3) / 2.0).collect(),
        // |T'(y)| = 2|y|           -> w = 1 / (2|y|)
        Transform::Y2 => ys.iter().map(|&y| 1.0 / (2.0 * y.abs().max(1e-300))).collect(),
        // |T'(y)| = 1/(1+y)        -> w = 1 + y
        Transform::Log1p => ys.iter().map(|&y| (1.0 + y).abs()).collect(),
        // |T'(y)| = 1/(y(1-y))     -> w = y(1-y)
        Transform::Logit => ys.iter().map(|&y| (y * (1.0 - y)).abs()).collect(),
        // |T'(y)| = 1/(2 sqrt(y(1-y))) -> w = 2 sqrt(y(1-y))
        Transform::AsinSqrt => ys
            .iter()
            .map(|&y| 2.0 * (y * (1.0 - y)).max(0.0).sqrt())
            .collect(),
        // |T'(y)| = 1/(1-y^2)      -> w = 1 - y^2
        Transform::Atanh => ys.iter().map(|&y| (1.0 - y * y).abs()).collect(),
    };
    let max = raw.iter().cloned().fold(0.0f64, f64::max);
    if !max.is_finite() || max <= 0.0 {
        return None;
    }
    // Normalize to max 1 and clip from below so no row is fully discarded.
    Some(raw.iter().map(|&w| (w / max).max(1e-3).sqrt()).collect())
}

/// Prepares the (transform, target, negate, sqrt-weights) tuples valid for
/// this dataset.
fn prepare_targets(ys: &[f64]) -> Vec<(Transform, Vec<f64>, bool, Option<Vec<f64>>)> {
    let all_pos = ys.iter().all(|&y| y > 0.0);
    let all_neg = ys.iter().all(|&y| y < 0.0);
    let ys_abs: Vec<f64> = ys.iter().map(|y| y.abs()).collect();

    let mut out = Vec::new();
    for &t in TRANSFORMS {
        match t {
            Transform::Id => out.push((t, ys.to_vec(), false, None)),
            Transform::Log1p => {
                // Signed target; defined for 1 + y bounded away from zero.
                let shifted_ok = ys.iter().all(|&y| 1.0 + y > 1e-9);
                if shifted_ok {
                    let vals: Vec<f64> = ys.iter().map(|&y| y.ln_1p()).collect();
                    if vals.iter().all(|v| v.is_finite()) {
                        let sw = transform_sqrt_weights(t, ys);
                        out.push((t, vals, false, sw));
                    }
                }
            }
            Transform::Logit => {
                if ys.iter().all(|&y| y > 1e-9 && y < 1.0 - 1e-9) {
                    let vals: Vec<f64> = ys.iter().map(|&y| ((1.0 - y) / y).ln()).collect();
                    if vals.iter().all(|v| v.is_finite()) {
                        let sw = transform_sqrt_weights(t, ys);
                        out.push((t, vals, false, sw));
                    }
                }
            }
            Transform::AsinSqrt => {
                if ys.iter().all(|&y| y > 1e-9 && y < 1.0 - 1e-9) {
                    let vals: Vec<f64> = ys.iter().map(|&y| y.sqrt().asin()).collect();
                    if vals.iter().all(|v| v.is_finite()) {
                        let sw = transform_sqrt_weights(t, ys);
                        out.push((t, vals, false, sw));
                    }
                }
            }
            Transform::Atanh => {
                if ys.iter().all(|&y| y.abs() < 1.0 - 1e-9)
                    && ys.iter().any(|&y| y.abs() > 0.2)
                {
                    let vals: Vec<f64> = ys.iter().map(|&y| y.atanh()).collect();
                    if vals.iter().all(|v| v.is_finite()) {
                        let sw = transform_sqrt_weights(t, ys);
                        out.push((t, vals, false, sw));
                    }
                }
            }
            _ => {
                if all_pos || all_neg {
                    if let Some(vals) = transform_target(t, &ys_abs) {
                        let sw = transform_sqrt_weights(t, &ys_abs);
                        out.push((t, vals, all_neg, sw));
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
    let cols: Vec<Vec<f64>> = terms.iter().map(|t| t.eval_rows(inputs)).collect();
    let m_pred: Vec<f64> = (0..ys.len())
        .map(|i| cols.iter().map(|c| c[i]).sum::<f64>())
        .collect();

    // Cancellation guard: a sum whose individual terms are vastly larger
    // than the sum itself is the signature of overfit junk (huge opposing
    // coefficients balanced against each other). Such solutions are also
    // numerically unstable off the training data, so they are discarded
    // outright instead of competing on RMSE.
    let pred_scale = rms(&m_pred).max(1e-300);
    let term_mass: f64 = cols.iter().map(|c| rms(c)).sum();
    if terms.len() > 1 && term_mass > 30.0 * pred_scale {
        return;
    }
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
    let positive = usable_variables(inputs);
    let real = real_variables(inputs);
    let targets = prepare_targets(ys);
    let y_std = {
        let mean = ys.iter().sum::<f64>() / ys.len() as f64;
        (ys.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / ys.len() as f64).sqrt()
    }
    .max(1e-30);
    let exact = |err: f64| err <= 1e-9 * y_std;

    // ---- Pass 1: greedy log-fit boosting + basic monomial dictionaries ----
    for (t, t_target, negate, sw) in &targets {
        if deadline.map_or(false, |dl| Instant::now() >= dl) {
            break;
        }
        let sw = sw.as_deref();
        let mut candidates: Vec<Vec<Term>> = Vec::new();
        if let Some(terms) =
            greedy_monomial_fit(inputs, t_target, &positive, config.max_boost_terms, sw)
        {
            candidates.push(terms);
        }
        candidates.extend(omp_monomial_fit(
            inputs,
            t_target,
            &positive,
            &real,
            config.max_boost_terms,
            deadline,
            false,
            sw,
        ));
        for terms in candidates {
            validate_and_assemble(*t, *negate, &terms, inputs, ys, reg, &mut fits);
        }
    }

    // ---- Pass 2: feature-augmented dictionary, only when still unsolved ----
    let best_so_far = fits.iter().map(|f| f.error).fold(f64::INFINITY, f64::min);
    if !exact(best_so_far) {
        for (t, t_target, negate, sw) in &targets {
            if deadline.map_or(false, |dl| Instant::now() >= dl) {
                break;
            }
            for terms in omp_monomial_fit(
                inputs,
                t_target,
                &positive,
                &real,
                config.max_boost_terms,
                deadline,
                true,
                sw.as_deref(),
            ) {
                validate_and_assemble(*t, *negate, &terms, inputs, ys, reg, &mut fits);
            }
        }
    }

    fits.sort_by(|a, b| a.error.partial_cmp(&b.error).unwrap());

    // Validation-ranked reorder with a one-standard-error band (P4-1): the
    // raw full-data error is a training error, so a junk multi-term fit can
    // edge out the true structure by chasing the noise. Candidates whose
    // held-out error is within one standard error of the best are ordered
    // simplest-first instead.
    let n = ys.len();
    let val_rows: Vec<usize> = (0..n).filter(|i| i % 5 == 4).collect();
    if val_rows.len() >= 10 && fits.len() > 1 {
        let val_inputs: Vec<Vec<f64>> = val_rows.iter().map(|&i| inputs[i].clone()).collect();
        let val_ys: Vec<f64> = val_rows.iter().map(|&i| ys[i]).collect();
        let vals: Vec<f64> = fits
            .iter()
            .map(|f| expression_rmse(&f.expression, &val_inputs, &val_ys, reg))
            .collect();
        let best_val = vals.iter().cloned().fold(f64::INFINITY, f64::min);
        if best_val.is_finite() {
            let eps = (1.0 / (2.0 * val_rows.len() as f64).sqrt()).clamp(0.01, 0.15);
            let band = best_val * (1.0 + eps);
            let mut idx: Vec<usize> = (0..fits.len()).collect();
            idx.sort_by(|&a, &b| {
                let in_a = vals[a] <= band;
                let in_b = vals[b] <= band;
                in_b.cmp(&in_a)
                    .then_with(|| {
                        if in_a && in_b {
                            fits[a]
                                .expression
                                .complexity()
                                .cmp(&fits[b].expression.complexity())
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    })
                    .then_with(|| {
                        vals[a]
                            .partial_cmp(&vals[b])
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
            });
            let mut reordered = Vec::with_capacity(fits.len());
            for i in idx {
                let f = &fits[i];
                reordered.push(PowerlawFit {
                    expression: f.expression.clone(),
                    error: f.error,
                });
            }
            fits = reordered;
        }
    }
    fits
}

/// Rational-function stage: fits y ~= P(x)/Q(x) by linearizing y*Q = P.
///
/// With a pivot term q0 (whose coefficient in Q is normalized to 1) the
/// model y*(q0 + sum q_k b_k) = sum p_j b_j is linear in {p_j, q_k}. We fold
/// y in as an extra input column so the whole fit becomes an ordinary
/// dictionary pursuit: P columns are basis terms, Q columns are basis terms
/// with exponent 1 on the y-column, and the target is y*q0. Trying the
/// constant and every single-variable term as pivot covers denominators
/// with and without a constant part (e.g. 1 + u*v/c^2 and m1 + m2).
pub fn run_rational(
    inputs: &[Vec<f64>],
    ys: &[f64],
    config: &SearchConfig,
    reg: &OperatorRegistry,
    deadline: Option<Instant>,
) -> Vec<PowerlawFit> {
    const INTEGER: &[f64] = &[-2.0, -1.0, 1.0, 2.0];
    let mut fits: Vec<PowerlawFit> = Vec::new();
    if inputs.len() < 20 || inputs[0].is_empty() {
        return fits;
    }
    let d = inputs[0].len();
    let real = real_variables(inputs);
    if real.is_empty() {
        return fits;
    }
    let basis = build_dictionary(d, &real, INTEGER, 3, 8_000);

    // Augmented rows: [x_0 .. x_{d-1}, y]. A Q column is a basis term with
    // exponent 1 on the y position.
    let aug: Vec<Vec<f64>> = inputs
        .iter()
        .zip(ys)
        .map(|(row, &y)| {
            let mut r = row.clone();
            r.push(y);
            r
        })
        .collect();

    // Pivot candidates: constant + every single-variable +1 term.
    let mut pivots: Vec<usize> = Vec::new();
    for (k, t) in basis.iter().enumerate() {
        let active: Vec<usize> = (0..d).filter(|&j| t.exponents[j] != 0.0).collect();
        if active.is_empty() || (active.len() == 1 && t.exponents[active[0]] == 1.0) {
            pivots.push(k);
        }
    }

    for &pivot in &pivots {
        if deadline.map_or(false, |dl| Instant::now() >= dl) {
            break;
        }
        let pivot_col = Monomial {
            coeff: 1.0,
            exponents: basis[pivot].exponents.clone(),
        }
        .eval_rows(inputs);
        if pivot_col.iter().any(|v| !v.is_finite()) {
            continue;
        }
        let target: Vec<f64> = ys.iter().zip(&pivot_col).map(|(y, p)| y * p).collect();

        // Dictionary over the augmented variables.
        let mut dictionary: Vec<Term> = Vec::with_capacity(basis.len() * 2);
        for (k, t) in basis.iter().enumerate() {
            let mut p_exps = t.exponents.clone();
            p_exps.push(0.0);
            dictionary.push(Term {
                coeff: 1.0,
                exponents: p_exps,
                feature: Feature::None,
            });
            if k != pivot {
                let mut q_exps = t.exponents.clone();
                q_exps.push(1.0);
                dictionary.push(Term {
                    coeff: 1.0,
                    exponents: q_exps,
                    feature: Feature::None,
                });
            }
        }

        // Denominator (Q) columns rank poorly in the initial correlation, so
        // force the strongest ones as alternative first picks.
        let q_starts: Vec<usize> = {
            let sub_n = target.len().min(250);
            let rows: Vec<usize> = (0..sub_n).map(|i| i * target.len() / sub_n).collect();
            let t_sub: Vec<f64> = rows.iter().map(|&i| target[i]).collect();
            let mut ranked: Vec<(usize, f64)> = dictionary
                .par_iter()
                .enumerate()
                .filter_map(|(idx, term)| {
                    if term.exponents[d] != 1.0 {
                        return None; // Q columns only
                    }
                    let mut col = Vec::with_capacity(rows.len());
                    for &i in &rows {
                        let mut acc = 1.0f64;
                        for (x, &e) in aug[i].iter().zip(&term.exponents) {
                            if e != 0.0 {
                                acc *= x.powf(e);
                            }
                        }
                        if !acc.is_finite() {
                            return None;
                        }
                        col.push(acc);
                    }
                    let norm = col.iter().map(|v| v * v).sum::<f64>().sqrt();
                    if norm < 1e-300 {
                        return None;
                    }
                    let dot: f64 = col.iter().zip(&t_sub).map(|(a, b)| a * b).sum();
                    let score = (dot / norm).abs();
                    if score.is_finite() {
                        Some((idx, score))
                    } else {
                        None
                    }
                })
                .collect();
            ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            // Small bases: try every denominator column as a forced first
            // pick (single-denominator-term rationals become exhaustive).
            let cap = if ranked.len() <= 160 { ranked.len() } else { 64 };
            ranked.into_iter().take(cap).map(|(i, _)| i).collect()
        };

        for terms in pursue_dictionary_with_starts(
            &aug,
            &target,
            &dictionary,
            config.max_boost_terms,
            deadline,
            &q_starts,
            // No transform is applied here (the linearized y*Q = P target is
            // in raw units); the remaining errors-in-variables bias from the
            // noisy y-columns is handled by the raw-space LM polish instead.
            None,
        ) {
            // Split into P (y-exponent 0) and Q (y-exponent 1) parts.
            let mut p_terms: Vec<Term> = Vec::new();
            let mut q_terms: Vec<Term> = Vec::new();
            let mut valid = true;
            for t in terms {
                let y_exp = t.exponents[d];
                let mut stripped = t.clone();
                stripped.exponents.truncate(d);
                if y_exp == 0.0 {
                    p_terms.push(stripped);
                } else if y_exp == 1.0 {
                    // y*q0 = P + c*(y*b)  =>  Q gains term -c*b.
                    stripped.coeff = -stripped.coeff;
                    q_terms.push(stripped);
                } else {
                    valid = false;
                    break;
                }
            }
            if !valid || p_terms.is_empty() {
                continue;
            }
            let mut q_all = vec![{
                let mut t = basis[pivot].clone();
                t.coeff = 1.0;
                t
            }];
            q_all.extend(q_terms);

            let p_expr = build::term_sum_expression(&p_terms, reg);
            let q_expr = build::term_sum_expression(&q_all, reg);
            let expr = build::binary("Divide", p_expr, q_expr, reg);
            let err = expression_rmse(&expr, inputs, ys, reg);
            if err.is_finite() {
                fits.push(PowerlawFit {
                    expression: expr,
                    error: err,
                });
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


#[cfg(test)]
mod v3_tests {
    use super::*;
    use crate::ops::registry::OperatorRegistry;

    fn lcg(seed: &mut u64) -> f64 {
        *seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (*seed >> 11) as f64 / (1u64 << 53) as f64
    }

    #[test]
    fn recovers_rational_function() {
        // y = (x0 + x1) / (1 + x0*x1)  (I.16.6 shape)
        let mut inputs = Vec::new();
        for i in 1..=25 {
            for j in 1..=25 {
                inputs.push(vec![0.2 + i as f64 * 0.15, 0.2 + j as f64 * 0.12]);
            }
        }
        let ys: Vec<f64> = inputs
            .iter()
            .map(|r| (r[0] + r[1]) / (1.0 + r[0] * r[1]))
            .collect();
        let reg = OperatorRegistry::with_builtins();
        let cfg = SearchConfig::fable_default();
        let fits = run_rational(&inputs, &ys, &cfg, &reg, None);
        assert!(!fits.is_empty());
        let scale = rms(&ys);
        assert!(
            fits[0].error < 1e-8 * scale,
            "error too large: {}",
            fits[0].error
        );
    }

    #[test]
    fn recovers_rational_without_constant_denominator() {
        // y = (a*r1 + b*r2) / (a + b)  (I.18.4 shape)
        let mut inputs = Vec::new();
        let mut seed = 999u64;
        let mut rand = || 1.0 + 4.0 * lcg(&mut seed);
        for _ in 0..600 {
            inputs.push(vec![rand(), rand(), rand(), rand()]);
        }
        let ys: Vec<f64> = inputs
            .iter()
            .map(|r| (r[0] * r[2] + r[1] * r[3]) / (r[0] + r[1]))
            .collect();
        let reg = OperatorRegistry::with_builtins();
        let cfg = SearchConfig::fable_default();
        let fits = run_rational(&inputs, &ys, &cfg, &reg, None);
        assert!(!fits.is_empty());
        let scale = rms(&ys);
        assert!(
            fits[0].error < 1e-8 * scale,
            "error too large: {}",
            fits[0].error
        );
    }

    #[test]
    fn recovers_monomial_with_negative_variables() {
        // y = 2*x0^2*x1 with x0 in (-3,3), x1 in (-2,2) — Feynman never has
        // negative variables; general data does.
        let mut inputs = Vec::new();
        for i in 0..30 {
            for j in 0..30 {
                inputs.push(vec![-3.0 + i as f64 * 0.21, -2.0 + j as f64 * 0.14]);
            }
        }
        let ys: Vec<f64> = inputs.iter().map(|r| 2.0 * r[0] * r[0] * r[1]).collect();
        let reg = OperatorRegistry::with_builtins();
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
    fn recovers_logistic_via_logit() {
        // y = 1/(1 + exp(x1 - 2*x0)) with x in (-2,2)
        let mut inputs = Vec::new();
        for i in 0..30 {
            for j in 0..30 {
                inputs.push(vec![-2.0 + i as f64 * 0.14, -2.0 + j as f64 * 0.14]);
            }
        }
        let ys: Vec<f64> = inputs
            .iter()
            .map(|r| 1.0 / (1.0 + (r[1] - 2.0 * r[0]).exp()))
            .collect();
        let reg = OperatorRegistry::with_builtins();
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
    fn noise_does_not_inflate_term_count() {
        // y = x0*x1 + 1% gaussian-ish noise: the recovered candidate must not
        // stack junk terms chasing the noise (validation gate).
        let mut inputs = Vec::new();
        let mut seed = 4242u64;
        for _ in 0..750 {
            let a = 1.0 + 4.0 * lcg(&mut seed);
            let b = 1.0 + 4.0 * lcg(&mut seed);
            inputs.push(vec![a, b]);
        }
        let clean: Vec<f64> = inputs.iter().map(|r| r[0] * r[1]).collect();
        let y_std = {
            let m = clean.iter().sum::<f64>() / clean.len() as f64;
            (clean.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / clean.len() as f64).sqrt()
        };
        let sigma = 0.01 * y_std;
        let ys: Vec<f64> = clean
            .iter()
            .enumerate()
            .map(|(i, &c)| {
                // Box-Muller-ish deterministic noise
                let u1 = lcg(&mut seed).max(1e-12);
                let u2 = lcg(&mut seed);
                let g = (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos();
                let _ = i;
                c + sigma * g
            })
            .collect();
        let reg = OperatorRegistry::with_builtins();
        let cfg = SearchConfig::fable_default();
        let fits = run_powerlaw(&inputs, &ys, &cfg, &reg, None);
        assert!(!fits.is_empty());
        // Fit error should be at the noise floor, not below (no overfit),
        // and the winning expression should stay small.
        let best = &fits[0];
        assert!(
            best.error < 1.3 * sigma,
            "error too large vs noise floor: {} vs sigma {}",
            best.error,
            sigma
        );
        assert!(
            best.expression.complexity() <= 12,
            "junk terms inflated the expression: complexity {}",
            best.expression.complexity()
        );
    }
}





#[cfg(test)]
mod v4_tests {
    use super::*;
    use crate::config::SearchConfig;
    use crate::engine::bfs;
    use crate::ops::registry::OperatorRegistry;

    fn lcg(seed: &mut u64) -> f64 {
        *seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (*seed >> 11) as f64 / (1u64 << 53) as f64
    }

    fn rand_rows(n: usize, d: usize, lo: f64, hi: f64, seed: u64) -> Vec<Vec<f64>> {
        let mut s = seed;
        (0..n)
            .map(|_| (0..d).map(|_| lo + (hi - lo) * lcg(&mut s)).collect())
            .collect()
    }

    fn best_pipeline_error(inputs: &[Vec<f64>], ys: &[f64]) -> f64 {
        // Full pipeline via public Searcher API.
        let mut cfg = SearchConfig::fable_default();
        cfg.max_complexity = 6;
        cfg.beam_width = 500;
        cfg.time_budget_s = 60.0;
        cfg.verbose = false;
        cfg.allow_approximate = true;
        let results = crate::engine::fable::run_fable(inputs, ys, &cfg).expect("search");
        results
            .iter()
            .map(|r| {
                let preds: Vec<f64> = inputs.iter().map(|row| {
                    let vals: Vec<crate::core::value::Value> =
                        row.iter().map(|&v| crate::core::value::real(v)).collect();
                    let reg = OperatorRegistry::with_builtins();
                    let _ = &reg;
                    r.eval_multi(row)
                }).collect();
                let _ = &preds;
                rmse(&preds, ys)
            })
            .fold(f64::INFINITY, f64::min)
    }

    #[test]
    fn recovers_abs_diff_times_z() {
        // y = |x0 - x1| * x2 — AbsDiff feature x monomial
        let inputs = rand_rows(750, 3, -3.0, 3.0, 77);
        let ys: Vec<f64> = inputs.iter().map(|r| (r[0] - r[1]).abs() * r[2]).collect();
        let err = best_pipeline_error(&inputs, &ys);
        let scale = rms(&ys);
        assert!(err < 1e-8 * scale, "error too large: {err}");
    }

    #[test]
    fn recovers_sigmoid_component() {
        // y = 2*sigmoid(x0) + x1 — Sigmoid feature + monomial
        let inputs = rand_rows(750, 2, -3.0, 3.0, 88);
        let ys: Vec<f64> = inputs
            .iter()
            .map(|r| 2.0 / (1.0 + (-r[0]).exp()) + r[1])
            .collect();
        let err = best_pipeline_error(&inputs, &ys);
        let scale = rms(&ys);
        assert!(err < 1e-8 * scale, "error too large: {err}");
    }

    #[test]
    fn recovers_min_via_beam() {
        // y = min(x0, x1) — 3-node beam structure
        let inputs = rand_rows(400, 2, -3.0, 3.0, 99);
        let ys: Vec<f64> = inputs.iter().map(|r| r[0].min(r[1])).collect();
        let mut cfg = SearchConfig::fable_default();
        cfg.max_complexity = 4;
        cfg.beam_width = 300;
        cfg.verbose = false;
        cfg.allow_approximate = true;
        let results = bfs::run_bfs(&inputs, &ys, &cfg).expect("search");
        let best = results
            .iter()
            .map(|r| {
                let preds: Vec<f64> = inputs.iter().map(|row| r.eval_multi(row)).collect();
                rmse(&preds, &ys)
            })
            .fold(f64::INFINITY, f64::min);
        let scale = rms(&ys);
        assert!(best < 1e-8 * scale, "error too large: {best}");
    }

    #[test]
    fn recovers_continuous_exponent() {
        // y = 2 * x^1.7 — continuous exponent via greedy raw-exponent fit
        let inputs = rand_rows(750, 1, 0.5, 5.0, 111);
        let ys: Vec<f64> = inputs.iter().map(|r| 2.0 * r[0].powf(1.7)).collect();
        let reg = OperatorRegistry::with_builtins();
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
    fn recovers_hill_equation() {
        // y = 3*x^2.5/(x^2.5 + 2^2.5) — Hill; InvY makes it 1/y = 1/3 + (2^2.5/3) x^{-2.5}
        let inputs = rand_rows(750, 1, 0.3, 6.0, 222);
        let k = 2.0f64.powf(2.5);
        let ys: Vec<f64> = inputs
            .iter()
            .map(|r| {
                let p = r[0].powf(2.5);
                3.0 * p / (p + k)
            })
            .collect();
        let reg = OperatorRegistry::with_builtins();
        let cfg = SearchConfig::fable_default();
        let fits = run_powerlaw(&inputs, &ys, &cfg, &reg, None);
        assert!(!fits.is_empty());
        let scale = rms(&ys);
        assert!(
            fits[0].error < 1e-7 * scale,
            "error too large: {}",
            fits[0].error
        );
    }
}

#[cfg(test)]
mod v5_tests {
    use super::*;
    use crate::config::SearchConfig;
    use crate::ops::registry::OperatorRegistry;

    fn lcg(seed: &mut u64) -> f64 {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((*seed >> 33) as f64) / (u64::MAX >> 33) as f64
    }

    fn gauss(seed: &mut u64) -> f64 {
        let u1 = lcg(seed).max(1e-12);
        let u2 = lcg(seed);
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }

    fn clean_rmse_on(
        expr: &crate::core::expression::Expression,
        inputs: &[Vec<f64>],
        clean: &[f64],
        reg: &OperatorRegistry,
    ) -> f64 {
        expression_rmse(expr, inputs, clean, reg)
    }

    /// P1-1: WLS in the InvY2 space keeps the Lorentz family recoverable
    /// under 1% gaussian noise. Without weights the 1/y^2 transform blows the
    /// noise up on the small-y rows and drags the fit off the true structure.
    #[test]
    fn wls_recovers_noisy_lorentz() {
        let mut seed = 99u64;
        let mut inputs = Vec::new();
        for _ in 0..750 {
            let x = -4.0 + 8.0 * lcg(&mut seed);
            inputs.push(vec![x]);
        }
        let clean: Vec<f64> = inputs.iter().map(|r| 1.0 / (1.0 + r[0] * r[0])).collect();
        let y_std = {
            let m = clean.iter().sum::<f64>() / clean.len() as f64;
            (clean.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / clean.len() as f64).sqrt()
        };
        let sigma = 0.01 * y_std;
        // Keep y positive so the InvY/InvY2 transforms stay in play.
        let ys: Vec<f64> = clean
            .iter()
            .map(|&c| (c + sigma * gauss(&mut seed)).max(1e-4))
            .collect();
        let reg = OperatorRegistry::with_builtins();
        let cfg = SearchConfig::fable_default();
        let fits = run_powerlaw(&inputs, &ys, &cfg, &reg, None);
        assert!(!fits.is_empty());
        let best_clean = fits
            .iter()
            .take(5)
            .map(|f| clean_rmse_on(&f.expression, &inputs, &clean, &reg))
            .fold(f64::INFINITY, f64::min);
        assert!(
            best_clean < 0.5 * sigma,
            "Lorentz not recovered under noise: clean rmse {} vs sigma {}",
            best_clean,
            sigma
        );
    }

    /// P1-3: under noise the continuous-exponent refinement must not drift
    /// off a clean integer exponent (validation-gated rounding preference).
    #[test]
    fn exponent_stays_rounded_under_noise() {
        let mut seed = 7u64;
        let mut inputs = Vec::new();
        for _ in 0..750 {
            inputs.push(vec![0.5 + 4.0 * lcg(&mut seed)]);
        }
        let clean: Vec<f64> = inputs.iter().map(|r| 3.0 * r[0] * r[0] + 0.5).collect();
        let y_std = {
            let m = clean.iter().sum::<f64>() / clean.len() as f64;
            (clean.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / clean.len() as f64).sqrt()
        };
        let sigma = 0.01 * y_std;
        let ys: Vec<f64> = clean.iter().map(|&c| c + sigma * gauss(&mut seed)).collect();
        let terms = greedy_monomial_fit(&inputs, &ys, &[0], 6, None).expect("greedy fit");
        let power_term = terms
            .iter()
            .find(|t| t.exponents.iter().any(|&e| e != 0.0))
            .expect("power term");
        assert_eq!(
            power_term.exponents[0], 2.0,
            "noise dragged the exponent off 2.0: {}",
            power_term.exponents[0]
        );
    }

    /// P4-2: a sum of huge mutually-cancelling terms must be rejected by the
    /// cancellation guard even when its residual looks fine on the samples.
    #[test]
    fn cancellation_guard_rejects_junk() {
        let mut seed = 21u64;
        let mut inputs = Vec::new();
        for _ in 0..200 {
            inputs.push(vec![1.0 + 2.0 * lcg(&mut seed), 1.0 + 2.0 * lcg(&mut seed)]);
        }
        let ys: Vec<f64> = inputs.iter().map(|r| r[0] + r[1]).collect();
        let reg = OperatorRegistry::with_builtins();

        // Junk: two gigantic opposing terms that cancel to something small.
        let junk = vec![
            Term { coeff: 1e7, exponents: vec![1.0, 0.0], feature: Feature::None },
            Term { coeff: -1e7, exponents: vec![1.0, 0.0], feature: Feature::None },
            Term { coeff: 1.0, exponents: vec![0.0, 1.0], feature: Feature::None },
        ];
        let mut fits: Vec<PowerlawFit> = Vec::new();
        validate_and_assemble(Transform::Id, false, &junk, &inputs, &ys, &reg, &mut fits);
        assert!(fits.is_empty(), "cancellation guard failed to reject junk");

        // Legitimate multi-term sums must pass untouched.
        let good = vec![
            Term { coeff: 1.0, exponents: vec![1.0, 0.0], feature: Feature::None },
            Term { coeff: 1.0, exponents: vec![0.0, 1.0], feature: Feature::None },
        ];
        let mut fits2: Vec<PowerlawFit> = Vec::new();
        validate_and_assemble(Transform::Id, false, &good, &inputs, &ys, &reg, &mut fits2);
        assert!(!fits2.is_empty(), "guard rejected a legitimate sum");
    }
}

#[cfg(test)]
mod v5_struct_tests {
    use super::*;
    use crate::config::SearchConfig;
    use crate::ops::registry::OperatorRegistry;

    fn lcg(seed: &mut u64) -> f64 {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((*seed >> 33) as f64) / (u64::MAX >> 33) as f64
    }

    fn gauss(seed: &mut u64) -> f64 {
        let u1 = lcg(seed).max(1e-12);
        let u2 = lcg(seed);
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }

    /// P2-1: z * sigmoid(x - y) is a single feature-dictionary term now.
    #[test]
    fn recovers_sigmoid_gate() {
        let mut seed = 11u64;
        let mut inputs = Vec::new();
        for _ in 0..750 {
            let x = -2.0 + 4.0 * lcg(&mut seed);
            let y = -2.0 + 4.0 * lcg(&mut seed);
            let z = -2.0 + 4.0 * lcg(&mut seed);
            inputs.push(vec![x, y, z]);
        }
        let ys: Vec<f64> = inputs
            .iter()
            .map(|r| r[2] / (1.0 + (r[1] - r[0]).exp()))
            .collect();
        let reg = OperatorRegistry::with_builtins();
        let cfg = SearchConfig::fable_default();
        let fits = run_powerlaw(&inputs, &ys, &cfg, &reg, None);
        assert!(!fits.is_empty());
        let best = fits
            .iter()
            .map(|f| f.error)
            .fold(f64::INFINITY, f64::min);
        assert!(best < 1e-8, "sigmoid gate not recovered: best error {}", best);
    }

    /// P2-2: y = sin^2(m) with a monomial argument is linearized by the
    /// asin(sqrt(y)) transform (argument within the principal branch).
    #[test]
    fn recovers_sin_squared() {
        let mut seed = 13u64;
        let mut inputs = Vec::new();
        for _ in 0..750 {
            // Keep the argument 0.5*x*y inside (0, pi/2).
            let x = 0.2 + 1.0 * lcg(&mut seed);
            let y = 0.2 + 1.0 * lcg(&mut seed);
            inputs.push(vec![x, y]);
        }
        let ys: Vec<f64> = inputs
            .iter()
            .map(|r| {
                let a = 0.5 * r[0] * r[1];
                let s = a.sin();
                s * s
            })
            .collect();
        let reg = OperatorRegistry::with_builtins();
        let cfg = SearchConfig::fable_default();
        let fits = run_powerlaw(&inputs, &ys, &cfg, &reg, None);
        assert!(!fits.is_empty());
        let best = fits
            .iter()
            .map(|f| f.error)
            .fold(f64::INFINITY, f64::min);
        assert!(best < 1e-8, "sin^2 not recovered: best error {}", best);
    }

    /// P2-3: y = tanh(m) is linearized by the atanh transform.
    #[test]
    fn recovers_tanh_of_monomial() {
        let mut seed = 17u64;
        let mut inputs = Vec::new();
        for _ in 0..750 {
            let x = 0.2 + 2.0 * lcg(&mut seed);
            let y = 0.2 + 2.0 * lcg(&mut seed);
            inputs.push(vec![x, y]);
        }
        let ys: Vec<f64> = inputs.iter().map(|r| (0.7 * r[0] / r[1]).tanh()).collect();
        let reg = OperatorRegistry::with_builtins();
        let cfg = SearchConfig::fable_default();
        let fits = run_powerlaw(&inputs, &ys, &cfg, &reg, None);
        assert!(!fits.is_empty());
        let best = fits
            .iter()
            .map(|f| f.error)
            .fold(f64::INFINITY, f64::min);
        assert!(best < 1e-8, "tanh law not recovered: best error {}", best);
    }

    /// Noise robustness of the sigmoid gate (WLS + validation machinery all
    /// active): 1% gaussian noise must not break the structural recovery.
    #[test]
    fn recovers_sigmoid_gate_under_noise() {
        let mut seed = 19u64;
        let mut inputs = Vec::new();
        for _ in 0..750 {
            let x = -2.0 + 4.0 * lcg(&mut seed);
            let y = -2.0 + 4.0 * lcg(&mut seed);
            let z = -2.0 + 4.0 * lcg(&mut seed);
            inputs.push(vec![x, y, z]);
        }
        let clean: Vec<f64> = inputs
            .iter()
            .map(|r| r[2] / (1.0 + (r[1] - r[0]).exp()))
            .collect();
        let y_std = {
            let m = clean.iter().sum::<f64>() / clean.len() as f64;
            (clean.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / clean.len() as f64).sqrt()
        };
        let sigma = 0.01 * y_std;
        let ys: Vec<f64> = clean.iter().map(|&c| c + sigma * gauss(&mut seed)).collect();
        let reg = OperatorRegistry::with_builtins();
        let cfg = SearchConfig::fable_default();
        let fits = run_powerlaw(&inputs, &ys, &cfg, &reg, None);
        assert!(!fits.is_empty());
        let best_clean = fits
            .iter()
            .take(5)
            .map(|f| expression_rmse(&f.expression, &inputs, &clean, &reg))
            .fold(f64::INFINITY, f64::min);
        assert!(
            best_clean < 0.5 * sigma,
            "noisy sigmoid gate not recovered: clean rmse {} vs sigma {}",
            best_clean,
            sigma
        );
    }
}

#[cfg(test)]
mod v5_robust_tests {
    use super::*;
    use crate::config::SearchConfig;
    use crate::ops::registry::OperatorRegistry;

    fn lcg(seed: &mut u64) -> f64 {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((*seed >> 33) as f64) / (u64::MAX >> 33) as f64
    }

    fn gauss(seed: &mut u64) -> f64 {
        let u1 = lcg(seed).max(1e-12);
        let u2 = lcg(seed);
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }

    /// P5-1: 1% gaussian noise plus 5% outliers at 10 sigma. The Huber-IRLS
    /// refit must keep the recovered coefficients near the truth instead of
    /// letting the outliers drag them.
    #[test]
    fn huber_refit_survives_outliers() {
        let mut seed = 23u64;
        let mut inputs = Vec::new();
        for _ in 0..750 {
            let a = 1.0 + 4.0 * lcg(&mut seed);
            let b = 1.0 + 4.0 * lcg(&mut seed);
            inputs.push(vec![a, b]);
        }
        let clean: Vec<f64> = inputs.iter().map(|r| 2.0 * r[0] + 3.0 * r[1]).collect();
        let y_std = {
            let m = clean.iter().sum::<f64>() / clean.len() as f64;
            (clean.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / clean.len() as f64).sqrt()
        };
        let sigma = 0.01 * y_std;
        let mut ys: Vec<f64> = clean.iter().map(|&c| c + sigma * gauss(&mut seed)).collect();
        // 5% outliers at +-10 sigma.
        let n_out = ys.len() / 20;
        for k in 0..n_out {
            let pos = (k * 997) % ys.len();
            let sign = if k % 2 == 0 { 1.0 } else { -1.0 };
            ys[pos] += sign * 10.0 * sigma;
        }
        let reg = OperatorRegistry::with_builtins();
        let cfg = SearchConfig::fable_default();
        let fits = run_powerlaw(&inputs, &ys, &cfg, &reg, None);
        assert!(!fits.is_empty());
        let best_clean = fits
            .iter()
            .take(5)
            .map(|f| expression_rmse(&f.expression, &inputs, &clean, &reg))
            .fold(f64::INFINITY, f64::min);
        assert!(
            best_clean < 0.5 * sigma,
            "outliers dragged the fit: clean rmse {} vs sigma {}",
            best_clean,
            sigma
        );
    }
}
