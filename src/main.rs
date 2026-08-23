mod expr;
mod parser;
mod solver;
mod matrix;
mod series;
mod ai;

use std::io;
use std::time::Instant;
use std::collections::HashMap;
use std::process::Command;
use crate::expr::{Expr, evaluate, evaluate_complex};
use crate::parser::Parser;
use crate::solver::EquationSolver;
use crate::matrix::{parse_matrix, parse_vector};
use crate::series::{compute_series, SeriesOp};
use crate::ai::AISolver;

fn detect_variables(expr: &Expr, vars: &mut Vec<char>) {
    match expr {
        Expr::Var(c) if !vars.contains(c) => vars.push(*c),
        Expr::Add(a, b) | Expr::Sub(a, b) | Expr::Mul(a, b) | Expr::Div(a, b) | 
        Expr::Pow(a, b) | Expr::Log(a, b) => {
            detect_variables(a, vars);
            detect_variables(b, vars);
        }
        Expr::Sin(a) | Expr::Cos(a) | Expr::Tan(a) | Expr::Asin(a) | Expr::Acos(a) | 
        Expr::Atan(a) | Expr::Sinh(a) | Expr::Cosh(a) | Expr::Tanh(a) | Expr::Ln(a) |
        Expr::Exp(a) | Expr::Sqrt(a) | Expr::Abs(a) => {
            detect_variables(a, vars);
        }
        _ => {}
    }
}

