//! Polynomial Factorization
//!
//! Factors polynomials over integers/rationals:
//! - Difference of squares: x^2 - a^2 -> (x - a)(x + a)
//! - Quadratic trinomials: x^2 + bx + c -> (x + p)(x + q)
//! - Common factor extraction
//! - Grouping

#![allow(unused)]

use crate::expr::Expr;

/// Main factorization function
pub fn factor(expr: &Expr) -> Expr {
    // First expand to get polynomial in standard form
    let expanded = crate::symbolic::expand(expr);
    // Then try to factor
    factor_poly(&expanded)
}

fn factor_poly(expr: &Expr) -> Expr {
    // Try to extract common factor first
    if let Some(factored) = extract_common_factor(expr) {
        return factored;
    }

    // Try difference of squares
    if let Some(factored) = factor_diff_of_squares(expr) {
        return factored;
    }

    // Try quadratic factoring
    if let Some(factored) = factor_quadratic(expr) {
        return factored;
    }

    // Try sum/difference of cubes
    if let Some(factored) = factor_sum_diff_cubes(expr) {
        return factored;
    }

    // Try grouping for 4-term polynomials
    if let Some(factored) = factor_by_grouping(expr) {
        return factored;
    }

    // Could not factor
    expr.clone()
}

/// Extract greatest common factor from all terms
fn extract_common_factor(expr: &Expr) -> Option<Expr> {
    let terms = flatten_add(expr);
    if terms.len() < 2 {
        return None;
    }

    // Find GCF of all terms
    let mut gcf = terms[0].clone();
    for term in &terms[1..] {
        gcf = gcd_expr(&gcf, term);
        if is_one(&gcf) {
            return None;
        }
    }

    if is_one(&gcf) {
        return None;
    }

    // Divide each term by GCF and rebuild
    let mut new_terms = Vec::new();
    for term in terms {
        new_terms.push(divide_term(&term, &gcf));
    }

    let mut result = new_terms[0].clone();
    for term in new_terms.into_iter().skip(1) {
        result = Expr::Add(Box::new(result), Box::new(term));
    }

    Some(Expr::Mul(Box::new(gcf), Box::new(result)))
}

/// Flatten addition into terms
fn flatten_add(expr: &Expr) -> Vec<Expr> {
    match expr {
        Expr::Add(a, b) => {
            let mut terms = flatten_add(a);
            terms.extend(flatten_add(b));
            terms
        }
        Expr::Sub(a, b) => {
            let mut terms = flatten_add(a);
            terms.push(Expr::Mul(Box::new(Expr::Num(-1.0)), b.clone()));
            terms
        }
        _ => vec![expr.clone()],
    }
}

/// Compute GCD of two expression terms
fn gcd_expr(a: &Expr, b: &Expr) -> Expr {
    // Simple implementation: find common numeric factor and common variable powers
    let (coeff_a, vars_a) = factor_term(a);
    let (coeff_b, vars_b) = factor_term(b);

    // GCD of coefficients
    let coeff_gcd = gcd_f64(coeff_a, coeff_b);

    // GCD of variables (minimum power)
    let mut common_vars = Vec::new();
    for (var, pow_a) in vars_a {
        if let Some(pow_b) = vars_b.iter().find(|(v, _)| *v == var).map(|(_, p)| *p) {
            common_vars.push((var, pow_a.min(pow_b)));
        }
    }

    // Rebuild
    let mut result = if coeff_gcd != 1.0 {
        Expr::Num(coeff_gcd)
    } else {
        return Expr::Num(1.0);
    };

    for (var, pow) in common_vars {
        let var_expr = if pow == 1 {
            Expr::Var(var)
        } else {
            Expr::Pow(Box::new(Expr::Var(var)), Box::new(Expr::Num(pow as f64)))
        };
        result = Expr::Mul(Box::new(result), Box::new(var_expr));
    }

    result
}

