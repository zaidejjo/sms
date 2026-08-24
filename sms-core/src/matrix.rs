use std::fmt;

#[derive(Debug, Clone)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<Vec<f64>>,
}

impl Matrix {
    pub fn new(data: Vec<Vec<f64>>) -> Self {
        let rows = data.len();
        let cols = if rows > 0 { data[0].len() } else { 0 };
        Matrix { rows, cols, data }
    }

    pub fn solve_linear(&self, b: &[f64]) -> Option<Vec<f64>> {
        if self.rows != self.cols || self.rows != b.len() {
            return None;
        }

        let n = self.rows;
        let mut aug = Matrix {
            rows: n,
            cols: n + 1,
            data: (0..n).map(|i| {
                let mut row = self.data[i].clone();
                row.push(b[i]);
                row
            }).collect(),
        };

        // Gaussian Elimination with partial pivoting
        for col in 0..n {
            let mut pivot = col;
            for row in col..n {
                if aug.data[row][col].abs() > aug.data[pivot][col].abs() {
                    pivot = row;
                }
            }
            aug.data.swap(col, pivot);

            let pivot_val = aug.data[col][col];
            if pivot_val.abs() < 1e-15 {
                return None;
            }

            for j in col..=n {
                aug.data[col][j] /= pivot_val;
            }

            for row in 0..n {
                if row != col {
                    let factor = aug.data[row][col];
                    for j in col..=n {
                        aug.data[row][j] -= factor * aug.data[col][j];
                    }
                }
            }
        }

        Some((0..n).map(|i| aug.data[i][n]).collect())
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.data {
            for val in row {
                write!(f, "{:8.4} ", val)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub fn parse_matrix(input: &str) -> Option<Matrix> {
    let trimmed = input.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return None;
    }
    
    let inner = &trimmed[1..trimmed.len()-1];
    let rows: Vec<&str> = inner.split("],[").collect();
    let mut data = Vec::new();
    
    for row_str in rows {
        let row_str = row_str.trim_start_matches('[').trim_end_matches(']');
        let values: Vec<f64> = row_str
            .split(',')
            .filter_map(|s| s.trim().parse::<f64>().ok())
            .collect();
        if values.is_empty() {
            return None;
        }
        data.push(values);
    }
    
    if data.is_empty() {
        return None;
    }
    
    Some(Matrix::new(data))
}

pub fn parse_vector(input: &str) -> Option<Vec<f64>> {
    let trimmed = input.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return None;
    }
    
    let inner = &trimmed[1..trimmed.len()-1];
    let values: Vec<f64> = inner
        .split(',')
        .filter_map(|s| s.trim().parse::<f64>().ok())
        .collect();
    
    if values.is_empty() {
        return None;
    }
    Some(values)
}
