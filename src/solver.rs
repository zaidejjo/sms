use std::collections::HashMap;
use num::Complex;
use crate::expr::{Expr, evaluate, evaluate_complex};

pub struct EquationSolver {
    max_iterations: usize,
    tolerance: f64,
    search_step: f64,
    search_range: i32,
    precision_mode: bool,
}

impl EquationSolver {
    // Fast mode (performance)
    #[allow(dead_code)]
    pub fn new() -> Self {
        EquationSolver {
            max_iterations: 30,        // Fewer iterations
            tolerance: 1e-8,           // Slightly lower precision
            search_step: 0.5,          // Larger step
            search_range: 40,          // Smaller range
            precision_mode: false,
        }
    }

    // Precision mode (for complex solutions)
    #[allow(dead_code)]
    pub fn new_precision() -> Self {
        EquationSolver {
            max_iterations: 80,
            tolerance: 1e-12,
            search_step: 0.25,
            search_range: 80,
            precision_mode: true,
        }
    }

    // Adaptive mode (auto-tunes to equation)
    pub fn new_adaptive(expr: &Expr) -> Self {
        // Estimate equation complexity
        let complexity = estimate_complexity(expr);
        
        if complexity < 10 {
            // Simple equations -> very fast
            EquationSolver {
                max_iterations: 20,
                tolerance: 1e-7,
                search_step: 0.5,
                search_range: 30,
                precision_mode: false,
            }
        } else if complexity < 50 {
            // Medium complexity
            EquationSolver {
                max_iterations: 40,
                tolerance: 1e-9,
                search_step: 0.4,
                search_range: 50,
                precision_mode: false,
            }
        } else {
            // Complex equations -> high precision
            EquationSolver {
                max_iterations: 80,
                tolerance: 1e-12,
                search_step: 0.25,
                search_range: 80,
                precision_mode: true,
            }
        }
    }

    pub fn solve_from(&self, expr: &Expr, var: char, start: f64) -> Option<f64> {
        let derivative_expr = derivative(expr, var);
        let mut x = start;
        let mut prev_x = x;
        
        for i in 0..self.max_iterations {
            let mut vars = HashMap::new();
            vars.insert(var, x);
            
            let f_x = evaluate(expr, &vars);
            let df_x = evaluate(&derivative_expr, &vars);
            
            if df_x.abs() < 1e-15 {
                let h = 1e-8;
                vars.insert(var, x + h);
                let f_h = evaluate(expr, &vars);
                let df = (f_h - f_x) / h;
                if df.abs() < 1e-15 {
                    return None;
                }
                let x_new = x - f_x / df;
                if (x_new - x).abs() < self.tolerance {
                    return Some(x_new);
                }
                x = x_new;
                continue;
            }
            
            let x_new = x - f_x / df_x;
            
            // Early exit on fast convergence
            if (x_new - x).abs() < self.tolerance * 0.1 {
                return Some(x_new);
            }
            
            // Divergence detection: try different starting point
            if i > 5 && (x_new - x).abs() > (prev_x - x).abs() * 1.5 {
                return None;
            }
            
            if x_new.is_nan() || x_new.is_infinite() {
                return None;
            }
            
            prev_x = x;
            x = x_new;
        }
        
        // Final accuracy check
        let mut vars = HashMap::new();
        vars.insert(var, x);
        let f_x = evaluate(expr, &vars);
        if f_x.abs() < 1e-6 {
            Some(x)
        } else {
            None
        }
    }

