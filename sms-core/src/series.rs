use std::collections::HashMap;
use crate::expr::{Expr, evaluate};

pub enum SeriesOp {
    Sum,
    Product,
}

pub fn compute_series(
    expr: &Expr, 
    var: char, 
    start: i64, 
    end: i64, 
    op: SeriesOp
) -> f64 {
    let mut result = match op {
        SeriesOp::Sum => 0.0,
        SeriesOp::Product => 1.0,
    };
    
    for i in start..=end {
        let mut vars = HashMap::new();
        vars.insert(var, i as f64);
        let val = evaluate(expr, &vars);
        
        match op {
            SeriesOp::Sum => result += val,
            SeriesOp::Product => result *= val,
        }
    }
    
    result
}
