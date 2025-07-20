import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";
import { 
  PostureUpdate, 
  ConnectionStatus, 
  SessionLogsUpdate, 
  NotificationEvent,
  PostureLog 
} from "./types";
import PostureDisplay from "./components/PostureDisplay";
import ConnectionIndicator from "./components/ConnectionIndicator";
import SessionHistory from "./components/SessionHistory";

function App() {
  const [postureUpdate, setPostureUpdate] = useState<PostureUpdate | null>(null);
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>({
    connected: false,
    message: "Initializing..."
  });
  const [sessionLogs, setSessionLogs] = useState<PostureLog[]>([]);
  const [isInitialized, setIsInitialized] = useState(false);

  useEffect(() => {
    const initializeApp = async () => {
      try {
        // Initialize the backend
        await invoke("initialize_app");
        setIsInitialized(true);
      } catch (error) {
        console.error("Failed to initialize app:", error);
        setConnectionStatus({
          connected: false,
          message: `Initialization failed: ${error}`
        });
      }
    };

    const setupEventListeners = async () => {
      // Listen for posture updates
      const postureUnlisten = await listen<PostureUpdate>("posture-update", (event) => {
        setPostureUpdate(event.payload);
      });

      // Listen for connection status changes
      const connectionUnlisten = await listen<ConnectionStatus>("connection-status", (event) => {
        setConnectionStatus(event.payload);
      });

      // Listen for session logs updates
      const logsUnlisten = await listen<SessionLogsUpdate>("session-logs-updated", (event) => {
        setSessionLogs(event.payload.logs);
      });

      // Listen for notification events (for frontend feedback)
      const notificationUnlisten = await listen<NotificationEvent>("notification-triggered", (event) => {
        console.log("Notification triggered:", event.payload);
      });

      // Cleanup function
      return () => {
        postureUnlisten();
        connectionUnlisten();
        logsUnlisten();
        notificationUnlisten();
      };
    };

    initializeApp();
    setupEventListeners();
  }, []);

  const fetchSessionLogs = async () => {
    try {
      const logs = await invoke<PostureLog[] | null>("get_session_logs");
      if (logs) {
        setSessionLogs(logs);
      }
    } catch (error) {
      console.error("Failed to fetch session logs:", error);
    }
  };

  const getConnectionDetails = async () => {
    try {
      const status = await invoke<ConnectionStatus>("get_connection_status");
      setConnectionStatus(status);
    } catch (error) {
      console.error("Failed to get connection status:", error);
    }
  };

  return (
    <main className="app">
      <header className="app-header">
        <h1>Arrow - Posture Monitor</h1>
        <ConnectionIndicator 
          status={connectionStatus} 
          onRefresh={getConnectionDetails}
        />
      </header>

      <div className="app-content">
        <div className="posture-section">
          <PostureDisplay postureUpdate={postureUpdate} />
        </div>

        <div className="history-section">
          <SessionHistory 
            logs={sessionLogs} 
            onRefresh={fetchSessionLogs}
          />
        </div>
      </div>

      {!isInitialized && (
        <div className="loading-overlay">
          <div className="loading-spinner">
            <p>Initializing posture monitoring...</p>
          </div>
        </div>
      )}
    </main>
  );
}

export default App;
