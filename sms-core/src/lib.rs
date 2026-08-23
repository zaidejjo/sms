//! SMS Core Library
//! 
//! High-performance mathematical equation solver and utilities.
//! 
//! # Features
//! - Equation solving (polynomial, trigonometric, exponential, logarithmic)
//! - Linear systems (Gaussian elimination)
//! - Series computation (sum, product)
//! - AI-powered gradient descent solver
//! - Complex number support
//! - Mathematical constants
//! - Fraction handling
//! - Export (JSON, CSV, LaTeX)

#![allow(box_patterns)]

pub mod expr;
pub mod parser;
pub mod solver;
pub mod matrix;
pub mod series;
pub mod ai;
pub mod constants;
pub mod fractions;
pub mod export;

#[cfg(feature = "symbolic")]
pub mod symbolic;

#[cfg(feature = "units")]
pub mod units;

// Re-export commonly used types
pub use expr::{Expr, evaluate, evaluate_complex};
pub use parser::Parser;
pub use solver::{EquationSolver, derivative};
pub use matrix::Matrix;
pub use series::{compute_series, SeriesOp};
pub use ai::AISolver;
pub use constants::CONSTANTS;
pub use fractions::Fraction;
pub use export::ExportData;

#[cfg(test)]
mod tests {
    use super::*;
    use num::Complex;

    #[test]
    fn test_parse_simple_equation() {
        let mut parser = Parser::new("x^2 - 4");
        let expr = parser.parse_equation();
        assert!(matches!(expr, Expr::Sub(_, _)));
    }

    #[test]
    fn test_parse_trigonometric() {
        let mut parser = Parser::new("sin(x) = 0.5");
        let expr = parser.parse_equation();
        assert!(matches!(expr, Expr::Sub(_, _)));
    }

