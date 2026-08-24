use std::fmt;
use std::ops::{Add, Sub, Mul, Div};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fraction {
    pub numerator: i64,
    pub denominator: i64,
}

impl Fraction {
    pub fn new(numerator: i64, denominator: i64) -> Self {
        if denominator == 0 {
            panic!("Denominator cannot be zero!");
        }
        let gcd = gcd(numerator.abs(), denominator.abs());
        let (n, d) = if denominator < 0 {
            (-numerator / gcd, -denominator / gcd)
        } else {
            (numerator / gcd, denominator / gcd)
        };
        Fraction { numerator: n, denominator: d }
    }

    pub fn from_f64(value: f64, tolerance: f64) -> Option<Self> {
        if value.is_nan() || value.is_infinite() {
            return None;
        }
        
        // Convert to fraction using continued fraction
        let mut h1 = 0;
        let mut h2 = 1;
        let mut k1 = 1;
        let mut k2 = 0;
        let mut x = value;
        let mut iterations = 0;
        
        while iterations < 20 {
            let a = x.floor() as i64;
            let h = a * h2 + h1;
            let k = a * k2 + k1;
            
            if k == 0 {
                break;
            }
            
            let approx = h as f64 / k as f64;
            if (value - approx).abs() < tolerance {
                return Some(Fraction::new(h, k));
            }
            
            h1 = h2;
            h2 = h;
            k1 = k2;
            k2 = k;
            
            let diff = x - a as f64;
            if diff.abs() < 1e-15 {
                break;
            }
            x = 1.0 / diff;
            iterations += 1;
        }
        
        None
    }

    #[allow(dead_code)]
    pub fn to_f64(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }

    pub fn simplify(&self) -> Self {
        let gcd = gcd(self.numerator.abs(), self.denominator.abs());
        Fraction {
            numerator: self.numerator / gcd,
            denominator: self.denominator / gcd,
        }
    }
}

fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a.abs()
}

impl fmt::Display for Fraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let simplified = self.simplify();
        if simplified.denominator == 1 {
            write!(f, "{}", simplified.numerator)
        } else if simplified.numerator == 0 {
            write!(f, "0")
        } else {
            write!(f, "{}/{}", simplified.numerator, simplified.denominator)
        }
    }
}

impl Add for Fraction {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Fraction::new(
            self.numerator * other.denominator + other.numerator * self.denominator,
            self.denominator * other.denominator,
        )
    }
}

impl Sub for Fraction {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Fraction::new(
            self.numerator * other.denominator - other.numerator * self.denominator,
            self.denominator * other.denominator,
        )
    }
}

impl Mul for Fraction {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Fraction::new(
            self.numerator * other.numerator,
            self.denominator * other.denominator,
        )
    }
}

impl Div for Fraction {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        Fraction::new(
            self.numerator * other.denominator,
            self.denominator * other.numerator,
        )
    }
}
