use crate::expr::Expr;
use crate::constants::CONSTANTS;

pub struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().filter(|c| !c.is_whitespace()).collect();
        Parser { chars, pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.peek();
        self.pos += 1;
        c
    }

    fn parse_number(&mut self) -> f64 {
        let mut num = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' {
                num.push(c);
                self.next();
            } else {
                break;
            }
        }
        num.parse().unwrap_or(0.0)
    }

    pub fn parse_expression(&mut self) -> Expr {
        self.parse_add_sub()
    }

    fn parse_add_sub(&mut self) -> Expr {
        let mut left = self.parse_mul_div();
        while let Some(c) = self.peek() {
            match c {
                '+' => {
                    self.next();
                    let right = self.parse_mul_div();
                    left = Expr::Add(Box::new(left), Box::new(right));
                }
                '-' => {
                    self.next();
                    let right = self.parse_mul_div();
                    left = Expr::Sub(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        left
    }

    fn parse_mul_div(&mut self) -> Expr {
        let mut left = self.parse_pow();
        while let Some(c) = self.peek() {
            match c {
                '*' => {
                    self.next();
                    let right = self.parse_pow();
                    left = Expr::Mul(Box::new(left), Box::new(right));
                }
                '/' => {
                    self.next();
                    let right = self.parse_pow();
                    left = Expr::Div(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        left
    }

    fn parse_pow(&mut self) -> Expr {
        let mut left = self.parse_unary();
        if let Some('^') = self.peek() {
            self.next();
            let right = self.parse_unary();
            left = Expr::Pow(Box::new(left), Box::new(right));
        }
        left
    }

    fn parse_unary(&mut self) -> Expr {
        if let Some('-') = self.peek() {
            self.next();
            let expr = self.parse_unary();
            return Expr::Mul(Box::new(Expr::Num(-1.0)), Box::new(expr));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Expr {
        match self.peek() {
            Some('(') => {
                self.next();
                let expr = self.parse_expression();
                if let Some(')') = self.peek() {
                    self.next();
                }
                if let Some(c) = self.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        let num = self.parse_number();
                        return Expr::Mul(Box::new(expr), Box::new(Expr::Num(num)));
                    }
                }
                expr
            }
            Some(c) if c.is_ascii_alphabetic() => {
                self.next();
                let mut name = String::new();
                name.push(c);
                while let Some(next) = self.peek() {
                    if next.is_ascii_alphabetic() || next.is_ascii_digit() || next == '_' {
                        name.push(next);
                        self.next();
                    } else {
                        break;
                    }
                }
                
                // Check if it's a constant first
                if let Some(&value) = CONSTANTS.get(&name) {
                    return Expr::Num(value);
                }
                
                match name.as_str() {
                    "sin" => { self.parse_function(Expr::Sin) }
                    "cos" => { self.parse_function(Expr::Cos) }
                    "tan" => { self.parse_function(Expr::Tan) }
                    "asin" => { self.parse_function(Expr::Asin) }
                    "acos" => { self.parse_function(Expr::Acos) }
                    "atan" => { self.parse_function(Expr::Atan) }
                    "sinh" => { self.parse_function(Expr::Sinh) }
                    "cosh" => { self.parse_function(Expr::Cosh) }
                    "tanh" => { self.parse_function(Expr::Tanh) }
                    "ln" => { self.parse_function(Expr::Ln) }
                    "exp" => { self.parse_function(Expr::Exp) }
                    "sqrt" => { self.parse_function(Expr::Sqrt) }
                    "abs" => { self.parse_function(Expr::Abs) }
                    "log" => {
                        if let Some('(') = self.peek() {
                            self.next();
                            let inner = self.parse_expression();
                            if let Some(',') = self.peek() {
                                self.next();
                                let base = self.parse_expression();
                                if let Some(')') = self.peek() {
                                    self.next();
                                }
                                return Expr::Log(Box::new(inner), Box::new(base));
                            }
                            if let Some(')') = self.peek() {
                                self.next();
                            }
                            return Expr::Log(Box::new(inner), Box::new(Expr::Num(10.0)));
                        }
                        Expr::Var('l')
                    }
                    _ => {
                        if let Some(next) = self.peek() {
                            if next.is_ascii_digit() || next == '.' {
                                let num = self.parse_number();
                                return Expr::Mul(
                                    Box::new(Expr::Var(name.chars().next().unwrap_or('x'))),
                                    Box::new(Expr::Num(num))
                                );
                            }
                        }
                        Expr::Var(name.chars().next().unwrap_or('x'))
                    }
                }
            }
            Some(c) if c.is_ascii_digit() || c == '.' => {
                let num = self.parse_number();
                if let Some(c) = self.peek() {
                    if c.is_ascii_alphabetic() && c != '(' {
                        let var = self.parse_unary();
                        return Expr::Mul(Box::new(Expr::Num(num)), Box::new(var));
                    }
                }
                Expr::Num(num)
            }
            _ => Expr::Num(0.0),
        }
    }

    fn parse_function<F>(&mut self, constructor: F) -> Expr
    where
        F: Fn(Box<Expr>) -> Expr,
    {
        if let Some('(') = self.peek() {
            self.next();
            let inner = self.parse_expression();
            if let Some(')') = self.peek() {
                self.next();
            }
            constructor(Box::new(inner))
        } else {
            Expr::Num(0.0)
        }
    }

    pub fn parse_equation(&mut self) -> Expr {
        let left = self.parse_expression();
        if let Some('=') = self.peek() {
            self.next();
            let right = self.parse_expression();
            Expr::Sub(Box::new(left), Box::new(right))
        } else {
            left
        }
    }
}
