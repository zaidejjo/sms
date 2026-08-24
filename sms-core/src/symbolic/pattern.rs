//! Pattern Matching Engine
//!
//! Unification-based pattern matching for expression rewriting.

#![allow(unused)]

use crate::expr::Expr;
use std::collections::HashMap;

/// Pattern variable (wildcard that can match any subexpression)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PatternVar(pub String);

/// A pattern that can contain variables
#[derive(Debug, Clone)]
pub enum Pattern {
    /// Match a specific constant
    Num(f64),
    /// Match a specific variable
    Var(char),
    /// Match any expression (wildcard)
    Wildcard(PatternVar),
    /// Match any expression but with a name for capture
    Capture(PatternVar),
    /// Pattern addition
    Add(Box<Pattern>, Box<Pattern>),
    /// Pattern subtraction
    Sub(Box<Pattern>, Box<Pattern>),
    /// Pattern multiplication
    Mul(Box<Pattern>, Box<Pattern>),
    /// Pattern division
    Div(Box<Pattern>, Box<Pattern>),
    /// Pattern power
    Pow(Box<Pattern>, Box<Pattern>),
    /// Pattern function application
    Func(String, Box<Pattern>),
}

/// Match result with captured variables
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub bindings: HashMap<PatternVar, Expr>,
}

impl MatchResult {
    pub fn new() -> Self {
        MatchResult {
            bindings: HashMap::new(),
        }
    }

    pub fn bind(&mut self, var: PatternVar, expr: Expr) -> bool {
        if let Some(existing) = self.bindings.get(&var) {
            exprs_equal(existing, &expr)
        } else {
            self.bindings.insert(var, expr);
            true
        }
    }

    pub fn get(&self, var: &PatternVar) -> Option<&Expr> {
        self.bindings.get(var)
    }

    pub fn substitute(&self, template: &Expr) -> Expr {
        substitute_vars(template, &self.bindings)
    }
}

/// Try to match an expression against a pattern
pub fn match_pattern(pattern: &Pattern, expr: &Expr) -> Option<MatchResult> {
    let mut result = MatchResult::new();
    if match_rec(pattern, expr, &mut result) {
        Some(result)
    } else {
        None
    }
}

fn match_rec(pattern: &Pattern, expr: &Expr, result: &mut MatchResult) -> bool {
    match (pattern, expr) {
        (Pattern::Num(p), Expr::Num(e)) => (p - e).abs() < 1e-12,
        (Pattern::Var(p), Expr::Var(e)) => p == e,
        (Pattern::Wildcard(_), _) => true,
        (Pattern::Capture(var), e) => result.bind(var.clone(), e.clone()),
        (Pattern::Add(p1, p2), Expr::Add(e1, e2)) => {
            match_rec(p1, e1, result) && match_rec(p2, e2, result)
        }
        (Pattern::Sub(p1, p2), Expr::Sub(e1, e2)) => {
            match_rec(p1, e1, result) && match_rec(p2, e2, result)
        }
        (Pattern::Mul(p1, p2), Expr::Mul(e1, e2)) => {
            // Try both orders for commutativity
            (match_rec(p1, e1, result) && match_rec(p2, e2, result))
                || (match_rec(p1, e2, result) && match_rec(p2, e1, result))
        }
        (Pattern::Div(p1, p2), Expr::Div(e1, e2)) => {
            match_rec(p1, e1, result) && match_rec(p2, e2, result)
        }
        (Pattern::Pow(p1, p2), Expr::Pow(e1, e2)) => {
            match_rec(p1, e1, result) && match_rec(p2, e2, result)
        }
        (Pattern::Func(name, pat), func_expr) => {
            match func_expr {
                Expr::Sin(e) if name == "sin" => match_rec(pat, e, result),
                Expr::Cos(e) if name == "cos" => match_rec(pat, e, result),
                Expr::Tan(e) if name == "tan" => match_rec(pat, e, result),
                Expr::Ln(e) if name == "ln" => match_rec(pat, e, result),
                Expr::Exp(e) if name == "exp" => match_rec(pat, e, result),
                Expr::Sqrt(e) if name == "sqrt" => match_rec(pat, e, result),
                _ => false,
            }
        }
        _ => false,
    }
}

