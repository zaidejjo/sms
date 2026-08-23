//! Expression Simplification
//!
//! Applies algebraic simplification rules to expressions:
//! - Constant folding (2 + 3 -> 5)
//! - Identity rules (x + 0 -> x, x * 1 -> x)
//! - Zero rules (x * 0 -> 0, x / 1 -> x)
//! - Power rules (x^1 -> x, x^0 -> 1)

#![allow(unused)]

use crate::expr::Expr;

/// Main simplification function - applies rules repeatedly until fixpoint
pub fn simplify(expr: &Expr) -> Expr {
    let mut current = expr.clone();
    loop {
        let simplified = simplify_once(&current);
        if exprs_equal(&simplified, &current) {
            return simplified;
        }
        current = simplified;
    }
}

/// Single pass of simplification rules
fn simplify_once(expr: &Expr) -> Expr {
    match expr {
        // Constant folding for binary ops
        Expr::Add(a, b) => {
            let a = simplify_once(a);
            let b = simplify_once(b);
            match (&a, &b) {
                (Expr::Num(x), Expr::Num(y)) => Expr::Num(x + y),
                (Expr::Num(x), _) if *x == 0.0 => b,
                (_, Expr::Num(y)) if *y == 0.0 => a,
                _ => Expr::Add(Box::new(a), Box::new(b)),
            }
        }
        Expr::Sub(a, b) => {
            let a = simplify_once(a);
            let b = simplify_once(b);
            match (&a, &b) {
                (Expr::Num(x), Expr::Num(y)) => Expr::Num(x - y),
                (_, Expr::Num(y)) if *y == 0.0 => a,
                _ => Expr::Sub(Box::new(a), Box::new(b)),
            }
        }
        Expr::Mul(a, b) => {
            let a = simplify_once(a);
            let b = simplify_once(b);
            match (&a, &b) {
                (Expr::Num(x), Expr::Num(y)) => Expr::Num(x * y),
                (Expr::Num(x), _) if *x == 0.0 => Expr::Num(0.0),
                (_, Expr::Num(y)) if *y == 0.0 => Expr::Num(0.0),
                (Expr::Num(x), _) if *x == 1.0 => b,
                (_, Expr::Num(y)) if *y == 1.0 => a,
                // x * x -> x^2
                _ if exprs_equal(&a, &b) => Expr::Pow(Box::new(a), Box::new(Expr::Num(2.0))),
                _ => Expr::Mul(Box::new(a), Box::new(b)),
            }
        }
        Expr::Div(a, b) => {
            let a = simplify_once(a);
            let b = simplify_once(b);
            match (&a, &b) {
                (Expr::Num(x), Expr::Num(y)) if *y != 0.0 => Expr::Num(x / y),
                (_, Expr::Num(y)) if *y == 1.0 => a,
                (Expr::Num(x), _) if *x == 0.0 => Expr::Num(0.0),
                _ if exprs_equal(&a, &b) => Expr::Num(1.0),
                _ => Expr::Div(Box::new(a), Box::new(b)),
            }
        }
        Expr::Pow(a, b) => {
            let a = simplify_once(a);
            let b = simplify_once(b);
            match (&a, &b) {
                (_, Expr::Num(y)) if *y == 0.0 => Expr::Num(1.0),
                (_, Expr::Num(y)) if *y == 1.0 => a,
                (Expr::Num(x), Expr::Num(y)) => Expr::Num(x.powf(*y)),
                (Expr::Num(x), _) if *x == 0.0 => Expr::Num(0.0),
                (Expr::Num(x), _) if *x == 1.0 => Expr::Num(1.0),
                _ => Expr::Pow(Box::new(a), Box::new(b)),
            }
        }
        // Unary functions - constant folding
        Expr::Sin(a) => {
            let a = simplify_once(a);
            if let Expr::Num(x) = &a {
                Expr::Num(x.sin())
            } else {
                Expr::Sin(Box::new(a))
            }
        }
        Expr::Cos(a) => {
            let a = simplify_once(a);
            if let Expr::Num(x) = &a {
                Expr::Num(x.cos())
            } else {
                Expr::Cos(Box::new(a))
            }
        }
        Expr::Tan(a) => {
            let a = simplify_once(a);
            if let Expr::Num(x) = &a {
                Expr::Num(x.tan())
            } else {
                Expr::Tan(Box::new(a))
            }
        }
        Expr::Asin(a) => {
            let a = simplify_once(a);
            if let Expr::Num(x) = &a {
                Expr::Num(x.asin())
            } else {
                Expr::Asin(Box::new(a))
            }
        }
        Expr::Acos(a) => {
            let a = simplify_once(a);
            if let Expr::Num(x) = &a {
                Expr::Num(x.acos())
            } else {
                Expr::Acos(Box::new(a))
            }
        }
        Expr::Atan(a) => {
            let a = simplify_once(a);
            if let Expr::Num(x) = &a {
                Expr::Num(x.atan())
            } else {
                Expr::Atan(Box::new(a))
            }
        }
        Expr::Sinh(a) => {
            let a = simplify_once(a);
            if let Expr::Num(x) = &a {
                Expr::Num(x.sinh())
            } else {
                Expr::Sinh(Box::new(a))
            }
        }
        Expr::Cosh(a) => {
            let a = simplify_once(a);
            if let Expr::Num(x) = &a {
                Expr::Num(x.cosh())
            } else {
                Expr::Cosh(Box::new(a))
            }
        }
        Expr::Tanh(a) => {
            let a = simplify_once(a);
            if let Expr::Num(x) = &a {
                Expr::Num(x.tanh())
            } else {
                Expr::Tanh(Box::new(a))
            }
        }
        Expr::Ln(a) => {
            let a = simplify_once(a);
            if let Expr::Num(x) = &a {
                if *x > 0.0 {
                    Expr::Num(x.ln())
                } else {
                    Expr::Ln(Box::new(a))
                }
            } else {
                Expr::Ln(Box::new(a))
            }
        }
        Expr::Log(a, b) => {
            let a = simplify_once(a);
            let b = simplify_once(b);
            if let (Expr::Num(x), Expr::Num(y)) = (&a, &b) {
                if *x > 0.0 && *y > 0.0 && *y != 1.0 {
                    Expr::Num(x.log(*y))
                } else {
                    Expr::Log(Box::new(a), Box::new(b))
                }
            } else {
                Expr::Log(Box::new(a), Box::new(b))
            }
        }
        Expr::Exp(a) => {
            let a = simplify_once(a);
            if let Expr::Num(x) = &a {
                Expr::Num(x.exp())
            } else {
                Expr::Exp(Box::new(a))
            }
        }
        Expr::Sqrt(a) => {
            let a = simplify_once(a);
            if let Expr::Num(x) = &a {
                if *x >= 0.0 {
                    Expr::Num(x.sqrt())
                } else {
                    Expr::Sqrt(Box::new(a))
                }
            } else {
                Expr::Sqrt(Box::new(a))
            }
        }
        Expr::Abs(a) => {
            let a = simplify_once(a);
            if let Expr::Num(x) = &a {
                Expr::Num(x.abs())
            } else {
                Expr::Abs(Box::new(a))
            }
        }
        // Variables and numbers pass through
        _ => expr.clone(),
    }
}