    pub fn find_all_roots(&self, expr: &Expr, var: char) -> (Vec<f64>, Vec<Complex<f64>>) {
        let mut raw_real: Vec<f64> = Vec::new();
        let mut raw_complex: Vec<Complex<f64>> = Vec::new();
        
        // Adaptive smart search
        let mut starts = Vec::new();
        
        // 1. Strategic points around zero
        for i in -self.search_range..=self.search_range {
            starts.push(i as f64 * self.search_step);
        }
        
        // 2. Additional points near potential solutions
        let mut found_any = false;
        
        // Extended search in precision mode
        if self.precision_mode {
            // Use more starting points
            for i in -(self.search_range * 2)..=(self.search_range * 2) {
                let start = i as f64 * self.search_step * 0.5;
                if !starts.contains(&start) {
                    starts.push(start);
                }
            }
        }
        
        for &start in &starts {
            if let Some(root) = self.solve_from(expr, var, start) {
                let root_rounded = (root * 100000.0).round() / 100000.0;
                let mut vars = HashMap::new();
                vars.insert(var, root);
                let f_root = evaluate(expr, &vars);
                
                if f_root.abs() < 1e-6 {
                    let mut is_duplicate = false;
                    for &existing_root in &raw_real {
                        if (existing_root - root_rounded).abs() < 1e-4 {
                            is_duplicate = true;
                            break;
                        }
                    }
                    if !is_duplicate {
                        raw_real.push(root_rounded);
                        found_any = true;
                    }
                }
            }
        }
        
        // Widen search if no solutions found
        if !found_any && !self.precision_mode {
            // Try broader search
            let wider_solver = EquationSolver {
                max_iterations: 50,
                tolerance: 1e-8,
                search_step: 0.5,
                search_range: 100,
                precision_mode: true,
            };
            return wider_solver.find_all_roots(expr, var);
        }
        
        // Complex roots (only in precision mode)
        if self.precision_mode {
            for i in -15..=15 {
                for j in -15..=15 {
                    let start = Complex::new(i as f64 * 0.5, j as f64 * 0.5);
                    if let Some(root) = self.solve_complex(expr, var, start) {
                        if root.im.abs() > 1e-8 {
                            let root_rounded = Complex::new(
                                (root.re * 100000.0).round() / 100000.0,
                                (root.im * 100000.0).round() / 100000.0
                            );
                            let mut is_duplicate = false;
                            for existing_root in &raw_complex {
                                if (existing_root - root_rounded).norm() < 1e-4 {
                                    is_duplicate = true;
                                    break;
                                }
                            }
                            if !is_duplicate {
                                raw_complex.push(root_rounded);
                            }
                        }
                    }
                }
            }
        }
        
        // Cluster/filter solutions
        let final_real = self.cluster_real_roots(&raw_real);
        let final_complex = self.cluster_complex_roots(&raw_complex);
        
        (final_real, final_complex)
    }

    // Cluster real roots (optimized for speed)
    fn cluster_real_roots(&self, roots: &[f64]) -> Vec<f64> {
        if roots.is_empty() {
            return Vec::new();
        }
        
        let mut sorted = roots.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mut groups: Vec<Vec<f64>> = Vec::new();
        let threshold = if self.precision_mode { 0.001 } else { 0.01 };
        
        for &root in &sorted {
            let mut found = false;
            for group in &mut groups {
                let avg: f64 = group.iter().sum::<f64>() / group.len() as f64;
                if (root - avg).abs() < threshold {
                    group.push(root);
                    found = true;
                    break;
                }
            }
            if !found {
                groups.push(vec![root]);
            }
        }
        
        groups.iter()
            .map(|g| g.iter().sum::<f64>() / g.len() as f64)
            .collect()
    }

    // Cluster complex roots
    fn cluster_complex_roots(&self, roots: &[Complex<f64>]) -> Vec<Complex<f64>> {
        if roots.is_empty() {
            return Vec::new();
        }
        
        let mut sorted = roots.to_vec();
        sorted.sort_by(|a, b| a.re.partial_cmp(&b.re).unwrap());
        
        let mut groups: Vec<Vec<Complex<f64>>> = Vec::new();
        let threshold = if self.precision_mode { 0.001 } else { 0.01 };
        
        for &root in &sorted {
            let mut found = false;
            for group in &mut groups {
                let avg_re: f64 = group.iter().map(|c| c.re).sum::<f64>() / group.len() as f64;
                let avg_im: f64 = group.iter().map(|c| c.im).sum::<f64>() / group.len() as f64;
                let avg = Complex::new(avg_re, avg_im);
                if (root - avg).norm() < threshold {
                    group.push(root);
                    found = true;
                    break;
                }
            }
            if !found {
                groups.push(vec![root]);
            }
        }
        
        groups.iter()
            .map(|g| {
                let avg_re: f64 = g.iter().map(|c| c.re).sum::<f64>() / g.len() as f64;
                let avg_im: f64 = g.iter().map(|c| c.im).sum::<f64>() / g.len() as f64;
                Complex::new(avg_re, avg_im)
            })
            .collect()
    }

