//! Main TUI Application

use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use plotters::prelude::*;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;
use std::collections::HashMap;

use crate::{
    panes::{InputPane, SolutionsPane, PlotPane, HistoryPane},
    keys::{KeyHandler, Action},
    config::Config,
    history::HistoryDB,
};
use sms_core::{Parser, evaluate, EquationSolver, Expr};

pub struct App {
    input: InputPane,
    solutions: SolutionsPane,
    plot: PlotPane,
    history: HistoryPane,
    key_handler: KeyHandler,
    config: Config,
    db: HistoryDB,
    current_pane: PaneFocus,
    last_expr: Option<Expr>,
    last_var: Option<char>,
    status: String,
    should_quit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PaneFocus {
    Input,
    Solutions,
    Plot,
    History,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load();
        let key_handler = KeyHandler::new(&config.keybindings);
        let db = HistoryDB::new()?;
        let history_entries = db.get_all(100).unwrap_or_default();

        let mut history = HistoryPane::new();
        history.load_from_db(history_entries);

        Ok(Self {
            input: InputPane::new(),
            solutions: SolutionsPane::new(),
            plot: PlotPane::new(),
            history,
            key_handler,
            config,
            db,
            current_pane: PaneFocus::Input,
            last_expr: None,
            last_var: None,
            status: String::new(),
            should_quit: false,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.clear()?;

        while !self.should_quit {
            terminal.draw(|f| self.render(f))?;
            self.handle_events()?;
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        Ok(())
    }

    fn render(&mut self, f: &mut ratatui::Frame) {
        let area = f.area();

        // Layout: top bar, main area (4 panes), bottom status
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(3),  // Title bar
                ratatui::layout::Constraint::Min(20),    // Main panes
                ratatui::layout::Constraint::Length(3),  // Status bar
            ])
            .split(area);

        // Title bar
        self.render_title(f, chunks[0]);

        // Main panes (2x2 grid)
        self.render_main_panes(f, chunks[1]);

        // Status bar
        self.render_status(f, chunks[2]);
    }

    fn render_title(&self, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let title = ratatui::widgets::Paragraph::new(" SMS - Smart Math Solver ")
            .style(ratatui::style::Style::default()
                .fg(ratatui::style::Color::Magenta)
                .add_modifier(ratatui::style::Modifier::BOLD))
            .alignment(ratatui::layout::Alignment::Center)
            .block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::ALL));
        f.render_widget(title, area);
    }

    fn render_main_panes(&mut self, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let horizontal = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Percentage(50),
                ratatui::layout::Constraint::Percentage(50),
            ])
            .split(area);

        let left_vertical = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(5),  // Input
                ratatui::layout::Constraint::Min(10),    // Solutions
            ])
            .split(horizontal[0]);

        let right_vertical = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Percentage(50), // Plot
                ratatui::layout::Constraint::Percentage(50), // History
            ])
            .split(horizontal[1]);

        // Update focus states
        self.input.focused = self.current_pane == PaneFocus::Input;
        self.solutions.focused = self.current_pane == PaneFocus::Solutions;
        self.plot.focused = self.current_pane == PaneFocus::Plot;
        self.history.focused = self.current_pane == PaneFocus::History;

        // Render panes
        self.input.render(f, left_vertical[0]);
        self.solutions.render(f, left_vertical[1]);
        self.plot.render(f, right_vertical[0]);
        self.history.render(f, right_vertical[1]);
    }

    fn render_status(&self, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let status_text = if self.status.is_empty() {
            "Tab: Switch pane | Enter: Solve | p: Plot | e: Export | c: Clear | q: Quit | ?: Help"
        } else {
            &self.status
        };

        let status = ratatui::widgets::Paragraph::new(status_text)
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Gray))
            .block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::ALL));
        f.render_widget(status, area);
    }

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                let action = self.key_handler.handle(key);
                self.handle_action(action)?;
            }
        }
        Ok(())
    }

    fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => {
                self.should_quit = true;
            }
            Action::Help => {
                self.show_help();
            }
            Action::Solve => {
                self.solve_equation()?;
            }
            Action::Plot => {
                self.plot_function()?;
            }
            Action::Export => {
                self.export_results()?;
            }
            Action::ExportPlot => {
                self.export_plot()?;
            }
            Action::HistoryUp => {
                match self.current_pane {
                    PaneFocus::Solutions => self.solutions.prev(),
                    PaneFocus::History => self.history.prev(),
                    PaneFocus::Input => {
                        if let Some(entry) = self.history.selected() {
                            self.input.buffer = entry.clone();
                            self.input.cursor = entry.len();
                        }
                    }
                    _ => {}
                }
            }
            Action::HistoryDown => {
                match self.current_pane {
                    PaneFocus::Solutions => self.solutions.next(),
                    PaneFocus::History => self.history.next(),
                    _ => {}
                }
            }
            Action::PaneNext => {
                self.current_pane = match self.current_pane {
                    PaneFocus::Input => PaneFocus::Solutions,
                    PaneFocus::Solutions => PaneFocus::Plot,
                    PaneFocus::Plot => PaneFocus::History,
                    PaneFocus::History => PaneFocus::Input,
                };
            }
            Action::PanePrev => {
                self.current_pane = match self.current_pane {
                    PaneFocus::Input => PaneFocus::History,
                    PaneFocus::Solutions => PaneFocus::Input,
                    PaneFocus::Plot => PaneFocus::Solutions,
                    PaneFocus::History => PaneFocus::Plot,
                };
            }
            Action::Clear => {
                self.clear_all();
            }
            Action::InputChar(_) | Action::InputBackspace | Action::InputLeft | Action::InputRight
            | Action::InputHome | Action::InputEnd => {
                if self.current_pane == PaneFocus::Input {
                    self.input.handle_input(action);
                }
            }
            Action::None => {}
        }
        Ok(())
    }

    fn solve_equation(&mut self) -> Result<()> {
        let input = self.input.buffer.trim();
        if input.is_empty() {
            return Ok(());
        }

        // Add to history
        self.history.add(input.to_string());
        self.db.add(input)?;

        // Parse and solve
        let mut parser = Parser::new(input);
        let expr = parser.parse_equation();

        // Detect variable
        let mut vars = Vec::new();
        self.detect_variables(&expr, &mut vars);

        if vars.is_empty() {
            // Just evaluate
            let result = evaluate(&expr, &HashMap::new());
            self.status = format!(" = {}", result);
            return Ok(());
        }

        let var = vars[0];
        let solver = EquationSolver::new_adaptive(&expr);
        let (real, complex) = solver.find_all_roots(&expr, var);

        // Update panes
        self.solutions.set_solutions(var, &real, &complex, &expr);
        if self.config.display.show_plot {
            self.plot.update(&expr, var, -10.0, 10.0);
        }

        // Store for later use
        self.last_expr = Some(expr);
        self.last_var = Some(var);

        // Status
        let total = real.len() + complex.len();
        self.status = format!(" Found {} solution(s) for '{}' ", total, var);

        // Switch to solutions pane
        self.current_pane = PaneFocus::Solutions;

        Ok(())
    }

    fn plot_function(&mut self) -> Result<()> {
        if let (Some(expr), Some(var)) = (&self.last_expr, self.last_var) {
            self.plot.update(expr, var, -10.0, 10.0);
            self.current_pane = PaneFocus::Plot;
            self.status = format!(" Plotting f({}) from -10 to 10 ", var);
        } else {
            self.status = " No equation to plot. Solve an equation first. ".to_string();
        }
        Ok(())
    }

    fn export_results(&mut self) -> Result<()> {
        if self.solutions.solutions.is_empty() {
            self.status = " No results to export ".to_string();
            return Ok(());
        }

        // Use the existing export functionality
        let mut export = sms_core::export::ExportData::new(
            self.input.buffer.clone(),
            self.last_var.unwrap_or('x').to_string(),
            0.0,
        );

        for sol in &self.solutions.solutions {
            if sol.is_complex {
                // Parse complex value
                let parts: Vec<&str> = sol.value.split_whitespace().collect();
                if parts.len() >= 3 {
                    let re = parts[0].parse().unwrap_or(0.0);
                    let im = parts[2].trim_end_matches('i').parse().unwrap_or(0.0);
                    export.add_complex(re, im, sol.error);
                }
            } else {
                let val = sol.value.split_whitespace().next().unwrap_or("0").parse().unwrap_or(0.0);
                export.add_solution(val, sol.error);
            }
        }

        let _ = export.export_json("result.json");
        let _ = export.export_csv("result.csv");
        let _ = export.export_latex("result.tex");

        self.status = " Exported to result.json, result.csv, result.tex ".to_string();
        Ok(())
    }

    fn export_plot(&mut self) -> Result<()> {
        if self.plot.data.is_empty() {
            self.status = " No plot to export. Press 'p' to plot first. ".to_string();
            return Ok(());
        }

        if let (Some(expr), Some(var)) = (&self.last_expr, self.last_var) {
            use plotters::prelude::*;
            
            let root = BitMapBackend::new("plot.png", (800, 600)).into_drawing_area();
            root.fill(&WHITE)?;

            let mut chart = ChartBuilder::on(&root)
                .caption(format!("f({})", var), ("sans-serif", 24))
                .margin(10)
                .x_label_area_size(30)
                .y_label_area_size(30)
                .build_cartesian_2d(-10.0..10.0, -10.0..10.0)?;

            chart.configure_mesh().draw()?;

            let num_points = 1000;
            let step = 20.0 / 1000.0;
            let mut points: Vec<(f64, f64)> = Vec::new();
            
            for i in 0..=num_points {
                let x = -10.0 + i as f64 * step;
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
            self.status = " Plot exported to plot.png ".to_string();
            Ok(())
        } else {
            self.status = " No equation to plot. Solve an equation first. ".to_string();
            Ok(())
        }
    }

    fn clear_all(&mut self) {
        self.input.buffer.clear();
        self.input.cursor = 0;
        self.solutions.solutions.clear();
        self.plot.data.clear();
        self.status.clear();
        self.last_expr = None;
        self.last_var = None;
    }

    fn show_help(&mut self) {
        self.status = " Help: Tab=Switch pane, Enter=Solve, p=Plot, e=Export, E=Export Plot, c=Clear, ↑↓=Navigate, q=Quit ".to_string();
    }

    fn detect_variables(&self, expr: &Expr, vars: &mut Vec<char>) {
        match expr {
            Expr::Var(c) if !vars.contains(c) => vars.push(*c),
            Expr::Add(a, b) | Expr::Sub(a, b) | Expr::Mul(a, b) | Expr::Div(a, b) | Expr::Pow(a, b) | Expr::Log(a, b) => {
                self.detect_variables(a, vars);
                self.detect_variables(b, vars);
            }
            Expr::Sin(a) | Expr::Cos(a) | Expr::Tan(a) | Expr::Asin(a) | Expr::Acos(a) | Expr::Atan(a) |
            Expr::Sinh(a) | Expr::Cosh(a) | Expr::Tanh(a) | Expr::Ln(a) | Expr::Exp(a) | Expr::Sqrt(a) | Expr::Abs(a) => {
                self.detect_variables(a, vars);
            }
            _ => {}
        }
    }
}