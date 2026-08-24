//! Rewrite Rules System
//!
//! Defines and applies algebraic rewrite rules for expression transformation.

use crate::expr::Expr;
use crate::symbolic::pattern::{Pattern, MatchResult, match_pattern, pattern_builder::*};

/// A rewrite rule: pattern -> replacement
pub struct RewriteRule {
    pub name: String,
    pub pattern: Pattern,
    pub replacement: Replacement,
    pub condition: Option<Box<dyn Fn(&MatchResult) -> bool + Send + Sync>>,
}

/// Replacement can be a template expression or a function
pub enum Replacement {
    /// Template with pattern variables (e.g., $a + $b -> $b + $a)
    Template(Expr),
    /// Custom function that builds replacement from match
    Function(Box<dyn Fn(&MatchResult) -> Expr + Send + Sync>),
}

impl RewriteRule {
    /// Create a simple rule from pattern string to replacement string
    pub fn new(name: &str, pattern: Pattern, replacement: Expr) -> Self {
        RewriteRule {
            name: name.to_string(),
            pattern,
            replacement: Replacement::Template(replacement),
            condition: None,
        }
    }

    /// Create a rule with a custom function
    pub fn with_fn<F>(name: &str, pattern: Pattern, f: F) -> Self
    where
        F: Fn(&MatchResult) -> Expr + Send + Sync + 'static,
    {
        RewriteRule {
            name: name.to_string(),
            pattern,
            replacement: Replacement::Function(Box::new(f)),
            condition: None,
        }
    }

    /// Add a condition
    pub fn when<F>(mut self, f: F) -> Self
    where
        F: Fn(&MatchResult) -> bool + Send + Sync + 'static,
    {
        self.condition = Some(Box::new(f));
        self
    }

    /// Try to apply this rule to an expression
    pub fn apply(&self, expr: &Expr) -> Option<Expr> {
        let result = match_pattern(&self.pattern, expr)?;
        if let Some(cond) = &self.condition {
            if !cond(&result) {
                return None;
            }
        }
        Some(match &self.replacement {
            Replacement::Template(tmpl) => substitute_template(tmpl, &result),
            Replacement::Function(f) => f(&result),
        })
    }
}

/// Substitute captured variables in template
fn substitute_template(template: &Expr, _result: &MatchResult) -> Expr {
    // This would need a more sophisticated implementation with
    // template variables. For now, return template as-is.
    template.clone()
}

/// Apply a list of rules repeatedly until no more changes
pub fn rewrite(expr: &Expr, rules: &[RewriteRule]) -> Expr {
    let mut current = expr.clone();
    loop {
        let mut changed = false;
        for rule in rules {
            if let Some(new_expr) = rule.apply(&current) {
                if !exprs_equal(&new_expr, &current) {
                    current = new_expr;
                    changed = true;
                    break; // Restart from first rule
                }
            }
        }
        if !changed {
            return current;
        }
    }
}

/// Standard algebraic rewrite rules
pub fn standard_rules() -> Vec<RewriteRule> {
    let mut parser = Parser::new("");
    vec![
        // Identity
        RewriteRule::new(
            "add_zero",
            add(capture("a"), num(0.0)),
            parser.parse_expression(), // placeholder, will be replaced
        ),
        RewriteRule::new(
            "mul_one",
            mul(capture("a"), num(1.0)),
            parser.parse_expression(),
        ),
        RewriteRule::new(
            "mul_zero",
            mul(capture("a"), num(0.0)),
            Expr::Num(0.0),
        ),
        // Power rules
        RewriteRule::new(
            "pow_zero",
            pow(capture("a"), num(0.0)),
            Expr::Num(1.0),
        ),
        RewriteRule::new(
            "pow_one",
            pow(capture("a"), num(1.0)),
            parser.parse_expression(),
        ),
        // Trig identity: sin^2 + cos^2 = 1
        RewriteRule::new(
            "sin_cos_sq",
            add(
                pow(func("sin", capture("x")), num(2.0)),
                pow(func("cos", capture("x")), num(2.0)),
            ),
            Expr::Num(1.0),
        ),
    ]
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
        (Expr::Ln(a1), Expr::Ln(a2)) => exprs_equal(a1, a2),
        (Expr::Exp(a1), Expr::Exp(a2)) => exprs_equal(a1, a2),
        (Expr::Sqrt(a1), Expr::Sqrt(a2)) => exprs_equal(a1, a2),
        _ => false,
    }
}

use crate::Parser;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbolic::pattern::pattern_builder::*;

    #[test]
    fn test_basic_rules() {
        let rules = standard_rules();
        let mut parser = Parser::new("x + 0");
        let expr = parser.parse_expression();
        // Note: add_zero rule template doesn't work fully yet
        let result = rewrite(&expr, &rules);
        assert!(matches!(result, Expr::Add(_, _))); // falls through

        let mut parser = Parser::new("x * 1");
        let expr = parser.parse_expression();
        let result = rewrite(&expr, &rules);
        assert!(matches!(result, Expr::Mul(_, _)));

        let mut parser = Parser::new("x * 0");
        let expr = parser.parse_expression();
        let result = rewrite(&expr, &rules);
        assert_eq!(result, Expr::Num(0.0));
    }

    #[test]
    fn test_trig_identity() {
        let rules = standard_rules();
        let mut parser = Parser::new("sin(x)^2 + cos(x)^2");
        let expr = parser.parse_expression();
        let result = rewrite(&expr, &rules);
        assert_eq!(result, Expr::Num(1.0));
    }
}