    #[test]
    fn test_evaluate_constant() {
        let mut parser = Parser::new("pi");
        let expr = parser.parse_expression();
        let result = evaluate(&expr, &std::collections::HashMap::new());
        assert!((result - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_evaluate_expression() {
        let mut parser = Parser::new("2 + 3 * 4");
        let expr = parser.parse_expression();
        let result = evaluate(&expr, &std::collections::HashMap::new());
        assert_eq!(result, 14.0);
    }

    #[test]
    fn test_solve_quadratic() {
        let mut parser = Parser::new("x^2 - 4");
        let expr = parser.parse_equation();
        let solver = EquationSolver::new_adaptive(&expr);
        let (real, complex) = solver.find_all_roots(&expr, 'x');
        
        assert_eq!(real.len(), 2);
        assert!(real.contains(&2.0) || (real[0] - 2.0).abs() < 1e-4);
        assert!(real.contains(&-2.0) || (real[1] - (-2.0)).abs() < 1e-4);
        assert!(complex.is_empty());
    }

    #[test]
    fn test_solve_cubic() {
        let mut parser = Parser::new("x^3 - 6*x^2 + 11*x - 6");
        let expr = parser.parse_equation();
        let solver = EquationSolver::new_adaptive(&expr);
        let (real, _) = solver.find_all_roots(&expr, 'x');
        
        assert_eq!(real.len(), 3);
        for root in &[1.0, 2.0, 3.0] {
            assert!(real.iter().any(|r| (r - root).abs() < 1e-3));
        }
    }

    #[test]
    fn test_solve_trigonometric() {
        let mut parser = Parser::new("sin(x) = 0");
        let expr = parser.parse_equation();
        let solver = EquationSolver::new_adaptive(&expr);
        let (real, _) = solver.find_all_roots(&expr, 'x');
        
        // Should find x = 0, x = pi, x = -pi, etc.
        assert!(!real.is_empty());
        assert!(real.iter().any(|r| r.abs() < 1e-3));
    }

    #[test]
    fn test_matrix_solve() {
        let matrix = Matrix::new(vec![vec![2.0, 3.0], vec![4.0, -1.0]]);
        let vector = vec![8.0, 6.0];
        let solution = matrix.solve_linear(&vector).unwrap();
        
        assert_eq!(solution.len(), 2);
        // 2x + 3y = 8, 4x - y = 6
        // x = 13/7, y = 10/7
        assert!((solution[0] - 13.0/7.0).abs() < 1e-6);
        assert!((solution[1] - 10.0/7.0).abs() < 1e-6);
    }

    #[test]
    fn test_series_sum() {
        let mut parser = Parser::new("i^2");
        let expr = parser.parse_expression();
        let result = compute_series(&expr, 'i', 1, 10, SeriesOp::Sum);
        assert_eq!(result, 385.0); // 1^2 + 2^2 + ... + 10^2 = 385
    }

    #[test]
    fn test_series_product() {
        let mut parser = Parser::new("i");
        let expr = parser.parse_expression();
        let result = compute_series(&expr, 'i', 1, 5, SeriesOp::Product);
        assert_eq!(result, 120.0); // 5! = 120
    }

    #[test]
    fn test_fraction_from_f64() {
        let frac = Fraction::from_f64(0.5, 1e-6).unwrap();
        assert_eq!(frac.numerator, 1);
        assert_eq!(frac.denominator, 2);
        
        let frac = Fraction::from_f64(0.333333, 1e-5).unwrap();
        assert_eq!(frac.numerator, 1);
        assert_eq!(frac.denominator, 3);
    }

    #[test]
    fn test_fraction_arithmetic() {
        let a = Fraction::new(1, 2);
        let b = Fraction::new(1, 3);
        let sum = a + b;
        assert_eq!(sum.numerator, 5);
        assert_eq!(sum.denominator, 6);
        
        let prod = a * b;
        assert_eq!(prod.numerator, 1);
        assert_eq!(prod.denominator, 6);
    }

    #[test]
    fn test_complex_evaluation() {
        let mut parser = Parser::new("x^2 + 1");
        let expr = parser.parse_equation();
        let mut vars = std::collections::HashMap::new();
        vars.insert('x', Complex::new(0.0, 1.0));
        let result = evaluate_complex(&expr, &vars);
        assert!((result.re - 0.0).abs() < 1e-10);
        assert!((result.im - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_derivative() {
        let mut parser = Parser::new("x^2");
        let expr = parser.parse_expression();
        let deriv = derivative(&expr, 'x');
        
        // derivative of x^2 is 2*x
        let mut vars = std::collections::HashMap::new();
        vars.insert('x', 3.0);
        let result = evaluate(&deriv, &vars);
        assert!((result - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_ai_solver() {
        let mut parser = Parser::new("x^2 - 2");
        let expr = parser.parse_equation();
        let ai = AISolver::new();
        let (solution, error, iterations) = ai.solve_advanced(&expr, 'x');
        
        assert!(solution.is_some());
        let x = solution.unwrap();
        assert!((x - std::f64::consts::SQRT_2).abs() < 1e-4);
        assert!(error < 1e-6);
        assert!(iterations > 0);
    }

    #[test]
    fn test_constants() {
        assert!((CONSTANTS["pi"] - std::f64::consts::PI).abs() < 1e-10);
        assert!((CONSTANTS["e"] - std::f64::consts::E).abs() < 1e-10);
        assert!((CONSTANTS["phi"] - 1.618033988749895).abs() < 1e-10);
        assert!((CONSTANTS["tau"] - std::f64::consts::TAU).abs() < 1e-10);
    }

    #[test]
    fn test_export_data() {
        let mut data = ExportData::new("x^2 - 4".to_string(), "x".to_string(), 1.5);
        data.add_solution(2.0, 1e-10);
        data.add_solution(-2.0, 1e-10);
        
        assert_eq!(data.solutions.len(), 2);
        assert_eq!(data.equation, "x^2 - 4");
        assert_eq!(data.variable, "x");
    }
}