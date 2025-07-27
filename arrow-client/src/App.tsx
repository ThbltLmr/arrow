import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";
import {
	PostureUpdate,
	ConnectionStatus,
	SessionLogsUpdate,
	NotificationEvent,
	PostureLog,
	WeeklyStats as WeeklyStatsType
} from "./types";
import PostureDisplay from "./components/PostureDisplay";
import ConnectionIndicator from "./components/ConnectionIndicator";
import SessionHistory from "./components/SessionHistory";
import WeeklyStats from "./components/WeeklyStats";

function App() {
	const [postureUpdate, setPostureUpdate] = useState<PostureUpdate | null>(null);
	const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>({
		connected: false,
		message: "Initializing..."
	});
	const [sessionLogs, setSessionLogs] = useState<PostureLog[]>([]);
	const [weeklyStats, setWeeklyStats] = useState<WeeklyStatsType>({ days: [] });
	const [isInitialized, setIsInitialized] = useState(false);

	useEffect(() => {
		let isCleanedUp = false;

		const initializeApp = async () => {
			try {
				// Initialize the backend
				await invoke("initialize_app");
				if (!isCleanedUp) {
					setIsInitialized(true);
					setConnectionStatus({
						message: "Connected",
						connected: true,
					})
				}
			} catch (error) {
				console.error("Failed to initialize app:", error);
				if (!isCleanedUp) {
					setConnectionStatus({
						connected: false,
						message: `Initialization failed: ${error}`
					});
				}
			}
		};

		const setupEventListeners = async () => {
			try {
				// Listen for posture updates
				const postureUnlisten = await listen<PostureUpdate>("posture-update", (event) => {
					if (!isCleanedUp) {
						setPostureUpdate(event.payload);
					}
				});

				// Listen for connection status changes
				const connectionUnlisten = await listen<ConnectionStatus>("connection-status", (event) => {
					if (!isCleanedUp) {
						setConnectionStatus(event.payload);
					}
				});

				// Listen for session logs updates
				const logsUnlisten = await listen<SessionLogsUpdate>("session-logs-updated", (event) => {
					if (!isCleanedUp) {
						setSessionLogs(event.payload.logs);
					}
				});

				// Listen for notification events (for frontend feedback)
				const notificationUnlisten = await listen<NotificationEvent>("notification-triggered", (event) => {
					if (!isCleanedUp) {
						console.log("Notification triggered:", event.payload);
					}
				});

				// Return cleanup function
				return () => {
					postureUnlisten();
					connectionUnlisten();
					logsUnlisten();
					notificationUnlisten();
				};
			} catch (error) {
				console.error("Failed to setup event listeners:", error);
				return () => { }; // No-op cleanup
			}
		};

		// Initialize app and setup listeners
		const init = async () => {
			await initializeApp();
			const cleanup = await setupEventListeners();
			return cleanup;
		};

		let cleanup: (() => void) | undefined;
		init().then((cleanupFn) => {
			cleanup = cleanupFn;
		});

		// Cleanup on unmount
		return () => {
			isCleanedUp = true;
			if (cleanup) {
				cleanup();
			}
			// Call cleanup on the backend
			invoke("cleanup_app").catch(console.error);
		};
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

	const fetchWeeklyStats = async () => {
		try {
			const stats = await invoke<WeeklyStatsType>("get_weekly_stats");
			setWeeklyStats(stats);
		} catch (error) {
			console.error("Failed to fetch weekly stats:", error);
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

				<div className="weekly-section">
					<WeeklyStats
						stats={weeklyStats}
						onRefresh={fetchWeeklyStats}
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