// 🔥 دالة لتنسيق الأعداد بشكل ذكي
fn format_number(n: f64) -> String {
    let s = format!("{:.6}", n);
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() {
        "0".to_string()
    } else if trimmed == "-0" {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

// 🔥 دالة لتنظيف الشاشة
fn clear_screen() {
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/c", "cls"]).status().unwrap();
    } else {
        Command::new("clear").status().unwrap();
    }
}

fn print_help() {
    println!("Commands:");
    println!("  <equation>          - Solve equation");
    println!("  matrix A b          - Solve linear system");
    println!("  sum expr, var=a..b  - Sum series");
    println!("  product expr, a..b  - Product series");
    println!("  ai <equation>       - AI solver");
    println!("  plot var,min,max    - Plot function");
    println!("  clear / cls         - Clear screen");
    println!("  help                - Show help");
    println!("  quit                - Exit");
    println!();
}

fn main() {
    // 🔥 اختيار وضع الحل حسب النظام
    let solver_mode = if cfg!(debug_assertions) {
        "Debug (slower)"
    } else {
        "Release (fast)"
    };
    
    println!("Smart Math Solver v7.0 (Ultra Fast)");
    println!("Mode: {}", solver_mode);
    println!("Type 'help' for commands");
    println!();
    
    loop {
        print!("> ");
        io::Write::flush(&mut io::stdout()).unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        match input {
            "quit" | "exit" => {
                println!("Goodbye!");
                break;
            }
            "clear" | "cls" => {
                clear_screen();
                continue;
            }
            "help" => {
                print_help();
                continue;
            }
            _ => {}
        }
        
        // Matrix solver
        if input.starts_with("matrix ") {
            let rest = &input[7..].trim();
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() == 2 {
                if let Some(matrix) = parse_matrix(parts[0]) {
                    if let Some(vector) = parse_vector(parts[1]) {
                        if let Some(solution) = matrix.solve_linear(&vector) {
                            for (i, val) in solution.iter().enumerate() {
                                println!("  x{} = {}", i+1, format_number(*val));
                            }
                        } else {
                            println!("  No solution");
                        }
                    } else {
                        println!("  Invalid vector");
                    }
                } else {
                    println!("  Invalid matrix");
                }
            } else {
                println!("  Usage: matrix [[2,3],[4,-1]] [8,6]");
            }
            continue;
        }
        
        // Sum series
        if input.starts_with("sum ") {
            let rest = &input[4..];
            if let Some(result) = parse_series(rest, SeriesOp::Sum) {
                println!("  = {}", format_number(result));
            } else {
                println!("  Usage: sum i^2, i=1..10");
            }
            continue;
        }
        
        // Product series
        if input.starts_with("product ") {
            let rest = &input[8..];
            if let Some(result) = parse_series(rest, SeriesOp::Product) {
                println!("  = {}", format_number(result));
            } else {
                println!("  Usage: product i, i=1..5");
            }
            continue;
        }
        
        // AI solver
        if input.starts_with("ai ") {
            let rest = &input[3..].trim();
            let mut parser = Parser::new(rest);
            let expr = parser.parse_equation();
            
            let mut vars = Vec::new();
            detect_variables(&expr, &mut vars);
            
            if vars.is_empty() {
                println!("  No variable found");
                continue;
            }
            
            let var = vars[0];
            let ai = AISolver::new();
            let (solution, error, iterations) = ai.solve_advanced(&expr, var);
            
            if let Some(x) = solution {
                let mut vars_map = HashMap::new();
                vars_map.insert(var, x);
                let f_x = evaluate(&expr, &vars_map);
                println!("  {} = {}  (error: {:.2e}, iter: {})", var, format_number(x), f_x.abs(), iterations);
            } else {
                println!("  No solution found (error: {:.2e})", error);
            }
            continue;
        }
        
        // Plot command
        if input.starts_with("plot ") {
            let rest = &input[5..];
            let parts: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
            if parts.len() == 3 {
                let var = parts[0].chars().next().unwrap_or('x');
                let x_min: f64 = parts[1].parse().unwrap_or(-10.0);
                let x_max: f64 = parts[2].parse().unwrap_or(10.0);
                
                let expr = Expr::Var(var);
                if let Err(e) = plot_function(&expr, var, x_min, x_max) {
                    println!("  Plot error: {}", e);
                }
            } else {
                println!("  Usage: plot x,-5,5");
            }
            continue;
        }
        
        // 🔥 Regular equation solver with adaptive speed
        let start = Instant::now();
        
        let mut parser = Parser::new(input);
        let expr = parser.parse_equation();
        
        let mut vars = Vec::new();
        detect_variables(&expr, &mut vars);
        
        if vars.is_empty() {
            let empty_vars = HashMap::new();
            let result = evaluate(&expr, &empty_vars);
            let elapsed = start.elapsed();
            println!("  = {}  (time: {:.3}ms)", format_number(result), elapsed.as_secs_f64() * 1000.0);
            continue;
        }
        
        let var = vars[0];
        
        // 🔥 اختيار الوضع المناسب تلقائياً
        let solver = EquationSolver::new_adaptive(&expr);
        let (real_roots, complex_roots) = solver.find_all_roots(&expr, var);
        
        let elapsed = start.elapsed();
        
        if real_roots.is_empty() && complex_roots.is_empty() {
            println!("  No solutions  (time: {:.3}ms)", elapsed.as_secs_f64() * 1000.0);
        } else {
            for (i, root) in real_roots.iter().enumerate() {
                println!("  {}. {} = {}", i+1, var, format_number(*root));
            }
            for (i, root) in complex_roots.iter().enumerate() {
                let real_str = format_number(root.re);
                let imag_str = format_number(root.im.abs());
                let sign = if root.im >= 0.0 { "+" } else { "-" };
                println!("  {}. {} = {} {} {}i", 
                         i + real_roots.len() + 1, var, real_str, sign, imag_str);
            }
            let total = real_roots.len() + complex_roots.len();
            if total > 1 {
                println!("  {} solutions found", total);
            } else {
                println!("  1 solution found");
            }
            println!("  Time: {:.3}ms", elapsed.as_secs_f64() * 1000.0);
        }
    }
}

fn parse_series(input: &str, op: SeriesOp) -> Option<f64> {
    let parts: Vec<&str> = input.split(',').collect();
    if parts.len() != 2 {
        return None;
    }
    
    let expr_str = parts[0].trim();
    let range_str = parts[1].trim();
    let range_parts: Vec<&str> = range_str.split('=').collect();
    if range_parts.len() != 2 {
        return None;
    }
    
    let var = range_parts[0].trim().chars().next()?;
    let bounds: Vec<&str> = range_parts[1].trim().split("..").collect();
    if bounds.len() != 2 {
        return None;
    }
    
    let start: i64 = bounds[0].trim().parse().ok()?;
    let end: i64 = bounds[1].trim().parse().ok()?;
    
    let mut parser = Parser::new(expr_str);
    let expr = parser.parse_expression();
    
    Some(compute_series(&expr, var, start, end, op))
}

// Plotting function
use plotters::prelude::*;

fn plot_function(expr: &Expr, var: char, x_min: f64, x_max: f64) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("plot.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("f({})", var), ("sans-serif", 24))
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(x_min..x_max, -10.0..10.0)?;

    chart.configure_mesh().draw()?;

    let num_points = 1000;
    let step = (x_max - x_min) / num_points as f64;
    let mut points: Vec<(f64, f64)> = Vec::new();
    
    for i in 0..=num_points {
        let x = x_min + i as f64 * step;
        let mut vars = HashMap::new();
        vars.insert(var, x);
        let y = evaluate(expr, &vars);
        if y.is_finite() && y.abs() < 100.0 {
            points.push((x, y));
        }
    }

    chart.draw_series(LineSeries::new(points, &RED))?
        .label("f(x)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart.configure_series_labels()
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    println!("  Plot saved to plot.png");
    
    Ok(())
}
