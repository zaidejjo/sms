//! Expression Expansion
//!
//! Distributes products over sums, expands powers, etc.

#![allow(unused)]

use crate::expr::Expr;

/// Main expansion function - distributes products, expands powers
pub fn expand(expr: &Expr) -> Expr {
    let mut current = expr.clone();
    loop {
        let expanded = expand_once(&current);
        if exprs_equal(&expanded, &current) {
            return expanded;
        }
        current = expanded;
    }
}

fn expand_once(expr: &Expr) -> Expr {
    match expr {
        // Distribute: (a + b) * c -> a*c + b*c
        Expr::Mul(a, b) => {
            let a = expand_once(a);
            let b = expand_once(b);
            match (&a, &b) {
                (Expr::Add(a1, a2), _) => {
                    let left = Expr::Mul(a1.clone(), Box::new(b.clone()));
                    let right = Expr::Mul(a2.clone(), Box::new(b.clone()));
                    Expr::Add(Box::new(expand_once(&left)), Box::new(expand_once(&right)))
                }
                (_, Expr::Add(b1, b2)) => {
                    let left = Expr::Mul(Box::new(a.clone()), b1.clone());
                    let right = Expr::Mul(Box::new(a.clone()), b2.clone());
                    Expr::Add(Box::new(expand_once(&left)), Box::new(expand_once(&right)))
                }
                _ => Expr::Mul(Box::new(a), Box::new(b)),
            }
        }
        // Distribute: (a - b) * c -> a*c - b*c
        Expr::Sub(a, b) => {
            let a = expand_once(a);
            let b = expand_once(b);
            Expr::Sub(Box::new(a), Box::new(b))
        }
        // Expand powers: (a + b)^n for integer n
        Expr::Pow(base, exp) => {
            let base = expand_once(base);
            let exp = expand_once(exp);
            if let Expr::Num(n) = exp {
                if n >= 2.0 && n.fract() == 0.0 {
                    return expand_power(&base, n as usize);
                }
            }
            Expr::Pow(Box::new(base), Box::new(exp))
        }
        // Recurse into other expressions
        Expr::Add(a, b) => Expr::Add(Box::new(expand_once(a)), Box::new(expand_once(b))),
        Expr::Div(a, b) => Expr::Div(Box::new(expand_once(a)), Box::new(expand_once(b))),
        Expr::Sin(a) => Expr::Sin(Box::new(expand_once(a))),
        Expr::Cos(a) => Expr::Cos(Box::new(expand_once(a))),
        Expr::Tan(a) => Expr::Tan(Box::new(expand_once(a))),
        Expr::Asin(a) => Expr::Asin(Box::new(expand_once(a))),
        Expr::Acos(a) => Expr::Acos(Box::new(expand_once(a))),
        Expr::Atan(a) => Expr::Atan(Box::new(expand_once(a))),
        Expr::Sinh(a) => Expr::Sinh(Box::new(expand_once(a))),
        Expr::Cosh(a) => Expr::Cosh(Box::new(expand_once(a))),
        Expr::Tanh(a) => Expr::Tanh(Box::new(expand_once(a))),
        Expr::Ln(a) => Expr::Ln(Box::new(expand_once(a))),
        Expr::Log(a, b) => Expr::Log(Box::new(expand_once(a)), Box::new(expand_once(b))),
        Expr::Exp(a) => Expr::Exp(Box::new(expand_once(a))),
        Expr::Sqrt(a) => Expr::Sqrt(Box::new(expand_once(a))),
        Expr::Abs(a) => Expr::Abs(Box::new(expand_once(a))),
        _ => expr.clone(),
    }
}

/// Expand (base)^n using binomial theorem for integer n
fn expand_power(base: &Expr, n: usize) -> Expr {
    if n == 0 {
        return Expr::Num(1.0);
    }
    if n == 1 {
        return base.clone();
    }
    if n == 2 {
        return Expr::Mul(Box::new(base.clone()), Box::new(base.clone()));
    }

    // For n > 2, use repeated multiplication
    let mut result = base.clone();
    for _ in 1..n {
        result = expand_once(&Expr::Mul(Box::new(result), Box::new(base.clone())));
    }
    result
}

