//! Key handling and input processing

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    Quit,
    Help,
    Solve,
    Plot,
    Export,
    ExportPlot,
    HistoryUp,
    HistoryDown,
    PaneNext,
    PanePrev,
    Clear,
    InputChar(char),
    InputBackspace,
    InputLeft,
    InputRight,
    InputHome,
    InputEnd,
    None,
}

pub struct KeyHandler {
    bindings: std::collections::HashMap<String, Action>,
}

impl KeyHandler {
    pub fn new(config: &crate::config::Keybindings) -> Self {
        let mut bindings = std::collections::HashMap::new();
        Self::bind(&mut bindings, &config.quit, Action::Quit);
        Self::bind(&mut bindings, &config.help, Action::Help);
        Self::bind(&mut bindings, &config.solve, Action::Solve);
        Self::bind(&mut bindings, &config.plot, Action::Plot);
        Self::bind(&mut bindings, &config.export, Action::Export);
        Self::bind(&mut bindings, &config.export_plot, Action::ExportPlot);
        Self::bind(&mut bindings, &config.history_up, Action::HistoryUp);
        Self::bind(&mut bindings, &config.history_down, Action::HistoryDown);
        Self::bind(&mut bindings, &config.pane_next, Action::PaneNext);
        Self::bind(&mut bindings, &config.pane_prev, Action::PanePrev);
        Self::bind(&mut bindings, &config.clear, Action::Clear);
        Self { bindings }
    }

    fn bind(map: &mut std::collections::HashMap<String, Action>, key: &str, action: Action) {
        map.insert(key.to_lowercase(), action);
    }

    pub fn handle(&self, event: KeyEvent) -> Action {
        let key_str = self.key_event_to_string(event);
        self.bindings.get(&key_str.to_lowercase())
            .cloned()
            .unwrap_or_else(|| {
                match event.code {
                    KeyCode::Char(c) => Action::InputChar(c),
                    KeyCode::Backspace => Action::InputBackspace,
                    KeyCode::Left => Action::InputLeft,
                    KeyCode::Right => Action::InputRight,
                    KeyCode::Home => Action::InputHome,
                    KeyCode::End => Action::InputEnd,
                    _ => Action::None,
                }
            })
    }

    fn key_event_to_string(&self, event: KeyEvent) -> String {
        let mut parts = Vec::new();
        if event.modifiers.contains(KeyModifiers::CONTROL) {
            parts.push("ctrl");
        }
        if event.modifiers.contains(KeyModifiers::SHIFT) {
            parts.push("shift");
        }
        if event.modifiers.contains(KeyModifiers::ALT) {
            parts.push("alt");
        }

        let key = match event.code {
            KeyCode::Enter => "enter",
            KeyCode::Tab => "tab",
            KeyCode::BackTab => "backtab",
            KeyCode::Up => "up",
            KeyCode::Down => "down",
            KeyCode::Left => "left",
            KeyCode::Right => "right",
            KeyCode::Home => "home",
            KeyCode::End => "end",
            KeyCode::PageUp => "pageup",
            KeyCode::PageDown => "pagedown",
            KeyCode::Delete => "delete",
            KeyCode::Insert => "insert",
            KeyCode::Esc => "esc",
            KeyCode::F(n) => return format!("f{}", n),
            KeyCode::Char(c) => return c.to_string(),
            KeyCode::Backspace => "backspace",
            _ => "unknown",
        };
        parts.push(key);
        parts.join("+")
    }
}