use rusqlite::{Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostureLog {
    pub posture: String,
    pub duration: Duration,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct EventLog {
    pub timestamp: String,
    pub event_type: String,
    pub posture: String,
    pub previous_posture: Option<String>,
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
                previous_posture TEXT
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
             VALUES (datetime('now'), 'STOP', ?, ?)",
            [last_posture, last_posture],
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

    pub fn get_session_logs(&self) -> Result<Option<Vec<PostureLog>>, Box<dyn std::error::Error>> {
        let mut start_stmt = self.conn.prepare(
            "SELECT id
                FROM posture_events
                WHERE event_type = 'START'
                ORDER BY timestamp DESC LIMIT 1",
        )?;

        let start_log_iter = start_stmt.query_map([], |row| Ok(row.get(0)?));

        let start_log_id: usize = start_log_iter.unwrap().next().unwrap().unwrap();

        self.get_logs(format!("logs.id > {}", start_log_id).as_str())
    }

    /*
    Generic function to select logs from the database
    @params:
    query: &str -> will be included in the WHERE clause of the SQL query
    example: get_logs("logs.id = 1".as_str())
    */
    fn get_logs(&self, query: &str) -> Result<Option<Vec<PostureLog>>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            format!("SELECT logs.previous_posture, ((julianday(logs.timestamp) - julianday(e2.timestamp)) * 86400.0) as duration
                FROM posture_events logs
                JOIN posture_events e2 ON logs.id = e2.id + 1
                WHERE {}
                AND ((julianday(logs.timestamp) - julianday(e2.timestamp)) * 86400.0) > 3
                AND logs.event_type != 'START'
                ORDER BY logs.timestamp DESC", query).as_str(),
        )?;

        let log_iter = stmt.query_map([], |row| {
            Ok(PostureLog {
                posture: row.get(0)?,
                duration: Duration::from_secs_f64(row.get(1)?),
            })
        });

        let log_vec = log_iter?
            .map(|res| res.unwrap())
            .collect::<Vec<PostureLog>>();

        if log_vec.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(log_vec))
        }
    }

    fn get_app_data_dir() -> PathBuf {
        let mut app_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        app_dir.push("PostureMonitor");
        app_dir
    }
}

#[allow(dead_code)]
mod timestamp {
    use std::time::Duration;

    pub struct Timestamp {
        year: i32,
        month: i8,
        day: i8,
        hour: i8,
        minutes: i8,
        seconds: i8,
    }

    impl From<&str> for Timestamp {
        fn from(value: &str) -> Self {
            let split_vec: Vec<&str> = value.split('-').collect();
            let year: i32 = split_vec[0].parse().unwrap();
            let month: i8 = split_vec[1].parse().unwrap();
            let (day_str, time_str): (&str, &str) = split_vec[2].split_once(' ').unwrap();
            let day: i8 = day_str.parse().unwrap();
            let time_vec: Vec<&str> = time_str.split(":").collect();

            let hour: i8 = time_vec[0].parse().unwrap();
            let minutes: i8 = time_vec[1].parse().unwrap();
            let seconds: i8 = time_vec[2].parse().unwrap();

            Timestamp {
                year,
                month,
                day,
                hour,
                minutes,
                seconds,
            }
        }
    }

    impl Timestamp {
        pub fn to_seconds(&self) -> u64 {
            (self.seconds as u64)
                + (self.minutes as u64) * 60
                + (self.hour as u64) * 3600
                + (self.day as u64) * 86400
                + (self.month as u64) * 2592000
                + (self.year as u64) * 31536000
        }
    }

    pub fn timestamp_difference(latest: Timestamp, earliest: Timestamp) -> Duration {
        Duration::from_secs(latest.to_seconds() - earliest.to_seconds())
    }
}