//! Collect Like Terms
//!
//! Combines terms with the same variable and power: x^2 + 2x + x^2 -> 2x^2 + 2x

use crate::expr::Expr;
use std::collections::HashMap;

/// Collect like terms with respect to a variable
pub fn collect(expr: &Expr, var: char) -> Expr {
    let terms = flatten_add(expr);
    let mut coeffs: HashMap<i32, Expr> = HashMap::new(); // power -> coefficient

    for term in terms {
        let (coeff, power) = extract_coeff_and_power(&term, var);
        coeffs
            .entry(power)
            .and_modify(|c| *c = add_exprs(c, &coeff))
            .or_insert(coeff);
    }

    // Rebuild expression from highest to lowest power
    let mut powers: Vec<i32> = coeffs.keys().cloned().collect();
    powers.sort_by(|a, b| b.cmp(a)); // descending

    if powers.is_empty() {
        return Expr::Num(0.0);
    }

    let mut result = term_from_coeff_power(&coeffs[&powers[0]], powers[0], var);
    for &power in &powers[1..] {
        let term = term_from_coeff_power(&coeffs[&power], power, var);
        result = Expr::Add(Box::new(result), Box::new(term));
    }

    result
}

/// Flatten nested additions into a list of terms
fn flatten_add(expr: &Expr) -> Vec<Expr> {
    match expr {
        Expr::Add(a, b) => {
            let mut terms = flatten_add(a);
            terms.extend(flatten_add(b));
            terms
        }
        Expr::Sub(a, b) => {
            let mut terms = flatten_add(a);
            // Subtract b: add (-1 * b)
            terms.push(Expr::Mul(Box::new(Expr::Num(-1.0)), b.clone()));
            terms
        }
        _ => vec![expr.clone()],
    }
}

/// Extract coefficient and power of var from a term
/// Returns (coefficient, power)
fn extract_coeff_and_power(term: &Expr, var: char) -> (Expr, i32) {
    match term {
        Expr::Var(v) if *v == var => (Expr::Num(1.0), 1),
        Expr::Num(_) => (term.clone(), 0),
        Expr::Mul(a, b) => {
            // Check if one factor is var^power
            let (coeff_a, pow_a) = extract_coeff_and_power(a, var);
            let (coeff_b, pow_b) = extract_coeff_and_power(b, var);
            (mul_exprs(&coeff_a, &coeff_b), pow_a + pow_b)
        }
        Expr::Pow(base, exp) => {
            if let Expr::Var(v) = &**base {
                if *v == var {
                    if let Expr::Num(n) = &**exp {
                        return (Expr::Num(1.0), *n as i32);
                    }
                }
            }
            (term.clone(), 0)
        }
        _ => (term.clone(), 0),
    }
}

/// Create term from coefficient and power
fn term_from_coeff_power(coeff: &Expr, power: i32, var: char) -> Expr {
    if power == 0 {
        return coeff.clone();
    }
    let var_pow = if power == 1 {
        Expr::Var(var)
    } else {
        Expr::Pow(Box::new(Expr::Var(var)), Box::new(Expr::Num(power as f64)))
    };

    match coeff {
        Expr::Num(c) if *c == 1.0 => var_pow,
        Expr::Num(c) if *c == -1.0 => Expr::Mul(Box::new(Expr::Num(-1.0)), Box::new(var_pow)),
        _ => Expr::Mul(Box::new(coeff.clone()), Box::new(var_pow)),
    }
}

/// Add two coefficient expressions
fn add_exprs(a: &Expr, b: &Expr) -> Expr {
    match (a, b) {
        (Expr::Num(x), Expr::Num(y)) => Expr::Num(x + y),
        (Expr::Num(x), _) if *x == 0.0 => b.clone(),
        (_, Expr::Num(y)) if *y == 0.0 => a.clone(),
        _ => Expr::Add(Box::new(a.clone()), Box::new(b.clone())),
    }
}

