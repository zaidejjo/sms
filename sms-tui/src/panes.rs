//! TUI Panes - Input, Solutions, Plot, History

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use sms_core::{evaluate, evaluate_complex, Expr};
use std::collections::HashMap;
use num::Complex;

mod fractions {
    use num::rational::Ratio;

    pub fn format_fraction(value: f64) -> String {
        let ratio = Ratio::approximate_float(value).unwrap_or_else(|| Ratio::new(value as i64, 1));
        if ratio.denom() == &1 {
            ratio.numer().to_string()
        } else {
            format!("{}/{}", ratio.numer(), ratio.denom())
        }
    }
}

pub struct InputPane {
    pub buffer: String,
    pub cursor: usize,
    pub focused: bool,
}

impl InputPane {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
            focused: false,
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let title = if self.focused { " Equation (Enter to solve) " } else { " Equation " };
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if self.focused { Color::Yellow } else { Color::Gray }));

        let input = Paragraph::new(self.buffer.as_str())
            .block(block)
            .wrap(Wrap { trim: false });

        f.render_widget(input, area);

        if self.focused {
            f.set_cursor_position((
                area.x + 1 + self.cursor as u16,
                area.y + 1,
            ));
        }
    }

    pub fn handle_input(&mut self, action: crate::keys::Action) {
        match action {
            crate::keys::Action::InputChar(c) => {
                self.buffer.insert(self.cursor, c);
                self.cursor += 1;
            }
            crate::keys::Action::InputBackspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.buffer.remove(self.cursor);
                }
            }
            crate::keys::Action::InputLeft => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            crate::keys::Action::InputRight => {
                if self.cursor < self.buffer.len() {
                    self.cursor += 1;
                }
            }
            crate::keys::Action::InputHome => {
                self.cursor = 0;
            }
            crate::keys::Action::InputEnd => {
                self.cursor = self.buffer.len();
            }
            _ => {}
        }
    }
}

pub struct SolutionsPane {
    pub solutions: Vec<SolutionDisplay>,
    pub state: ListState,
    pub focused: bool,
}

#[derive(Clone)]
pub struct SolutionDisplay {
    pub index: usize,
    pub variable: char,
    pub value: String,
    pub error: f64,
    pub is_complex: bool,
}

impl SolutionsPane {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            solutions: Vec::new(),
            state,
            focused: false,
        }
    }

    pub fn set_solutions(&mut self, variable: char, real: &[f64], complex: &[Complex<f64>], expr: &Expr) {
        self.solutions.clear();
        let mut idx = 1;

        for &r in real {
            let mut vars = HashMap::new();
            vars.insert(variable, r);
            let err = evaluate(expr, &vars).abs();
            let frac = fractions::format_fraction(r);
            self.solutions.push(SolutionDisplay {
                index: idx,
                variable,
                value: if frac != r.to_string() { format!("{} ({})", r, frac) } else { r.to_string() },
                error: err,
                is_complex: false,
            });
            idx += 1;
        }

        for c in complex {
            let mut vars = HashMap::new();
            vars.insert(variable, *c);
            let err = evaluate_complex(expr, &vars).norm();
            let re_frac = fractions::format_fraction(c.re);
            let im_frac = fractions::format_fraction(c.im.abs());
            let sign = if c.im >= 0.0 { "+" } else { "-" };
            self.solutions.push(SolutionDisplay {
                index: idx,
                variable,
                value: format!("{} {} {}i", re_frac, sign, im_frac),
                error: err,
                is_complex: true,
            });
            idx += 1;
        }

        self.state.select(Some(0));
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let title = if self.focused { " Solutions (↑↓ to navigate) " } else { " Solutions " };
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if self.focused { Color::Yellow } else { Color::Gray }));

        let items: Vec<ListItem> = if self.solutions.is_empty() {
            vec![ListItem::new("No solutions found").style(Style::default().fg(Color::Gray))]
        } else {
            self.solutions.iter().map(|s| {
                let content = format!("{}. {} = {}  (err: {:.2e})", s.index, s.variable, s.value, s.error);
                ListItem::new(content)
            }).collect()
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, area, &mut self.state);
    }

    pub fn next(&mut self) {
        let i = self.state.selected().unwrap_or(0);
        if i + 1 < self.solutions.len() {
            self.state.select(Some(i + 1));
        }
    }

    pub fn prev(&mut self) {
        let i = self.state.selected().unwrap_or(0);
        if i > 0 {
            self.state.select(Some(i - 1));
        }
    }
}

