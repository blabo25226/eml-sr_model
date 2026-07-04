//! Programmatic expression construction helpers (fable edition).
//!
//! Stage A (power-law solver) and the affine-scaling wrapper need to assemble
//! expression trees outside the BFS enumeration loop. These helpers build RPN
//! node vectors together with a canonical display string, reusing the exact
//! same operators registered in the [`OperatorRegistry`].

use crate::core::expression::{Expression, Node};
use crate::ops::registry::OperatorRegistry;

/// Formats a float with the shortest round-trip representation.
pub fn fmt_num(v: f64) -> String {
    format!("{v}")
}

/// A literal numeric constant expression.
pub fn num(v: f64) -> Expression {
    Expression::new(vec![Node::Num(v)], 0, 0, fmt_num(v))
}

/// A variable reference expression `v_{i}`.
pub fn var(i: usize) -> Expression {
    Expression::variable(i as u8, format!("v_{{{i}}}"))
}

/// Applies a unary operator by name (panics if the operator is unknown).
pub fn unary(name: &str, child: Expression, reg: &OperatorRegistry) -> Expression {
    let op_id = reg
        .id_by_name(name)
        .unwrap_or_else(|| panic!("unknown unary operator: {name}"));
    let display = format!("{}({})", name, child.display());
    let mut nodes = child.nodes.clone();
    nodes.push(Node::Op { op_id, arity: 1 });
    Expression::new(nodes, child.var_count(), child.param_count(), display)
}

/// Applies a binary operator by name, offsetting right-side param ids.
pub fn binary(name: &str, left: Expression, right: Expression, reg: &OperatorRegistry) -> Expression {
    let op_id = reg
        .id_by_name(name)
        .unwrap_or_else(|| panic!("unknown binary operator: {name}"));
    let display = format!("{}({}, {})", name, left.display(), right.display());
    let mut nodes = left.nodes.clone();
    let mut right_nodes = right.nodes.clone();
    for node in &mut right_nodes {
        if let Node::Param { id, .. } = node {
            *id += left.param_count();
        }
    }
    nodes.extend_from_slice(&right_nodes);
    nodes.push(Node::Op { op_id, arity: 2 });
    Expression::new(
        nodes,
        left.var_count().max(right.var_count()),
        left.param_count() + right.param_count(),
        display,
    )
}

/// Builds `base^exponent` economically from Square/Cube/Sqrt/Inv/Pow.
///
/// The exponent must be non-zero. Negative exponents wrap the positive power
/// in `Inv`.
pub fn power(base: Expression, exponent: f64, reg: &OperatorRegistry) -> Expression {
    if exponent < 0.0 {
        return unary("Inv", power(base, -exponent, reg), reg);
    }
    // Positive exponents.
    let eps = 1e-12;
    let close = |a: f64, b: f64| (a - b).abs() < eps;
    if close(exponent, 1.0) {
        base
    } else if close(exponent, 2.0) {
        unary("Square", base, reg)
    } else if close(exponent, 3.0) {
        unary("Cube", base, reg)
    } else if close(exponent, 4.0) {
        unary("Square", unary("Square", base, reg), reg)
    } else if close(exponent, 0.5) {
        unary("Sqrt", base, reg)
    } else if close(exponent, 1.5) {
        unary("Sqrt", unary("Cube", base, reg), reg)
    } else {
        binary("Pow", base, num(exponent), reg)
    }
}

/// One multiplicative term `coeff * prod_i v_i^{exponents[i]}`.
#[derive(Clone, Debug)]
pub struct Monomial {
    pub coeff: f64,
    /// One exponent per input variable (0.0 = variable unused).
    pub exponents: Vec<f64>,
}

impl Monomial {
    /// Evaluates the monomial on a full data matrix (rows = samples).
    pub fn eval_rows(&self, inputs: &[Vec<f64>]) -> Vec<f64> {
        inputs
            .iter()
            .map(|row| {
                let mut acc = self.coeff;
                for (x, &e) in row.iter().zip(&self.exponents) {
                    if e != 0.0 {
                        acc *= x.powf(e);
                    }
                }
                acc
            })
            .collect()
    }

    /// Builds the expression tree for this monomial.
    pub fn to_expression(&self, reg: &OperatorRegistry) -> Expression {
        let mut numer: Option<Expression> = None;
        let mut denom: Option<Expression> = None;
        for (i, &e) in self.exponents.iter().enumerate() {
            if e == 0.0 {
                continue;
            }
            let factor = power(var(i), e.abs(), reg);
            let slot = if e > 0.0 { &mut numer } else { &mut denom };
            *slot = Some(match slot.take() {
                None => factor,
                Some(prev) => binary("Times", prev, factor, reg),
            });
        }

        let coeff_is_one = (self.coeff - 1.0).abs() < 1e-15;
        let coeff_is_neg_one = (self.coeff + 1.0).abs() < 1e-15;

        let mut expr = match (numer, denom) {
            (Some(n), Some(d)) => binary("Divide", n, d, reg),
            (Some(n), None) => n,
            (None, Some(d)) => unary("Inv", d, reg),
            (None, None) => return num(self.coeff),
        };

        if coeff_is_neg_one {
            expr = unary("Neg", expr, reg);
        } else if !coeff_is_one {
            expr = binary("Times", num(self.coeff), expr, reg);
        }
        expr
    }
}

/// Sums a list of monomials into one expression tree.
pub fn monomial_sum_expression(terms: &[Monomial], reg: &OperatorRegistry) -> Expression {
    assert!(!terms.is_empty());
    let mut expr: Option<Expression> = None;
    for term in terms {
        let term_expr = term.to_expression(reg);
        expr = Some(match expr.take() {
            None => term_expr,
            Some(prev) => binary("Plus", prev, term_expr, reg),
        });
    }
    expr.unwrap()
}

/// Wraps a fitted structure with the affine map `a*f + b`, omitting trivial
/// parts (`a ≈ 1`, `b ≈ 0` relative to `scale`).
pub fn affine_wrap(
    f: Expression,
    a: f64,
    b: f64,
    scale: f64,
    reg: &OperatorRegistry,
) -> Expression {
    let scale = scale.abs().max(1e-300);
    let mut expr = f;
    if (a - 1.0).abs() > 1e-12 {
        expr = if (a + 1.0).abs() <= 1e-12 {
            unary("Neg", expr, reg)
        } else {
            binary("Times", num(a), expr, reg)
        };
    }
    if b.abs() > 1e-10 * scale {
        expr = binary("Plus", expr, num(b), reg);
    }
    expr
}
