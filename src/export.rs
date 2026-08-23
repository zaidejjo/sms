use std::fs::File;
use std::io::Write;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData {
    pub equation: String,
    pub variable: String,
    pub solutions: Vec<Solution>,
    pub time_ms: f64,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Solution {
    pub value: f64,
    pub is_complex: bool,
    pub real: Option<f64>,
    pub imag: Option<f64>,
    pub error: f64,
}

impl ExportData {
    pub fn new(equation: String, variable: String, time_ms: f64) -> Self {
        ExportData {
            equation,
            variable,
            solutions: Vec::new(),
            time_ms,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }

    pub fn add_solution(&mut self, value: f64, error: f64) {
        self.solutions.push(Solution {
            value,
            is_complex: false,
            real: Some(value),
            imag: None,
            error,
        });
    }

    pub fn add_complex(&mut self, real: f64, imag: f64, error: f64) {
        self.solutions.push(Solution {
            value: 0.0,
            is_complex: true,
            real: Some(real),
            imag: Some(imag),
            error,
        });
    }

    pub fn export_json(&self, filename: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        let mut file = File::create(filename).map_err(|e| e.to_string())?;
        file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn export_csv(&self, filename: &str) -> Result<(), String> {
        let mut content = String::new();
        content.push_str("Equation,Variable,Solution,Real,Imaginary,Error\n");
        
        for sol in &self.solutions {
            let line = if sol.is_complex {
                format!(
                    "{},{},{:.6},{:.6},{:.6},{:.2e}\n",
                    self.equation,
                    self.variable,
                    "complex",
                    sol.real.unwrap_or(0.0),
                    sol.imag.unwrap_or(0.0),
                    sol.error
                )
            } else {
                format!(
                    "{},{},{:.6},{:.6},{:.6},{:.2e}\n",
                    self.equation,
                    self.variable,
                    "real",
                    sol.real.unwrap_or(0.0),
                    0.0,
                    sol.error
                )
            };
            content.push_str(&line);
        }
        
        let mut file = File::create(filename).map_err(|e| e.to_string())?;
        file.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn export_latex(&self, filename: &str) -> Result<(), String> {
        let mut content = String::new();
        content.push_str("\\documentclass{article}\n");
        content.push_str("\\begin{document}\n");
        content.push_str(&format!("\\section{{Solutions for: {}}}\n", self.equation));
        content.push_str(&format!("Variable: {}\n\n", self.variable));
        content.push_str("\\begin{itemize}\n");
        
        for sol in &self.solutions {
            if sol.is_complex {
                content.push_str(&format!(
                    "\\item $x = {:.6} + {:.6}i$ (error: {:.2e})\n",
                    sol.real.unwrap_or(0.0),
                    sol.imag.unwrap_or(0.0),
                    sol.error
                ));
            } else {
                content.push_str(&format!(
                    "\\item $x = {:.6}$ (error: {:.2e})\n",
                    sol.value,
                    sol.error
                ));
            }
        }
        
        content.push_str("\\end{itemize}\n");
        content.push_str(&format!("Time: {:.3}ms\n", self.time_ms));
        content.push_str("\\end{document}\n");
        
        let mut file = File::create(filename).map_err(|e| e.to_string())?;
        file.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
        Ok(())
    }
}