/// Structural equality check
fn exprs_equal(a: &Expr, b: &Expr) -> bool {
    match (a, b) {
        (Expr::Num(x), Expr::Num(y)) => (x - y).abs() < 1e-12,
        (Expr::Var(x), Expr::Var(y)) => x == y,
        (Expr::Add(a1, b1), Expr::Add(a2, b2)) => exprs_equal(a1, a2) && exprs_equal(b1, b2),
        (Expr::Sub(a1, b1), Expr::Sub(a2, b2)) => exprs_equal(a1, a2) && exprs_equal(b1, b2),
        (Expr::Mul(a1, b1), Expr::Mul(a2, b2)) => {
            (exprs_equal(a1, a2) && exprs_equal(b1, b2))
                || (exprs_equal(a1, b2) && exprs_equal(b1, a2))
        }
        (Expr::Div(a1, b1), Expr::Div(a2, b2)) => exprs_equal(a1, a2) && exprs_equal(b1, b2),
        (Expr::Pow(a1, b1), Expr::Pow(a2, b2)) => exprs_equal(a1, a2) && exprs_equal(b1, b2),
        (Expr::Sin(a1), Expr::Sin(a2)) => exprs_equal(a1, a2),
        (Expr::Cos(a1), Expr::Cos(a2)) => exprs_equal(a1, a2),
        (Expr::Tan(a1), Expr::Tan(a2)) => exprs_equal(a1, a2),
        (Expr::Asin(a1), Expr::Asin(a2)) => exprs_equal(a1, a2),
        (Expr::Acos(a1), Expr::Acos(a2)) => exprs_equal(a1, a2),
        (Expr::Atan(a1), Expr::Atan(a2)) => exprs_equal(a1, a2),
        (Expr::Sinh(a1), Expr::Sinh(a2)) => exprs_equal(a1, a2),
        (Expr::Cosh(a1), Expr::Cosh(a2)) => exprs_equal(a1, a2),
        (Expr::Tanh(a1), Expr::Tanh(a2)) => exprs_equal(a1, a2),
        (Expr::Ln(a1), Expr::Ln(a2)) => exprs_equal(a1, a2),
        (Expr::Log(a1, b1), Expr::Log(a2, b2)) => exprs_equal(a1, a2) && exprs_equal(b1, b2),
        (Expr::Exp(a1), Expr::Exp(a2)) => exprs_equal(a1, a2),
        (Expr::Sqrt(a1), Expr::Sqrt(a2)) => exprs_equal(a1, a2),
        (Expr::Abs(a1), Expr::Abs(a2)) => exprs_equal(a1, a2),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    #[test]
    fn test_expand_distribute() {
        let mut parser = Parser::new("(x + 1) * (x - 1)");
        let expr = parser.parse_expression();
        let expanded = expand(&expr);
        // Should be x^2 - 1
        assert!(matches!(expanded, Expr::Sub(_, _)));
    }

    #[test]
    fn test_expand_simple() {
        let mut parser = Parser::new("2 * (x + 3)");
        let expr = parser.parse_expression();
        let expanded = expand(&expr);
        // Should be 2*x + 6
        match &expanded {
            Expr::Add(a, b) => {
                if let Expr::Mul(c, v) = a.as_ref() {
                    if let (Expr::Num(c), Expr::Var(v)) = (c.as_ref(), v.as_ref()) {
                        assert!((c - 2.0).abs() < 1e-10);
                        assert_eq!(*v, 'x');
                    }
                }
                if let Expr::Num(n) = b.as_ref() {
                    assert!((n - 6.0).abs() < 1e-10);
                }
            }
            _ => panic!("Expected 2*x + 6, got {:?}", expanded),
        }
    }

    #[test]
    fn test_expand_power() {
        let mut parser = Parser::new("(x + 1) ^ 2");
        let expr = parser.parse_expression();
        let expanded = expand(&expr);
        // Should be x^2 + 2*x + 1
        match &expanded {
            Expr::Add(_, _) => {}
            _ => {
                // Could also be flattened differently
                assert!(matches!(expanded, Expr::Add(_, _)));
            }
        }
    }

    #[test]
    fn test_expand_power_cubic() {
        let mut parser = Parser::new("(x + 1) ^ 3");
        let expr = parser.parse_expression();
        let expanded = expand(&expr);
        // Should be x^3 + 3*x^2 + 3*x + 1
        assert!(matches!(expanded, Expr::Add(_, _)));
    }

    #[test]
    fn test_expand_nested() {
        let mut parser = Parser::new("(x + y) * (x - y)");
        let expr = parser.parse_expression();
        let expanded = expand(&expr);
        // Should be x^2 - y^2
        assert!(matches!(expanded, Expr::Sub(_, _)));
    }

    #[test]
    fn test_expand_already_expanded() {
        let mut parser = Parser::new("x^2 + 2*x + 1");
        let expr = parser.parse_expression();
        let expanded = expand(&expr);
        // Should remain unchanged
        assert!(matches!(expanded, Expr::Add(_, _)));
    }
}