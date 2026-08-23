use std::collections::HashMap;
use rand::Rng;
use crate::expr::{Expr, evaluate};

pub struct AISolver {
    learning_rate: f64,
    max_iterations: usize,
    tolerance: f64,
}

impl AISolver {
    pub fn new() -> Self {
        AISolver {
            learning_rate: 0.01,
            max_iterations: 10000,
            tolerance: 1e-8,
        }
    }

    pub fn solve_advanced(&self, expr: &Expr, var: char) -> (Option<f64>, f64, usize) {
        let mut rng = rand::thread_rng();
        let mut best_x = rng.gen_range(-10.0..10.0);
        let mut best_f = f64::INFINITY;
        let mut total_iterations = 0;
        
        // Try multiple random starts
        for _ in 0..10 {
            let mut x = rng.gen_range(-10.0..10.0);
            
            for i in 0..self.max_iterations / 10 {
                total_iterations += 1;
                let mut vars = HashMap::new();
                vars.insert(var, x);
                let f_x = evaluate(expr, &vars);
                
                if f_x.abs() < best_f {
                    best_f = f_x.abs();
                    best_x = x;
                }
                
                if f_x.abs() < self.tolerance {
                    return (Some(x), f_x.abs(), total_iterations);
                }
                
                // Numerical gradient
                let h = 1e-8;
                vars.insert(var, x + h);
                let f_plus = evaluate(expr, &vars);
                let gradient = (f_plus - f_x) / h;
                
                // Adaptive learning rate
                let lr = self.learning_rate / (1.0 + i as f64 * 0.001);
                x -= lr * gradient * f_x;
                
                if x.is_nan() || x.is_infinite() {
                    break;
                }
            }
        }
        
        if best_f < 1e-6 {
            (Some(best_x), best_f, total_iterations)
        } else {
            (None, best_f, total_iterations)
        }
    }
}