pub struct PlotPane {
    pub data: Vec<(f64, f64)>,
    pub x_range: (f64, f64),
    pub focused: bool,
}

impl PlotPane {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            x_range: (-10.0, 10.0),
            focused: false,
        }
    }

    pub fn update(&mut self, expr: &Expr, var: char, x_min: f64, x_max: f64) {
        self.x_range = (x_min, x_max);
        self.data.clear();
        let steps = 200;
        let step = (x_max - x_min) / steps as f64;
        for i in 0..=steps {
            let x = x_min + i as f64 * step;
            let mut vars = HashMap::new();
            vars.insert(var, x);
            let y = evaluate(expr, &vars);
            if y.is_finite() && y.abs() < 100.0 {
                self.data.push((x, y));
            }
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let title = if self.focused { " Plot " } else { " Plot " };
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if self.focused { Color::Yellow } else { Color::Gray }));

        if self.data.is_empty() {
            let placeholder = Paragraph::new("No plot data\nEnter equation and press 'p' to plot")
                .block(block)
                .style(Style::default().fg(Color::Gray));
            f.render_widget(placeholder, area);
            return;
        }

        let x_min = self.x_range.0;
        let x_max = self.x_range.1;
        let y_vals: Vec<f64> = self.data.iter().map(|(_, y)| *y).collect();
        let y_min = y_vals.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let y_max = y_vals.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        let plot = ratatui::widgets::canvas::Canvas::default()
            .block(block)
            .x_bounds([x_min, x_max])
            .y_bounds([y_min, y_max])
            .paint(|ctx| {
                ctx.draw(&ratatui::widgets::canvas::Points {
                    coords: &self.data,
                    color: Color::Cyan,
                });
            });

        f.render_widget(plot, area);
    }
}

pub struct HistoryPane {
    pub entries: Vec<String>,
    pub state: ListState,
    pub focused: bool,
}

impl HistoryPane {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            entries: Vec::new(),
            state,
            focused: false,
        }
    }

    pub fn add(&mut self, entry: String) {
        self.entries.push(entry);
        self.state.select(Some(self.entries.len() - 1));
    }

    pub fn load_from_db(&mut self, entries: Vec<String>) {
        self.entries = entries;
        if !self.entries.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let title = if self.focused { " History (↑↓ to navigate, Enter to reuse) " } else { " History " };
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if self.focused { Color::Yellow } else { Color::Gray }));

        let items: Vec<ListItem> = if self.entries.is_empty() {
            vec![ListItem::new("No history").style(Style::default().fg(Color::Gray))]
        } else {
            self.entries.iter().rev().enumerate().map(|(i, e)| {
                ListItem::new(format!("{}. {}", i + 1, e))
            }).collect()
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, area, &mut self.state);
    }

    pub fn next(&mut self) {
        let i = self.state.selected().unwrap_or(0);
        if i + 1 < self.entries.len() {
            self.state.select(Some(i + 1));
        }
    }

    pub fn prev(&mut self) {
        let i = self.state.selected().unwrap_or(0);
        if i > 0 {
            self.state.select(Some(i - 1));
        }
    }

    pub fn selected(&self) -> Option<&String> {
        self.state.selected().and_then(|i| self.entries.get(self.entries.len() - 1 - i))
    }
}