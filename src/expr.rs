use std::collections::HashMap;
use num::Complex;

#[derive(Debug, Clone)]
pub enum Expr {
    Num(f64),
    Var(char),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Sin(Box<Expr>),
    Cos(Box<Expr>),
    Tan(Box<Expr>),
    Asin(Box<Expr>),
    Acos(Box<Expr>),
    Atan(Box<Expr>),
    Sinh(Box<Expr>),
    Cosh(Box<Expr>),
    Tanh(Box<Expr>),
    Ln(Box<Expr>),
    Log(Box<Expr>, Box<Expr>),
    Exp(Box<Expr>),
    Sqrt(Box<Expr>),
    Abs(Box<Expr>),
}

pub fn evaluate(expr: &Expr, vars: &HashMap<char, f64>) -> f64 {
    let mut complex_vars = HashMap::new();
    for (k, v) in vars {
        complex_vars.insert(*k, Complex::new(*v, 0.0));
    }
    evaluate_complex(expr, &complex_vars).re
}

pub fn evaluate_complex(expr: &Expr, vars: &HashMap<char, Complex<f64>>) -> Complex<f64> {
    match expr {
        Expr::Num(n) => Complex::new(*n, 0.0),
        Expr::Var(c) => *vars.get(c).unwrap_or(&Complex::new(0.0, 0.0)),
        Expr::Add(a, b) => evaluate_complex(a, vars) + evaluate_complex(b, vars),
        Expr::Sub(a, b) => evaluate_complex(a, vars) - evaluate_complex(b, vars),
        Expr::Mul(a, b) => evaluate_complex(a, vars) * evaluate_complex(b, vars),
        Expr::Div(a, b) => evaluate_complex(a, vars) / evaluate_complex(b, vars),
        Expr::Pow(a, b) => evaluate_complex(a, vars).powf(evaluate_complex(b, vars).re),
        Expr::Sin(a) => evaluate_complex(a, vars).sin(),
        Expr::Cos(a) => evaluate_complex(a, vars).cos(),
        Expr::Tan(a) => evaluate_complex(a, vars).tan(),
        Expr::Asin(a) => evaluate_complex(a, vars).asin(),
        Expr::Acos(a) => evaluate_complex(a, vars).acos(),
        Expr::Atan(a) => evaluate_complex(a, vars).atan(),
        Expr::Sinh(a) => evaluate_complex(a, vars).sinh(),
        Expr::Cosh(a) => evaluate_complex(a, vars).cosh(),
        Expr::Tanh(a) => evaluate_complex(a, vars).tanh(),
        Expr::Ln(a) => evaluate_complex(a, vars).ln(),
        Expr::Log(a, b) => {
            let val = evaluate_complex(a, vars);
            let base = evaluate_complex(b, vars);
            val.ln() / base.ln()
        }
        Expr::Exp(a) => evaluate_complex(a, vars).exp(),
        Expr::Sqrt(a) => evaluate_complex(a, vars).sqrt(),
        Expr::Abs(a) => {
            let val = evaluate_complex(a, vars);
            Complex::new(val.norm(), 0.0)
        }
    }
}
