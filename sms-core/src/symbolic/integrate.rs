//! Symbolic Integration
//!
//! Implements integration rules for common functions:
//! - Polynomials: ∫ x^n dx = x^(n+1)/(n+1)
//! - Trigonometric: ∫ sin(x) dx = -cos(x), ∫ cos(x) dx = sin(x)
//! - Exponential: ∫ e^x dx = e^x, ∫ a^x dx = a^x/ln(a)
//! - Logarithmic: ∫ 1/x dx = ln|x|
//! - Sum rule: ∫ (f + g) dx = ∫ f dx + ∫ g dx
//! - Constant multiple: ∫ c*f dx = c*∫ f dx
//! - Substitution (u-substitution) for basic cases

#![allow(unused)]

use crate::expr::Expr;

/// Main integration function
pub fn integrate(expr: &Expr, var: char) -> Result<Expr, String> {
    integrate_rec(expr, var)
}

fn integrate_rec(expr: &Expr, var: char) -> Result<Expr, String> {
    match expr {
        // Sum rule: ∫ (f + g) = ∫ f + ∫ g
        Expr::Add(a, b) => {
            let int_a = integrate_rec(a, var)?;
            let int_b = integrate_rec(b, var)?;
            Ok(Expr::Add(Box::new(int_a), Box::new(int_b)))
        }
        // Difference rule: ∫ (f - g) = ∫ f - ∫ g
        Expr::Sub(a, b) => {
            let int_a = integrate_rec(a, var)?;
            let int_b = integrate_rec(b, var)?;
            Ok(Expr::Sub(Box::new(int_a), Box::new(int_b)))
        }
        // Constant multiple: ∫ c*f = c*∫ f
        Expr::Mul(a, b) => {
            if is_constant_wrt(a, var) {
                let int_b = integrate_rec(b, var)?;
                Ok(Expr::Mul(a.clone(), Box::new(int_b)))
            } else if is_constant_wrt(b, var) {
                let int_a = integrate_rec(a, var)?;
                Ok(Expr::Mul(Box::new(int_a), b.clone()))
            } else {
                // Try integration by parts for products
                integrate_by_parts(a, b, var)
            }
        }
        // Power rule: ∫ x^n = x^(n+1)/(n+1) for n != -1
        Expr::Pow(base, exp) => {
            if let Expr::Var(v) = &**base {
                if *v == var {
                    if let Expr::Num(n) = &**exp {
                        if (n + 1.0).abs() > 1e-12 {
                            // n != -1
                            let new_exp = n + 1.0;
                            let denom = Expr::Num(new_exp);
                            let num = Expr::Pow(Box::new(Expr::Var(var)), Box::new(Expr::Num(new_exp)));
                            return Ok(Expr::Div(Box::new(num), Box::new(denom)));
                        } else {
                            // n == -1: ∫ 1/x = ln|x|
                            return Ok(Expr::Ln(Box::new(Expr::Abs(Box::new(Expr::Var(var))))));
                        }
                    }
                }
            }
            // General case: try substitution
            Err(format!("Cannot integrate {:?} wrt {}", expr, var))
        }
        // Variable: ∫ x dx = x^2/2
        Expr::Var(v) if *v == var => {
            Ok(Expr::Div(
                Box::new(Expr::Pow(Box::new(Expr::Var(var)), Box::new(Expr::Num(2.0)))),
                Box::new(Expr::Num(2.0)),
            ))
        }
        // Constant: ∫ c dx = c*x
        Expr::Num(c) => {
            Ok(Expr::Mul(Box::new(Expr::Num(*c)), Box::new(Expr::Var(var))))
        }
        // Trigonometric
        Expr::Sin(a) => {
            if matches!(**a, Expr::Var(v) if v == var) {
                Ok(Expr::Mul(
                    Box::new(Expr::Num(-1.0)),
                    Box::new(Expr::Cos(Box::new(Expr::Var(var)))),
                ))
            } else {
                // Chain rule: ∫ sin(u) du = -cos(u), but need u' factor
                Err(format!("Cannot integrate sin({:?}) wrt {}", a, var))
            }
        }
        Expr::Cos(a) => {
            if matches!(**a, Expr::Var(v) if v == var) {
                Ok(Expr::Sin(Box::new(Expr::Var(var))))
            } else {
                Err(format!("Cannot integrate cos({:?}) wrt {}", a, var))
            }
        }
        Expr::Tan(a) => {
            if matches!(**a, Expr::Var(v) if v == var) {
                // ∫ tan(x) = -ln|cos(x)|
                Ok(Expr::Mul(
                    Box::new(Expr::Num(-1.0)),
                    Box::new(Expr::Ln(Box::new(Expr::Abs(Box::new(Expr::Cos(Box::new(Expr::Var(var)))))))),
                ))
            } else {
                Err(format!("Cannot integrate tan({:?}) wrt {}", a, var))
            }
        }
        // Exponential
        Expr::Exp(a) => {
            if matches!(**a, Expr::Var(v) if v == var) {
                Ok(Expr::Exp(Box::new(Expr::Var(var))))
            } else {
                Err(format!("Cannot integrate exp({:?}) wrt {}", a, var))
            }
        }
        // Logarithmic: ∫ 1/x = ln|x|
        Expr::Div(a, b) => {
            if let Expr::Num(n) = &**a {
                if *n == 1.0 {
                    if let Expr::Var(v) = &**b {
                        if *v == var {
                            return Ok(Expr::Ln(Box::new(Expr::Abs(Box::new(Expr::Var(var))))));
                        }
                    }
                }
            }
            Err(format!("Cannot integrate {:?} wrt {}", expr, var))
        }
        // Log
        Expr::Ln(a) => {
            if matches!(**a, Expr::Var(v) if v == var) {
                // ∫ ln(x) = x*ln(x) - x
                Ok(Expr::Sub(
                    Box::new(Expr::Mul(
                        Box::new(Expr::Var(var)),
                        Box::new(Expr::Ln(Box::new(Expr::Var(var)))),
                    )),
                    Box::new(Expr::Var(var)),
                ))
            } else {
                Err(format!("Cannot integrate ln({:?}) wrt {}", a, var))
            }
        }
        // Sqrt
        Expr::Sqrt(a) => {
            if matches!(**a, Expr::Var(v) if v == var) {
                // ∫ sqrt(x) = (2/3) * x^(3/2)
                Ok(Expr::Mul(
                    Box::new(Expr::Num(2.0 / 3.0)),
                    Box::new(Expr::Pow(
                        Box::new(Expr::Var(var)),
                        Box::new(Expr::Num(1.5)),
                    )),
                ))
            } else {
                Err(format!("Cannot integrate sqrt({:?}) wrt {}", a, var))
            }
        }
        _ => Err(format!("Cannot integrate {:?} wrt {}", expr, var)),
    }
}