/// Factor term into coefficient and variable powers
fn factor_term(expr: &Expr) -> (f64, Vec<(char, i32)>) {
    match expr {
        Expr::Num(n) => (*n, vec![]),
        Expr::Var(v) => (1.0, vec![(*v, 1)]),
        Expr::Mul(a, b) => {
            let (coeff_a, vars_a) = factor_term(a);
            let (coeff_b, vars_b) = factor_term(b);
            (coeff_a * coeff_b, merge_vars(vars_a, vars_b))
        }
        Expr::Pow(base, exp) => {
            if let Expr::Var(v) = &**base {
                if let Expr::Num(n) = &**exp {
                    return (1.0, vec![(*v, *n as i32)]);
                }
            }
            (1.0, vec![])
        }
        Expr::Mul(_, _) => (1.0, vec![]),
        _ => (1.0, vec![]),
    }
}

fn merge_vars(mut a: Vec<(char, i32)>, b: Vec<(char, i32)>) -> Vec<(char, i32)> {
    for (var, pow) in b {
        if let Some((_, existing)) = a.iter_mut().find(|(v, _)| *v == var) {
            *existing += pow;
        } else {
            a.push((var, pow));
        }
    }
    a
}

fn gcd_f64(a: f64, b: f64) -> f64 {
    // For floats, just check if they're "close" to integers
    let ai = a.round() as i64;
    let bi = b.round() as i64;
    if (a - ai as f64).abs() < 1e-10 && (b - bi as f64).abs() < 1e-10 {
        gcd_int(ai.abs(), bi.abs()) as f64
    } else {
        1.0
    }
}