    pub fn solve_complex(&self, expr: &Expr, var: char, start: Complex<f64>) -> Option<Complex<f64>> {
        let derivative_expr = derivative(expr, var);
        let mut z = start;
        let max_iter = if self.precision_mode { 80 } else { 30 };
        
        for _ in 0..max_iter {
            let mut vars = HashMap::new();
            vars.insert(var, z);
            
            let f_z = evaluate_complex(expr, &vars);
            let df_z = evaluate_complex(&derivative_expr, &vars);
            
            if df_z.norm() < 1e-15 {
                return None;
            }
            
            let z_new = z - f_z / df_z;
            
            if (z_new - z).norm() < self.tolerance {
                return Some(z_new);
            }
            
            if z_new.re.is_nan() || z_new.im.is_nan() {
                return None;
            }
            
            z = z_new;
        }
        
        None
    }
}

// Estimate equation complexity for adaptive solver
fn estimate_complexity(expr: &Expr) -> usize {
    match expr {
        Expr::Num(_) | Expr::Var(_) => 1,
        Expr::Add(a, b) | Expr::Sub(a, b) | Expr::Mul(a, b) | Expr::Div(a, b) | Expr::Pow(a, b) => {
            estimate_complexity(a) + estimate_complexity(b)
        }
        Expr::Sin(a) | Expr::Cos(a) | Expr::Tan(a) | Expr::Asin(a) | Expr::Acos(a) | 
        Expr::Atan(a) | Expr::Sinh(a) | Expr::Cosh(a) | Expr::Tanh(a) | Expr::Ln(a) |
        Expr::Exp(a) | Expr::Sqrt(a) | Expr::Abs(a) => {
            estimate_complexity(a) + 5
        }
        Expr::Log(a, b) => estimate_complexity(a) + estimate_complexity(b) + 5,
    }
}

