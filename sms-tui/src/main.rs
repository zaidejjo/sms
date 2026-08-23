//! SMS Terminal User Interface
//!
//! Interactive TUI dashboard for the Smart Math Solver

mod app;
mod panes;
mod keys;
mod config;
mod history;

use anyhow::Result;
use app::App;

fn main() -> Result<()> {
    let mut app = App::new()?;
    app.run()?;
    Ok(())
}