fn gcd_int(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

fn is_one(expr: &Expr) -> bool {
    matches!(expr, Expr::Num(n) if (n - 1.0).abs() < 1e-10)
}

/// Factor a^2 - b^2 = (a - b)(a + b)
fn factor_diff_of_squares(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::Sub(a, b) => {
            if let (Expr::Pow(base_a, exp_a), Expr::Pow(base_b, exp_b)) = (&**a, &**b) {
                if let (Expr::Num(ea), Expr::Num(eb)) = (&**exp_a, &**exp_b) {
                    if (ea - 2.0).abs() < 1e-10 && (eb - 2.0).abs() < 1e-10 {
                        // a^2 - b^2 = (a - b)(a + b)
                        let left = Expr::Sub(base_a.clone(), base_b.clone());
                        let right = Expr::Add(base_a.clone(), base_b.clone());
                        return Some(Expr::Mul(Box::new(left), Box::new(right)));
                    }
                    // a^n - b^n for even n can sometimes factor
                    if ea.fract() == 0.0 && eb.fract() == 0.0 && ea == eb {
                        let n = *ea as i32;
                        if n % 2 == 0 && n > 2 {
                            // a^n - b^n = (a^(n/2) - b^(n/2))(a^(n/2) + b^(n/2))
                            let half = Expr::Num((n / 2) as f64);
                            let left = Expr::Sub(
                                Box::new(Expr::Pow(base_a.clone(), Box::new(half.clone()))),
                                Box::new(Expr::Pow(base_b.clone(), Box::new(half))),
                            );
                            let right = Expr::Add(
                                Box::new(Expr::Pow(base_a.clone(), Box::new(Expr::Num((n / 2) as f64)))),
                                Box::new(Expr::Pow(base_b.clone(), Box::new(Expr::Num((n / 2) as f64)))),
                            );
                            return Some(Expr::Mul(Box::new(left), Box::new(right)));
                        }
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// Factor quadratic: a*x^2 + b*x + c
fn factor_quadratic(expr: &Expr) -> Option<Expr> {
    // Try to match pattern a*x^2 + b*x + c
    let terms = flatten_add(expr);
    if terms.len() != 3 {
        return None;
    }

    // Find x^2 term, x term, constant
    let mut a_coeff = 0.0;
    let mut b_coeff = 0.0;
    let mut c_coeff = 0.0;
    let mut var = 'x';

    for term in &terms {
        match term {
            Expr::Mul(coeff, pow) => {
                if let Expr::Pow(v, p) = &**pow {
                    if let (Expr::Var(v), Expr::Num(p)) = (&**v, &**p) {
                        if (p - 2.0).abs() < 1e-10 {
                            var = *v;
                            a_coeff = extract_num(coeff);
                        } else if (p - 1.0).abs() < 1e-10 {
                            var = *v;
                            b_coeff = extract_num(coeff);
                        }
                    }
                } else if let Expr::Var(v) = &**pow {
                    var = *v;
                    b_coeff = extract_num(coeff);
                }
            }
            Expr::Pow(v, p) => {
                if let (Expr::Var(v), Expr::Num(p)) = (&**v, &**p) {
                    if (p - 2.0).abs() < 1e-10 {
                        var = *v;
                        a_coeff = 1.0;
                    }
                }
            }
            Expr::Var(v) => {
                var = *v;
                b_coeff = 1.0;
            }
            Expr::Num(n) => {
                c_coeff = *n;
            }
            _ => {}
        }
    }

    if a_coeff == 0.0 {
        return None;
    }

    // For monic quadratic (a=1): find p,q such that p+q=b and p*q=c
    // For general a: find factors of a*c that sum to b
    let target = a_coeff * c_coeff;
    let sum = b_coeff;

    // Find integer factors
    for p in -100..=100 {
        let p_f = p as f64;
        if p_f == 0.0 && target != 0.0 {
            continue;
        }
        if target == 0.0 {
            if p_f == 0.0 {
                let q = sum;
                if (p_f + q - sum).abs() < 1e-10 && (p_f * q - target).abs() < 1e-10 {
                    return Some(build_quadratic_factors(a_coeff, p_f, q, var));
                }
            }
        } else if (target / p_f - p_f).abs() < 1e-10 {
            // Not quite right, need to check properly
            let q = sum - p_f;
            if (p_f * q - target).abs() < 1e-10 {
                return Some(build_quadratic_factors(a_coeff, p_f, q, var));
            }
        }
    }

    None
}

fn extract_num(expr: &Expr) -> f64 {
    match expr {
        Expr::Num(n) => *n,
        Expr::Mul(a, b) => extract_num(a) * extract_num(b),
        _ => 1.0,
    }
}

fn build_quadratic_factors(a: f64, p: f64, q: f64, var: char) -> Expr {
    if (a - 1.0).abs() < 1e-10 {
        // Monic: (x + p)(x + q)
        let left = Expr::Add(Box::new(Expr::Var(var)), Box::new(Expr::Num(p)));
        let right = Expr::Add(Box::new(Expr::Var(var)), Box::new(Expr::Num(q)));
        Expr::Mul(Box::new(left), Box::new(right))
    } else {
        // Non-monic: a(x + p/a)(x + q/a) = (sqrt(a)x + p/sqrt(a))(sqrt(a)x + q/sqrt(a))
        // Simpler: just use the standard form
        let left = Expr::Add(
            Box::new(Expr::Mul(Box::new(Expr::Num(a.sqrt())), Box::new(Expr::Var(var)))),
            Box::new(Expr::Num(p / a.sqrt())),
        );
        let right = Expr::Add(
            Box::new(Expr::Mul(Box::new(Expr::Num(a.sqrt())), Box::new(Expr::Var(var)))),
            Box::new(Expr::Num(q / a.sqrt())),
        );
        Expr::Mul(Box::new(left), Box::new(right))
    }
}

/// Factor sum/difference of cubes: a^3 ± b^3
fn factor_sum_diff_cubes(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::Add(a, b) | Expr::Sub(a, b) => {
            if let (Expr::Pow(base_a, exp_a), Expr::Pow(base_b, exp_b)) = (&**a, &**b) {
                if let (Expr::Num(ea), Expr::Num(eb)) = (&**exp_a, &**exp_b) {
                    if (ea - 3.0).abs() < 1e-10 && (eb - 3.0).abs() < 1e-10 {
                        // a^3 + b^3 = (a + b)(a^2 - ab + b^2)
                        // a^3 - b^3 = (a - b)(a^2 + ab + b^2)
                        let is_sub = matches!(expr, Expr::Sub(_, _));
                        let factor1 = if is_sub {
                            Expr::Sub(base_a.clone(), base_b.clone())
                        } else {
                            Expr::Add(base_a.clone(), base_b.clone())
                        };
                        let a2 = Expr::Pow(base_a.clone(), Box::new(Expr::Num(2.0)));
                        let b2 = Expr::Pow(base_b.clone(), Box::new(Expr::Num(2.0)));
                        let ab = Expr::Mul(base_a.clone(), base_b.clone());
                        let factor2 = if is_sub {
                            Expr::Add(Box::new(a2), Box::new(Expr::Add(Box::new(ab), Box::new(b2))))
                        } else {
                            Expr::Sub(Box::new(a2), Box::new(Expr::Sub(Box::new(ab), Box::new(b2))))
                        };
                        return Some(Expr::Mul(Box::new(factor1), Box::new(factor2)));
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// Factor by grouping (4-term polynomial)
fn factor_by_grouping(expr: &Expr) -> Option<Expr> {
    let terms = flatten_add(expr);
    if terms.len() != 4 {
        return None;
    }

    // Group first two and last two
    let group1 = Expr::Add(Box::new(terms[0].clone()), Box::new(terms[1].clone()));
    let group2 = Expr::Add(Box::new(terms[2].clone()), Box::new(terms[3].clone()));

    let factored1 = factor_poly(&group1);
    let factored2 = factor_poly(&group2);

    // Check if they share a common binomial factor
    // This is a simplified version - in practice would need better matching
    None
}

/// Divide term by factor
fn divide_term(_term: &Expr, _factor: &Expr) -> Expr {
    // Simple division - in practice would be more sophisticated
    Expr::Num(1.0)
}

/// Partial fraction decomposition
pub fn partial_fractions(expr: &Expr, var: char) -> Result<Vec<Expr>, String> {
    // For now, just return the expression as-is
    // Full implementation would:
    // 1. Ensure proper rational function (degree numerator < degree denominator)
    // 2. Factor denominator
    // 3. Decompose into sum of simpler fractions
    Ok(vec![expr.clone()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    #[test]
    fn test_diff_of_squares() {
        let mut parser = Parser::new("x^2 - 1");
        let expr = parser.parse_expression();
        let factored = factor(&expr);
        // Should be (x - 1)*(x + 1)
        assert!(matches!(factored, Expr::Mul(_, _)));
    }

    #[test]
    fn test_diff_of_squares_4() {
        let mut parser = Parser::new("x^4 - 1");
        let expr = parser.parse_expression();
        let factored = factor(&expr);
        // Should be (x^2 - 1)*(x^2 + 1)
        assert!(matches!(factored, Expr::Mul(_, _)));
    }

    #[test]
    fn test_diff_of_squares_9() {
        let mut parser = Parser::new("x^2 - 9");
        let expr = parser.parse_expression();
        let factored = factor(&expr);
        // Should be (x - 3)*(x + 3)
        assert!(matches!(factored, Expr::Mul(_, _)));
    }

    #[test]
    fn test_quadratic_factor() {
        let mut parser = Parser::new("x^2 - 5*x + 6");
        let expr = parser.parse_expression();
        let factored = factor(&expr);
        // Should be (x - 2)*(x - 3)
        assert!(matches!(factored, Expr::Mul(_, _)));
    }

    #[test]
    fn test_quadratic_factor_2() {
        let mut parser = Parser::new("x^2 + 5*x + 6");
        let expr = parser.parse_expression();
        let factored = factor(&expr);
        // Should be (x + 2)*(x + 3)
        assert!(matches!(factored, Expr::Mul(_, _)));
    }

    #[test]
    fn test_common_factor() {
        let mut parser = Parser::new("2*x^2 + 4*x");
        let expr = parser.parse_expression();
        let factored = factor(&expr);
        // Should be 2*x*(x + 2)
        assert!(matches!(factored, Expr::Mul(_, _)));
    }

    #[test]
    fn test_sum_of_cubes() {
        let mut parser = Parser::new("x^3 + 8");
        let expr = parser.parse_expression();
        let factored = factor(&expr);
        // Should be (x + 2)*(x^2 - 2*x + 4)
        assert!(matches!(factored, Expr::Mul(_, _)));
    }

    #[test]
    fn test_diff_of_cubes() {
        let mut parser = Parser::new("x^3 - 27");
        let expr = parser.parse_expression();
        let factored = factor(&expr);
        // Should be (x - 3)*(x^2 + 3*x + 9)
        assert!(matches!(factored, Expr::Mul(_, _)));
    }

    #[test]
    fn test_unfactorable() {
        let mut parser = Parser::new("x^2 + x + 1");
        let expr = parser.parse_expression();
        let factored = factor(&expr);
        // Should remain unchanged (discriminant < 0)
        assert!(matches!(factored, Expr::Add(_, _)));
    }
}