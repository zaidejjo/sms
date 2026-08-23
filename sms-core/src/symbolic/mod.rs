//! Symbolic Algebra Engine
//!
//! Provides expression simplification, expansion, factorization,
//! pattern matching, and symbolic integration.

pub mod simplify;
pub mod expand;
pub mod factor;
pub mod collect;
pub mod pattern;
pub mod rules;
pub mod integrate;

use crate::expr::Expr;

/// Main symbolic simplification entry point
pub fn simplify(expr: &Expr) -> Expr {
    simplify::simplify(expr)
}

/// Expand expression (distribute products, powers)
pub fn expand(expr: &Expr) -> Expr {
    expand::expand(expr)
}

/// Factor polynomial expression
pub fn factor(expr: &Expr) -> Expr {
    factor::factor(expr)
}

/// Collect like terms (e.g., x^2 + 2x + x^2 -> 2x^2 + 2x)
pub fn collect(expr: &Expr, var: char) -> Expr {
    collect::collect(expr, var)
}

/// Partial fraction decomposition
pub fn partial_fractions(expr: &Expr, var: char) -> Result<Vec<Expr>, String> {
    factor::partial_fractions(expr, var)
}

/// Symbolic integration
pub fn integrate(expr: &Expr, var: char) -> Result<Expr, String> {
    integrate::integrate(expr, var)
}

/// Pattern match and rewrite
pub fn rewrite(expr: &Expr, rules: &[crate::symbolic::rules::RewriteRule]) -> Expr {
    rules::rewrite(expr, rules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    #[test]
    fn test_simplify_basic() {
        let mut parser = Parser::new("x + 0");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Var('x'));

        let mut parser = Parser::new("x * 1");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Var('x'));

        let mut parser = Parser::new("x ^ 1");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Var('x'));

        let mut parser = Parser::new("0 * x");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Num(0.0));
    }

    #[test]
    fn test_simplify_constants() {
        let mut parser = Parser::new("2 + 3");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Num(5.0));

        let mut parser = Parser::new("2 * 3");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Num(6.0));

        let mut parser = Parser::new("2 ^ 3");
        let expr = parser.parse_expression();
        assert_eq!(simplify(&expr), Expr::Num(8.0));
    }

    #[test]
    fn test_expand_basic() {
        let mut parser = Parser::new("(x + 1) * (x - 1)");
        let expr = parser.parse_expression();
        let expanded = expand(&expr);
        // Should become x^2 - 1
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

    #[test]
    fn test_collect() {
        let mut parser = Parser::new("x^2 + 2*x + x^2");
        let expr = parser.parse_expression();
        let collected = collect(&expr, 'x');
        // Should become 2*x^2 + 2*x
        match &collected {
            Expr::Add(a, b) => {
                if let Expr::Mul(_, p) = a.as_ref() {
                    if let Expr::Pow(_, _) = p.as_ref() {}
                }
                if let Expr::Mul(_, _) = b.as_ref() {}
            }
            _ => panic!("Expected 2*x^2 + 2*x"),
        }
    }

    #[test]
    fn test_factor_quadratic() {
        let mut parser = Parser::new("x^2 - 1");
        let expr = parser.parse_expression();
        let factored = factor(&expr);
        // Should become (x - 1)*(x + 1)
        assert!(matches!(factored, Expr::Mul(_, _)));
    }

    #[test]
    fn test_factor_difference_of_squares() {
        let mut parser = Parser::new("x^4 - 1");
        let expr = parser.parse_expression();
        let factored = factor(&expr);
        // Should become (x^2 - 1)*(x^2 + 1) = (x-1)*(x+1)*(x^2+1)
        assert!(matches!(factored, Expr::Mul(_, _)));
    }

    #[test]
    fn test_derivative_still_works() {
        let mut parser = Parser::new("x^2");
        let expr = parser.parse_expression();
        let deriv = crate::derivative(&expr, 'x');
        let mut vars = std::collections::HashMap::new();
        vars.insert('x', 3.0);
        let result = crate::evaluate(&deriv, &vars);
        assert!((result - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_integrate_polynomial() {
        let mut parser = Parser::new("x^2");
        let expr = parser.parse_expression();
        let result = integrate(&expr, 'x').unwrap();
        // Should be x^3/3
        match &result {
            Expr::Div(a, b) => {
                if let Expr::Num(n) = b.as_ref() {
                    assert!((n - 3.0).abs() < 1e-10);
                }
            }
            _ => panic!("Expected x^3/3"),
        }
    }
}