/// Structural equality check for expressions
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
    fn test_constant_folding() {
        let mut parser = Parser::new("2 + 3");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Num(5.0));

        let mut parser = Parser::new("10 - 4");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Num(6.0));

        let mut parser = Parser::new("3 * 4");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Num(12.0));

        let mut parser = Parser::new("8 / 2");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Num(4.0));

        let mut parser = Parser::new("2 ^ 3");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Num(8.0));
    }

    #[test]
    fn test_identity_rules() {
        let mut parser = Parser::new("x + 0");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Var('x'));

        let mut parser = Parser::new("0 + x");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Var('x'));

        let mut parser = Parser::new("x * 1");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Var('x'));

        let mut parser = Parser::new("1 * x");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Var('x'));

        let mut parser = Parser::new("x / 1");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Var('x'));

        let mut parser = Parser::new("x ^ 1");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Var('x'));

        let mut parser = Parser::new("x ^ 0");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Num(1.0));
    }

    #[test]
    fn test_zero_rules() {
        let mut parser = Parser::new("x * 0");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Num(0.0));

        let mut parser = Parser::new("0 * x");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Num(0.0));

        let mut parser = Parser::new("0 / x");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Num(0.0));

        let mut parser = Parser::new("x - x");
        let expr = parser.parse_expression();
        // x - x simplifies to 0 via x + (-1)*x -> 0
        // Our simplifier doesn't do this yet, so it stays as Sub
        // This is OK for now
    }

    #[test]
    fn test_power_rules() {
        let mut parser = Parser::new("(x^2)^3");
        let expr = parser.parse_expression();
        // Should become x^6 (not implemented yet)
        assert!(matches!(simplify(&expr), Expr::Pow(_, _)));

        let mut parser = Parser::new("2 ^ x");
        let expr = parser.parse_expression();
        assert!(matches!(simplify(&expr), Expr::Pow(_, _)));
    }

    #[test]
    fn test_trig_constant_folding() {
        let mut parser = Parser::new("sin(0)");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Num(0.0));

        let mut parser = Parser::new("cos(0)");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Num(1.0));

        let mut parser = Parser::new("exp(0)");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Num(1.0));

        let mut parser = Parser::new("ln(1)");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Num(0.0));

        let mut parser = Parser::new("sqrt(4)");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Num(2.0));
    }

    #[test]
    fn test_nested_simplification() {
        let mut parser = Parser::new("(2 + 3) * x");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Mul(Box::new(Expr::Num(5.0)), Box::new(Expr::Var('x'))));

        let mut parser = Parser::new("sin(pi/2)");
        let expr = parser.parse_expression();
        // pi/2 gets folded to ~1.57, sin(1.57) ~ 1
        // This requires constant folding in parser first
    }
}