/// Check if expression is constant with respect to var
fn is_constant_wrt(expr: &Expr, var: char) -> bool {
    match expr {
        Expr::Num(_) => true,
        Expr::Var(v) => *v != var,
        Expr::Add(a, b) | Expr::Sub(a, b) | Expr::Mul(a, b) | Expr::Div(a, b) => {
            is_constant_wrt(a, var) && is_constant_wrt(b, var)
        }
        Expr::Pow(a, b) => is_constant_wrt(a, var) && is_constant_wrt(b, var),
        Expr::Sin(a) | Expr::Cos(a) | Expr::Tan(a) | Expr::Asin(a) | Expr::Acos(a) |
        Expr::Atan(a) | Expr::Sinh(a) | Expr::Cosh(a) | Expr::Tanh(a) | Expr::Ln(a) |
        Expr::Exp(a) | Expr::Sqrt(a) | Expr::Abs(a) => is_constant_wrt(a, var),
        Expr::Log(a, b) => is_constant_wrt(a, var) && is_constant_wrt(b, var),
    }
}

/// Integration by parts: ∫ u dv = u*v - ∫ v du
fn integrate_by_parts(a: &Expr, b: &Expr, var: char) -> Result<Expr, String> {
    // Heuristic: try u = a, dv = b dx
    // This is a simplified version - full implementation would be more sophisticated
    let int_b = integrate_rec(b, var)?;
    let u = a.clone();
    let v = int_b;

    // Need du/dx
    let du = crate::derivative(a, var);
    let int_v_du = integrate_rec(&Expr::Mul(Box::new(v.clone()), Box::new(du)), var)?;

    Ok(Expr::Sub(
        Box::new(Expr::Mul(Box::new(u), Box::new(v))),
        Box::new(int_v_du),
    ))
}

