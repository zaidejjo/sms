//! History persistence using SQLite

use rusqlite::Connection;
use std::path::PathBuf;
use dirs;
use anyhow::Context;

pub struct HistoryDB {
    conn: Connection,
}

impl HistoryDB {
    pub fn new() -> anyhow::Result<Self> {
        let db_path = Self::db_path()?;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }
        let conn = Connection::open(&db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                equation TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    pub fn add(&self, equation: &str) -> anyhow::Result<()> {
        self.conn.execute(
            "INSERT INTO history (equation) VALUES (?1)",
            [equation],
        )?;
        Ok(())
    }

    pub fn get_all(&self, limit: usize) -> anyhow::Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT equation FROM history ORDER BY timestamp DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map([limit], |row| row.get(0))?;
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        Ok(entries)
    }

    #[allow(dead_code)]
    pub fn clear(&self) -> anyhow::Result<()> {
        self.conn.execute("DELETE FROM history", [])?;
        Ok(())
    }

    fn db_path() -> anyhow::Result<PathBuf> {
        let config_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sms");
        Ok(config_dir.join("history.db"))
    }
}