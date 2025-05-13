use rusqlite::{Connection, Result as SqlResult};
use std::fs;
use std::path::PathBuf;

pub struct DbManager {
    conn: Connection,
}

impl DbManager {
    fn get_app_data_dir(&self) -> PathBuf {
        let mut app_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        app_dir.push("PostureMonitor");
        app_dir
    }
}