// Symbolic differentiation
pub fn derivative(expr: &Expr, var: char) -> Expr {
    use crate::expr::Expr;
    
    match expr {
        Expr::Num(_) => Expr::Num(0.0),
        Expr::Var(c) if *c == var => Expr::Num(1.0),
        Expr::Var(_) => Expr::Num(0.0),
        Expr::Add(a, b) => Expr::Add(Box::new(derivative(a, var)), Box::new(derivative(b, var))),
        Expr::Sub(a, b) => Expr::Sub(Box::new(derivative(a, var)), Box::new(derivative(b, var))),
        Expr::Mul(a, b) => {
            let da = derivative(a, var);
            let db = derivative(b, var);
            Expr::Add(
                Box::new(Expr::Mul(a.clone(), Box::new(db))),
                Box::new(Expr::Mul(Box::new(da), b.clone())),
            )
        }
        Expr::Div(a, b) => {
            let da = derivative(a, var);
            let db = derivative(b, var);
            Expr::Div(
                Box::new(Expr::Sub(
                    Box::new(Expr::Mul(Box::new(da), b.clone())),
                    Box::new(Expr::Mul(a.clone(), Box::new(db))),
                )),
                Box::new(Expr::Pow(b.clone(), Box::new(Expr::Num(2.0)))),
            )
        }
        Expr::Pow(a, b) => {
            match **b {
                Expr::Num(n) => {
                    if n == 0.0 { return Expr::Num(0.0); }
                    if n == 1.0 { return derivative(a, var); }
                    let new_pow = Expr::Pow(a.clone(), Box::new(Expr::Num(n - 1.0)));
                    Expr::Mul(
                        Box::new(Expr::Mul(Box::new(Expr::Num(n)), Box::new(new_pow))),
                        Box::new(derivative(a, var)),
                    )
                }
                _ => {
                    let ln_a = Expr::Ln(a.clone());
                    let term1 = Expr::Mul(Box::new(ln_a), Box::new(derivative(b, var)));
                    let term2 = Expr::Mul(
                        Box::new(Expr::Div(Box::new(derivative(a, var)), a.clone())),
                        b.clone(),
                    );
                    let sum = Expr::Add(Box::new(term1), Box::new(term2));
                    Expr::Mul(Box::new(expr.clone()), Box::new(sum))
                }
            }
        }
        Expr::Sin(a) => Expr::Mul(Box::new(Expr::Cos(a.clone())), Box::new(derivative(a, var))),
        Expr::Cos(a) => Expr::Mul(
            Box::new(Expr::Num(-1.0)),
            Box::new(Expr::Mul(Box::new(Expr::Sin(a.clone())), Box::new(derivative(a, var)))),
        ),
        Expr::Tan(a) => {
            let cos_a = Expr::Cos(a.clone());
            Expr::Div(Box::new(derivative(a, var)), Box::new(Expr::Pow(Box::new(cos_a), Box::new(Expr::Num(2.0)))))
        }
        Expr::Asin(a) => {
            let one = Expr::Num(1.0);
            let inner = Expr::Sub(Box::new(one), Box::new(Expr::Pow(a.clone(), Box::new(Expr::Num(2.0)))));
            Expr::Div(Box::new(derivative(a, var)), Box::new(Expr::Sqrt(Box::new(inner))))
        }
        Expr::Acos(a) => {
            let one = Expr::Num(1.0);
            let inner = Expr::Sub(Box::new(one), Box::new(Expr::Pow(a.clone(), Box::new(Expr::Num(2.0)))));
            Expr::Mul(Box::new(Expr::Num(-1.0)), Box::new(Expr::Div(Box::new(derivative(a, var)), Box::new(Expr::Sqrt(Box::new(inner))))))
        }
        Expr::Atan(a) => {
            let one = Expr::Num(1.0);
            let inner = Expr::Add(Box::new(one), Box::new(Expr::Pow(a.clone(), Box::new(Expr::Num(2.0)))));
            Expr::Div(Box::new(derivative(a, var)), Box::new(inner))
        }
        Expr::Sinh(a) => Expr::Mul(Box::new(Expr::Cosh(a.clone())), Box::new(derivative(a, var))),
        Expr::Cosh(a) => Expr::Mul(Box::new(Expr::Sinh(a.clone())), Box::new(derivative(a, var))),
        Expr::Tanh(a) => {
            let sech2 = Expr::Sub(
                Box::new(Expr::Num(1.0)),
                Box::new(Expr::Pow(Box::new(Expr::Tanh(a.clone())), Box::new(Expr::Num(2.0))))
            );
            Expr::Mul(Box::new(sech2), Box::new(derivative(a, var)))
        }
        Expr::Ln(a) => Expr::Div(Box::new(derivative(a, var)), a.clone()),
        Expr::Log(a, b) => {
            let one = Expr::Num(1.0);
            let ln_a = Expr::Ln(a.clone());
            let ln_b = Expr::Ln(b.clone());
            Expr::Mul(
                Box::new(Expr::Div(Box::new(derivative(a, var)), Box::new(Expr::Mul(Box::new(ln_a), Box::new(ln_b))))),
                Box::new(one)
            )
        }
        Expr::Exp(a) => Expr::Mul(Box::new(Expr::Exp(a.clone())), Box::new(derivative(a, var))),
        Expr::Sqrt(a) => {
            let two = Expr::Num(2.0);
            let sqrt_a = Expr::Sqrt(a.clone());
            Expr::Div(Box::new(derivative(a, var)), Box::new(Expr::Mul(Box::new(two), Box::new(sqrt_a))))
        }
        Expr::Abs(a) => {
            let sign = Expr::Div(a.clone(), Box::new(Expr::Abs(a.clone())));
            Expr::Mul(Box::new(sign), Box::new(derivative(a, var)))
        }
    }
}