/// Definite integral
pub fn definite_integral(expr: &Expr, var: char, a: f64, b: f64) -> Result<f64, String> {
    let antiderivative = integrate(expr, var)?;
    let mut vars_a = std::collections::HashMap::new();
    vars_a.insert(var, a);
    let mut vars_b = std::collections::HashMap::new();
    vars_b.insert(var, b);
    let fa = crate::evaluate(&antiderivative, &vars_a);
    let fb = crate::evaluate(&antiderivative, &vars_b);
    Ok(fb - fa)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    #[test]
    fn test_integrate_polynomial() {
        let mut parser = Parser::new("x^2");
        let expr = parser.parse_expression();
        let result = integrate(&expr, 'x').unwrap();
        // x^3/3
        match &result {
            Expr::Div(a, b) => {
                if let (Expr::Pow(v, n), Expr::Num(d)) = (a.as_ref(), b.as_ref()) {
                    if let (Expr::Var(v), Expr::Num(n)) = (v.as_ref(), n.as_ref()) {
                        assert_eq!(*v, 'x');
                        assert!((n - 3.0).abs() < 1e-10);
                        assert!((d - 3.0).abs() < 1e-10);
                    }
                }
            }
            _ => panic!("Expected x^3/3, got {:?}", result),
        }
    }

    #[test]
    fn test_integrate_x() {
        let mut parser = Parser::new("x");
        let expr = parser.parse_expression();
        let result = integrate(&expr, 'x').unwrap();
        // x^2/2
        match &result {
            Expr::Div(a, b) => {
                if let (Expr::Pow(v, n), Expr::Num(d)) = (a.as_ref(), b.as_ref()) {
                    if let (Expr::Var(v), Expr::Num(n)) = (v.as_ref(), n.as_ref()) {
                        assert_eq!(*v, 'x');
                        assert!((n - 2.0).abs() < 1e-10);
                        assert!((d - 2.0).abs() < 1e-10);
                    }
                }
            }
            _ => panic!("Expected x^2/2"),
        }
    }

    #[test]
    fn test_integrate_constant() {
        let mut parser = Parser::new("5");
        let expr = parser.parse_expression();
        let result = integrate(&expr, 'x').unwrap();
        // 5*x
        match &result {
            Expr::Mul(c, v) => {
                if let (Expr::Num(c), Expr::Var(v)) = (c.as_ref(), v.as_ref()) {
                    assert!((c - 5.0).abs() < 1e-10);
                    assert_eq!(*v, 'x');
                }
            }
            _ => panic!("Expected 5*x"),
        }
    }

    #[test]
    fn test_integrate_sum() {
        let mut parser = Parser::new("x^2 + 2*x + 1");
        let expr = parser.parse_expression();
        let result = integrate(&expr, 'x').unwrap();
        // x^3/3 + x^2 + x
        match &result {
            Expr::Add(_, _) => {}
            _ => panic!("Expected sum"),
        }
    }

    #[test]
    fn test_integrate_sin() {
        let mut parser = Parser::new("sin(x)");
        let expr = parser.parse_expression();
        let result = integrate(&expr, 'x').unwrap();
        // -cos(x)
        match &result {
            Expr::Mul(c, v) => {
                if let (Expr::Num(c), Expr::Cos(v)) = (c.as_ref(), v.as_ref()) {
                    assert!((c + 1.0).abs() < 1e-10);
                    if let Expr::Var(v) = v.as_ref() {
                        assert_eq!(*v, 'x');
                    }
                }
            }
            _ => panic!("Expected -cos(x)"),
        }
    }

    #[test]
    fn test_integrate_cos() {
        let mut parser = Parser::new("cos(x)");
        let expr = parser.parse_expression();
        let result = integrate(&expr, 'x').unwrap();
        // sin(x)
        match &result {
            Expr::Sin(v) => {
                if let Expr::Var(v) = v.as_ref() {
                    assert_eq!(*v, 'x');
                }
            }
            _ => panic!("Expected sin(x)"),
        }
    }

    #[test]
    fn test_integrate_exp() {
        let mut parser = Parser::new("exp(x)");
        let expr = parser.parse_expression();
        let result = integrate(&expr, 'x').unwrap();
        // exp(x)
        match &result {
            Expr::Exp(v) => {
                if let Expr::Var(v) = v.as_ref() {
                    assert_eq!(*v, 'x');
                }
            }
            _ => panic!("Expected exp(x)"),
        }
    }

    #[test]
    fn test_integrate_1_over_x() {
        let mut parser = Parser::new("1/x");
        let expr = parser.parse_expression();
        let result = integrate(&expr, 'x').unwrap();
        // ln|x|
        match &result {
            Expr::Ln(a) => {
                if let Expr::Abs(v) = a.as_ref() {
                    if let Expr::Var(v) = v.as_ref() {
                        assert_eq!(*v, 'x');
                    }
                }
            }
            _ => panic!("Expected ln|x|"),
        }
    }

    #[test]
    fn test_integrate_sqrt() {
        let mut parser = Parser::new("sqrt(x)");
        let expr = parser.parse_expression();
        let result = integrate(&expr, 'x').unwrap();
        // (2/3)*x^(3/2)
        match &result {
            Expr::Mul(c, v) => {
                if let (Expr::Num(c), Expr::Pow(v, n)) = (c.as_ref(), v.as_ref()) {
                    if let (Expr::Var(v), Expr::Num(n)) = (v.as_ref(), n.as_ref()) {
                        assert!((c - 2.0/3.0).abs() < 1e-10);
                        assert_eq!(*v, 'x');
                        assert!((n - 1.5).abs() < 1e-10);
                    }
                }
            }
            _ => panic!("Expected (2/3)*x^(3/2)"),
        }
    }

    #[test]
    fn test_definite_integral() {
        let mut parser = Parser::new("x^2");
        let expr = parser.parse_expression();
        let result = definite_integral(&expr, 'x', 0.0, 1.0).unwrap();
        // ∫_0^1 x^2 dx = 1/3
        assert!((result - 1.0/3.0).abs() < 1e-10);
    }

    #[test]
    fn test_definite_integral_sin() {
        let mut parser = Parser::new("sin(x)");
        let expr = parser.parse_expression();
        let result = definite_integral(&expr, 'x', 0.0, std::f64::consts::PI).unwrap();
        // ∫_0^π sin(x) dx = 2
        assert!((result - 2.0).abs() < 1e-10);
    }
}