/// Substitute pattern variables in a template expression
fn substitute_vars(expr: &Expr, bindings: &HashMap<PatternVar, Expr>) -> Expr {
    // This is a simplified version - in practice you'd have pattern variables
    // embedded in the template. For now, just return the expression as-is.
    expr.clone()
}

/// Check if two expressions are structurally equal
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
        (Expr::Ln(a1), Expr::Ln(a2)) => exprs_equal(a1, a2),
        (Expr::Exp(a1), Expr::Exp(a2)) => exprs_equal(a1, a2),
        (Expr::Sqrt(a1), Expr::Sqrt(a2)) => exprs_equal(a1, a2),
        _ => false,
    }
}

/// Builder for patterns
pub mod pattern_builder {
    use super::*;

    pub fn num(n: f64) -> Pattern {
        Pattern::Num(n)
    }

    pub fn var(v: char) -> Pattern {
        Pattern::Var(v)
    }

    pub fn wildcard(name: &str) -> Pattern {
        Pattern::Wildcard(PatternVar(name.to_string()))
    }

    pub fn capture(name: &str) -> Pattern {
        Pattern::Capture(PatternVar(name.to_string()))
    }

    pub fn add(a: Pattern, b: Pattern) -> Pattern {
        Pattern::Add(Box::new(a), Box::new(b))
    }

    pub fn sub(a: Pattern, b: Pattern) -> Pattern {
        Pattern::Sub(Box::new(a), Box::new(b))
    }

    pub fn mul(a: Pattern, b: Pattern) -> Pattern {
        Pattern::Mul(Box::new(a), Box::new(b))
    }

    pub fn div(a: Pattern, b: Pattern) -> Pattern {
        Pattern::Div(Box::new(a), Box::new(b))
    }

    pub fn pow(a: Pattern, b: Pattern) -> Pattern {
        Pattern::Pow(Box::new(a), Box::new(b))
    }

    pub fn func(name: &str, arg: Pattern) -> Pattern {
        Pattern::Func(name.to_string(), Box::new(arg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;
    use pattern_builder::*;

    #[test]
    fn test_match_num() {
        let pattern = num(5.0);
        let mut parser = Parser::new("5");
        let expr = parser.parse_expression();
        assert!(match_pattern(&pattern, &expr).is_some());
    }

    #[test]
    fn test_match_var() {
        let pattern = var('x');
        let mut parser = Parser::new("x");
        let expr = parser.parse_expression();
        assert!(match_pattern(&pattern, &expr).is_some());
    }

    #[test]
    fn test_match_add() {
        let pattern = add(capture("a"), capture("b"));
        let mut parser = Parser::new("x + y");
        let expr = parser.parse_expression();
        let result = match_pattern(&pattern, &expr).unwrap();
        assert!(result.get(&PatternVar("a".to_string())).is_some());
        assert!(result.get(&PatternVar("b".to_string())).is_some());
    }

    #[test]
    fn test_match_mul() {
        let pattern = mul(capture("a"), capture("b"));
        let mut parser = Parser::new("x * y");
        let expr = parser.parse_expression();
        let result = match_pattern(&pattern, &expr).unwrap();
        assert!(result.get(&PatternVar("a".to_string())).is_some());
        assert!(result.get(&PatternVar("b".to_string())).is_some());
    }

    #[test]
    fn test_match_sin() {
        let pattern = func("sin", capture("x"));
        let mut parser = Parser::new("sin(y)");
        let expr = parser.parse_expression();
        let result = match_pattern(&pattern, &expr).unwrap();
        assert!(result.get(&PatternVar("x".to_string())).is_some());
    }
}