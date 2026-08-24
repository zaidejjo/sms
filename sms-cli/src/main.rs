use sms_core::{Expr, evaluate, evaluate_complex, Parser, EquationSolver, SeriesOp, compute_series, AISolver, Fraction, ExportData};
use sms_core::matrix::{parse_matrix, parse_vector};
use std::io;
use std::time::Instant;
use std::collections::HashMap;
use std::process::Command;
use plotters::prelude::*;

// 🔥 تاريخ المعادلات (للتنقل بالأسهم)
struct History {
    entries: Vec<String>,
    current: usize,
}

impl History {
    fn new() -> Self {
        History {
            entries: Vec::new(),
            current: 0,
        }
    }

    fn add(&mut self, entry: String) {
        self.entries.push(entry);
        self.current = self.entries.len();
    }

    fn previous(&mut self) -> Option<&str> {
        if self.current > 0 {
            self.current -= 1;
            self.entries.get(self.current).map(|s| s.as_str())
        } else {
            None
        }
    }

    fn next(&mut self) -> Option<&str> {
        if self.current + 1 < self.entries.len() {
            self.current += 1;
            self.entries.get(self.current).map(|s| s.as_str())
        } else {
            None
        }
    }

}

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

// 🔥 تحويل رقم إلى كسر إذا كان بسيطاً
fn to_fraction_str(value: f64) -> String {
    let value_rounded = (value * 100000.0).round() / 100000.0;
    if let Some(fraction) = Fraction::from_f64(value_rounded, 1e-6) {
        if fraction.denominator > 1 && fraction.denominator <= 100 {
            return fraction.to_string();
        }
    }
    format_number(value)
}

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
    println!("  export filename     - Export results to JSON/CSV/LATEX");
    println!("  clear / cls         - Clear screen");
    println!("  history             - Show history");
    println!("  ↑ / ↓               - Navigate history");
    println!("  help                - Show help");
    println!("  quit                - Exit");
    println!();
}

