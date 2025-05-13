use rusqlite::{Connection, Result as SqlResult};
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct EventLog {
    timestamp: String,
    event_type: String,
    posture: String,
    previous_posture: Option<String>,
}

pub struct DbManager {
    conn: Connection,
}

impl DbManager {
    pub fn new() -> SqlResult<Self, Box<dyn std::error::Error>> {
        // Create app data directory if it doesn't exist
        let app_dir = DbManager::get_app_data_dir();
        fs::create_dir_all(&app_dir)?;

        // Database file path
        let db_path = app_dir.join("posture_data.db");

        // Open/create the database
        let conn = Connection::open(db_path)?;

        // Initialize the database
        let manager = DbManager { conn };
        manager.init_db()?;

        Ok(manager)
    }

    fn init_db(&self) -> SqlResult<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS posture_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                event_type TEXT NOT NULL,
                posture TEXT NOT NULL,
                previous_posture TEXT,
            )",
            [],
        )?;

        Ok(())
    }

    pub fn log_session_start(&self) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO posture_events 
             (timestamp, event_type, posture, previous_posture)
             VALUES (datetime('now'), 'START', 'UNKNOWN', NULL)",
            [],
        )?;

        Ok(())
    }

    pub fn log_session_end(&self, last_posture: &str) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO posture_events 
             (timestamp, event_type, posture, previous_posture)
             VALUES (datetime('now'), 'STOP', ?, NULL)",
            [last_posture],
        )?;

        Ok(())
    }

    pub fn log_posture_change(&self, current_posture: &str, last_posture: &str) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO posture_events 
             (timestamp, event_type, posture, previous_posture)
             VALUES (datetime('now'), 'CHANGE', ?, ?)",
            [current_posture, last_posture],
        )?;

        Ok(())
    }

    pub fn get_last_logs(
        &self,
        number: usize,
    ) -> Result<Vec<EventLog>, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, event_type, posture, previous_posture
                                            FROM posture_events
                                            ORDER BY timestamp DESC LIMIT ?",
        )?;
        let log_iter = stmt.query_map([number], |row| {
            Ok(EventLog {
                timestamp: row.get(0)?,
                event_type: row.get(1)?,
                posture: row.get(2)?,
                previous_posture: row.get(3)?,
            })
        });

        Ok(log_iter
            .unwrap()
            .map(|res| res.unwrap())
            .collect::<Vec<EventLog>>())
    }

    fn get_app_data_dir() -> PathBuf {
        let mut app_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        app_dir.push("PostureMonitor");
        app_dir
    }
}
