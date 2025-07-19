use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

use crate::ConnectionStatus;

pub struct PythonServer {
    process: Option<Child>,
    server_path: PathBuf,
}

impl PythonServer {
    pub fn new() -> Self {
        // Find the server directory relative to the client
        let mut server_path = std::env::current_dir().unwrap();
        
        // Navigate to the server directory
        if server_path.ends_with("client") {
            server_path.pop(); // Go back to project root
            server_path.push("server");
        } else if server_path.file_name() == Some("src-tauri".as_ref()) {
            server_path.pop(); // Go back to client
            server_path.pop(); // Go back to project root
            server_path.push("server");
        } else {
            // Assume we're in project root
            server_path.push("server");
        }

        Self {
            process: None,
            server_path,
        }
    }

    pub fn start(&mut self, app_handle: &AppHandle) -> Result<(), String> {
        if self.process.is_some() {
            return Ok(()); // Already started
        }

        // Check if Python is available
        let python_cmd = if Command::new("python3").arg("--version").output().is_ok() {
            "python3"
        } else if Command::new("python").arg("--version").output().is_ok() {
            "python"
        } else {
            return Err("Python not found. Please install Python 3.x".to_string());
        };

        // Check if server directory exists
        if !self.server_path.exists() {
            return Err(format!("Server directory not found: {}", self.server_path.display()));
        }

        let main_py = self.server_path.join("main.py");
        if !main_py.exists() {
            return Err(format!("main.py not found in: {}", self.server_path.display()));
        }

        // Check if requirements are installed
        self.install_requirements(python_cmd)?;

        println!("Starting Python server from: {}", self.server_path.display());

        // Start the Python server
        let mut child = Command::new(python_cmd)
            .arg("main.py")
            .current_dir(&self.server_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start Python server: {}", e))?;

        // Give the server a moment to start
        std::thread::sleep(std::time::Duration::from_millis(2000));

        // Check if the process is still running
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(format!("Python server exited immediately with status: {}", status));
            }
            Ok(None) => {
                // Process is still running, which is good
                println!("Python server started successfully");
                
                let status = ConnectionStatus {
                    connected: false,
                    message: "Python server started. Waiting for connection...".to_string(),
                };
                let _ = app_handle.emit("connection-status", &status);
            }
            Err(e) => {
                return Err(format!("Error checking Python server status: {}", e));
            }
        }

        self.process = Some(child);
        Ok(())
    }

    fn install_requirements(&self, python_cmd: &str) -> Result<(), String> {
        let requirements_file = self.server_path.join("requirements.txt");
        if !requirements_file.exists() {
            return Ok(()); // No requirements file, skip installation
        }

        println!("Installing Python requirements...");
        let output = Command::new(python_cmd)
            .args(["-m", "pip", "install", "-r", "requirements.txt"])
            .current_dir(&self.server_path)
            .output()
            .map_err(|e| format!("Failed to install requirements: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to install requirements: {}", stderr));
        }

        println!("Python requirements installed successfully");
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(mut child) = self.process.take() {
            println!("Stopping Python server...");
            let _ = child.kill();
            let _ = child.wait();
            println!("Python server stopped");
        }
    }

    pub fn is_running(&mut self) -> bool {
        if let Some(child) = &mut self.process {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    self.process = None;
                    false
                }
                Ok(None) => {
                    // Process is still running
                    true
                }
                Err(_) => {
                    // Error checking status, assume dead
                    self.process = None;
                    false
                }
            }
        } else {
            false
        }
    }
}

impl Drop for PythonServer {
    fn drop(&mut self) {
        self.stop();
    }
}

pub type PythonServerState = Mutex<PythonServer>;