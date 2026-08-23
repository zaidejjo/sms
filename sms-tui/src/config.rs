//! Configuration management

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use dirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: Theme,
    pub keybindings: Keybindings,
    pub solver: SolverConfig,
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub fg: String,
    pub bg: String,
    pub highlight: String,
    pub error: String,
    pub success: String,
    pub border: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybindings {
    pub quit: String,
    pub help: String,
    pub solve: String,
    pub plot: String,
    pub export: String,
    pub history_up: String,
    pub history_down: String,
    pub pane_next: String,
    pub pane_prev: String,
    pub clear: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverConfig {
    pub default_mode: String, // "adaptive", "fast", "precision"
    pub max_iterations: usize,
    pub tolerance: f64,
    pub show_steps: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub show_plot: bool,
    pub plot_height: u16,
    pub decimal_places: usize,
    pub show_fractions: bool,
    pub show_errors: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            keybindings: Keybindings::default(),
            solver: SolverConfig::default(),
            display: DisplayConfig::default(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            fg: "white".to_string(),
            bg: "black".to_string(),
            highlight: "yellow".to_string(),
            error: "red".to_string(),
            success: "green".to_string(),
            border: "cyan".to_string(),
            title: "magenta".to_string(),
        }
    }
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            quit: "q".to_string(),
            help: "?".to_string(),
            solve: "Enter".to_string(),
            plot: "p".to_string(),
            export: "e".to_string(),
            history_up: "Up".to_string(),
            history_down: "Down".to_string(),
            pane_next: "Tab".to_string(),
            pane_prev: "BackTab".to_string(),
            clear: "c".to_string(),
        }
    }
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            default_mode: "adaptive".to_string(),
            max_iterations: 100,
            tolerance: 1e-12,
            show_steps: false,
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_plot: true,
            plot_height: 20,
            decimal_places: 6,
            show_fractions: true,
            show_errors: true,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::config_path();
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sms")
            .join("config.toml")
    }
}