fn main() {
    println!("SMS - Smart Math Solver");
    println!("Features: Constants, Fractions, Export, History");
    println!("Type 'help' for commands");
    println!();
    
    let mut history = History::new();
    let mut last_export_data: Option<ExportData> = None;
    
    loop {
        print!("> ");
        io::Write::flush(&mut io::stdout()).unwrap();
        
        let mut input = String::new();
        let bytes_read = io::stdin().read_line(&mut input).unwrap();
        
        // Handle EOF (piping input)
        if bytes_read == 0 {
            break;
        }
        
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        // 🔥 تاريخ المعادلات
        history.add(input.to_string());
        
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
            "history" => {
                for (i, entry) in history.entries.iter().enumerate() {
                    println!("  {}. {}", i+1, entry);
                }
                continue;
            }
            "export" => {
                if let Some(data) = &last_export_data {
                    // Export last result
                    println!("  Exporting to result.json, result.csv, result.tex");
                    let _ = data.export_json("result.json");
                    let _ = data.export_csv("result.csv");
                    let _ = data.export_latex("result.tex");
                    println!("  Exported!");
                } else {
                    println!("  No results to export!");
                }
                continue;
            }
            _ => {}
        }
        
        // 🔥 معالجة الأسهم (↑/↓) - نافذة تفاعلية بسيطة
        if input == "↑" || input == "up" {
            if let Some(prev) = history.previous() {
                println!("  {}", prev);
            }
            continue;
        }
        if input == "↓" || input == "down" {
            if let Some(next) = history.next() {
                println!("  {}", next);
            }
            continue;
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
                                println!("  x{} = {}", i+1, to_fraction_str(*val));
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
                println!("  = {}  (fraction: {})", 
                    format_number(result), 
                    to_fraction_str(result));
            } else {
                println!("  Usage: sum i^2, i=1..10");
            }
            continue;
        }
        
        // Product series
        if input.starts_with("product ") {
            let rest = &input[8..];
            if let Some(result) = parse_series(rest, SeriesOp::Product) {
                println!("  = {}  (fraction: {})", 
                    format_number(result), 
                    to_fraction_str(result));
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
                println!("  {} = {}  (error: {:.2e}, iter: {})", 
                    var, to_fraction_str(x), f_x.abs(), iterations);
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
        
        // 🔥 Regular equation solver
        let start = Instant::now();
        
        let mut parser = Parser::new(input);
        let expr = parser.parse_equation();
        
        let mut vars = Vec::new();
        detect_variables(&expr, &mut vars);
        
        if vars.is_empty() {
            let empty_vars = HashMap::new();
            let result = evaluate(&expr, &empty_vars);
            let elapsed = start.elapsed();
            println!("  = {}  (fraction: {})  (time: {:.3}ms)", 
                format_number(result),
                to_fraction_str(result),
                elapsed.as_secs_f64() * 1000.0);
            continue;
        }
        
        let var = vars[0];
        let solver = EquationSolver::new_adaptive(&expr);
        let (real_roots, complex_roots) = solver.find_all_roots(&expr, var);
        
        let elapsed = start.elapsed();
        
        // 🔥 حفظ للتصدير
        let mut export_data = ExportData::new(input.to_string(), var.to_string(), elapsed.as_secs_f64() * 1000.0);
        
        if real_roots.is_empty() && complex_roots.is_empty() {
            println!("  No solutions  (time: {:.3}ms)", elapsed.as_secs_f64() * 1000.0);
        } else {
            for (i, root) in real_roots.iter().enumerate() {
                let mut vars_map = HashMap::new();
                vars_map.insert(var, *root);
                let f_root = evaluate(&expr, &vars_map);
                println!("  {}. {} = {}  (error: {:.2e})", 
                    i+1, var, to_fraction_str(*root), f_root.abs());
                export_data.add_solution(*root, f_root.abs());
            }
            for (i, root) in complex_roots.iter().enumerate() {
                let real_str = to_fraction_str(root.re);
                let imag_str = to_fraction_str(root.im.abs());
                let sign = if root.im >= 0.0 { "+" } else { "-" };
                let mut vars_map = HashMap::new();
                vars_map.insert(var, *root);
                let f_root = evaluate_complex(&expr, &vars_map);
                println!("  {}. {} = {} {} {}i  (error: {:.2e})", 
                         i + real_roots.len() + 1, var, real_str, sign, imag_str, f_root.norm());
                export_data.add_complex(root.re, root.im, f_root.norm());
            }
            let total = real_roots.len() + complex_roots.len();
            if total > 1 {
                println!("  {} solutions found", total);
            } else {
                println!("  1 solution found");
            }
            println!("  Time: {:.3}ms", elapsed.as_secs_f64() * 1000.0);
            
            // 🔥 حفظ آخر تصدير
            last_export_data = Some(export_data);
            println!("  Type 'export' to save results");
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

    chart.draw_series(LineSeries::new(points.clone(), &RED))?
        .label("f(x)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart.configure_series_labels()
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    println!("  Plot saved to plot.png");
    
    // Also render ASCII plot in terminal
    render_ascii_plot(&points, x_min, x_max, var);
    
    Ok(())
}

/// Render a simple ASCII plot in the terminal
fn render_ascii_plot(points: &[(f64, f64)], x_min: f64, x_max: f64, var: char) {
    if points.is_empty() {
        println!("  No points to plot");
        return;
    }
    
    let y_vals: Vec<f64> = points.iter().map(|(_, y)| *y).collect();
    let y_min = y_vals.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let y_max = y_vals.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    let width = 80;
    let height = 20;
    
    // Create a grid
    let mut grid = vec![vec![' '; width]; height];
    
    // Map points to grid
    for &(x, y) in points {
        if !x.is_finite() || !y.is_finite() {
            continue;
        }
        let col = if x_max != x_min {
            ((x - x_min) / (x_max - x_min) * (width - 1) as f64).round() as usize
        } else {
            width / 2
        };
        let row = if y_max != y_min {
            ((y_max - y) / (y_max - y_min) * (height - 1) as f64).round() as usize
        } else {
            height / 2
        };
        
        if col < width && row < height {
            grid[row][col] = '●';
        }
    }
    
    // Draw axes
    let zero_row = if y_max != y_min {
        ((y_max - 0.0) / (y_max - y_min) * (height - 1) as f64).round() as usize
    } else {
        height / 2
    };
    let zero_col = if x_max != x_min {
        ((0.0 - x_min) / (x_max - x_min) * (width - 1) as f64).round() as usize
    } else {
        width / 2
    };
    
    // Draw Y axis
    if zero_col < width {
        for r in 0..height {
            if grid[r][zero_col] == ' ' {
                grid[r][zero_col] = '│';
            }
        }
    }
    
    // Draw X axis
    if zero_row < height {
        for c in 0..width {
            if grid[zero_row][c] == ' ' {
                grid[zero_row][c] = '─';
            }
        }
    }
    
    // Origin
    if zero_row < height && zero_col < width {
        grid[zero_row][zero_col] = '┼';
    }
    
    // Print Y-axis labels and grid
    println!("  f({}) from {:.2} to {:.2}", var, x_min, x_max);
    
    // Y-axis labels
    for (i, row) in grid.iter().enumerate().rev() {
        let _y_val = y_max - (i as f64 / (height - 1) as f64) * (y_max - y_min);
        let label = if i == 0 {
            format!("{:.2}", y_max)
        } else if i == height - 1 {
            format!("{:.2}", y_min)
        } else {
            "".to_string()
        };
        
        let line: String = row.iter().collect();
        if !label.is_empty() {
            println!(" {:>6} │{}", label, line);
        } else {
            println!("       │{}", line);
        }
    }
    
    // X-axis labels
    print!("       └");
    for i in 0..width {
        if i % 10 == 0 {
            let x_val = x_min + (i as f64 / (width - 1) as f64) * (x_max - x_min);
            print!("{:.1}", x_val);
        } else {
            print!("─");
        }
    }
    println!();
}