/// Multiply two coefficient expressions
fn mul_exprs(a: &Expr, b: &Expr) -> Expr {
    match (a, b) {
        (Expr::Num(x), Expr::Num(y)) => Expr::Num(x * y),
        (Expr::Num(x), _) if *x == 0.0 => Expr::Num(0.0),
        (_, Expr::Num(y)) if *y == 0.0 => Expr::Num(0.0),
        (Expr::Num(x), _) if *x == 1.0 => b.clone(),
        (_, Expr::Num(y)) if *y == 1.0 => a.clone(),
        _ => Expr::Mul(Box::new(a.clone()), Box::new(b.clone())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    #[test]
    fn test_collect_basic() {
        let mut parser = Parser::new("x^2 + x^2");
        let expr = parser.parse_expression();
        let collected = collect(&expr, 'x');
        // Should be 2*x^2
        match &collected {
            Expr::Mul(c, _) => {
                if let Expr::Num(c) = c.as_ref() {
                    assert!((c - 2.0).abs() < 1e-10);
                }
            }
            _ => panic!("Expected 2*x^2"),
        }
    }

    #[test]
    fn test_collect_mixed() {
        let mut parser = Parser::new("x^2 + 2*x + x^2");
        let expr = parser.parse_expression();
        let collected = collect(&expr, 'x');
        // Should be 2*x^2 + 2*x
        match &collected {
            Expr::Add(a, b) => {
                if let Expr::Mul(c1, p1) = a.as_ref() {
                    if let Expr::Num(c1) = c1.as_ref() {
                        assert!((c1 - 2.0).abs() < 1e-10);
                    }
                    if let Expr::Pow(_, p1) = p1.as_ref() {
                        if let Expr::Num(p1) = p1.as_ref() {
                            assert!((p1 - 2.0).abs() < 1e-10);
                        }
                    }
                }
                if let Expr::Mul(c2, _) = b.as_ref() {
                    if let Expr::Num(c2) = c2.as_ref() {
                        assert!((c2 - 2.0).abs() < 1e-10);
                    }
                }
            }
            _ => panic!("Expected 2*x^2 + 2*x, got {:?}", collected),
        }
    }

    #[test]
    fn test_collect_constants() {
        let mut parser = Parser::new("3 + 5 + x");
        let expr = parser.parse_expression();
        let collected = collect(&expr, 'x');
        // Should be 8 + x
        match &collected {
            Expr::Add(c, v) => {
                if let Expr::Num(c) = c.as_ref() {
                    assert!((c - 8.0).abs() < 1e-10);
                }
                assert!(matches!(v.as_ref(), Expr::Var(_)));
            }
            _ => panic!("Expected 8 + x"),
        }
    }

    #[test]
    fn test_collect_subtraction() {
        let mut parser = Parser::new("x^2 - x^2 + 3*x");
        let expr = parser.parse_expression();
        let collected = collect(&expr, 'x');
        // Should be 3*x
        match &collected {
            Expr::Mul(c, v) => {
                if let Expr::Num(c) = c.as_ref() {
                    assert!((c - 3.0).abs() < 1e-10);
                }
                assert!(matches!(v.as_ref(), Expr::Var(_)));
            }
            _ => panic!("Expected 3*x"),
        }
    }

    #[test]
    fn test_collect_different_vars() {
        let mut parser = Parser::new("x^2 + y^2 + x^2");
        let expr = parser.parse_expression();
        let collected = collect(&expr, 'x');
        // Should be 2*x^2 + y^2
        match &collected {
            Expr::Add(a, _) => {
                if let Expr::Mul(c, _) = a.as_ref() {
                    if let Expr::Num(c) = c.as_ref() {
                        assert!((c - 2.0).abs() < 1e-10);
                    }
                }
            }
            _ => panic!("Expected 2*x^2 + y^2"),
        }
    }

    #[test]
    fn test_collect_only_constants() {
        let mut parser = Parser::new("5 + 3");
        let expr = parser.parse_expression();
        let collected = collect(&expr, 'x');
        assert_eq!(collected, Expr::Num(8.0));
    }
}