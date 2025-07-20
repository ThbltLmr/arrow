import React from "react";
import { PostureLog } from "../types";

interface SessionHistoryProps {
  logs: PostureLog[];
  onRefresh: () => void;
}

const SessionHistory: React.FC<SessionHistoryProps> = ({ logs, onRefresh }) => {
  const formatDuration = (duration: { secs: number; nanos: number }): string => {
    const totalSeconds = duration.secs + duration.nanos / 1_000_000_000;
    
    if (totalSeconds < 60) {
      return `${Math.round(totalSeconds)}s`;
    } else if (totalSeconds < 3600) {
      const minutes = Math.floor(totalSeconds / 60);
      const seconds = Math.round(totalSeconds % 60);
      return `${minutes}m ${seconds}s`;
    } else {
      const hours = Math.floor(totalSeconds / 3600);
      const minutes = Math.floor((totalSeconds % 3600) / 60);
      return `${hours}h ${minutes}m`;
    }
  };

  const getPostureDisplayName = (posture: string): string => {
    switch (posture) {
      case "STRAIGHT":
        return "Good Posture";
      case "SLOUCHING_BACK":
        return "Slouching Back";
      case "LEANING_IN":
        return "Leaning In";
      case "HEAD_TILT_LEFT":
        return "Head Tilt Left";
      case "HEAD_TILT_RIGHT":
        return "Head Tilt Right";
      case "BODY_TILT_LEFT":
        return "Body Tilt Left";
      case "BODY_TILT_RIGHT":
        return "Body Tilt Right";
      case "SHOULDERS_NOT_VISIBLE":
        return "Shoulders Not Visible";
      case "HEAD_NOT_VISIBLE":
        return "Head Not Visible";
      default:
        return posture.replace(/_/g, " ").toLowerCase().replace(/\b\w/g, l => l.toUpperCase());
    }
  };

  const getPostureClass = (posture: string): string => {
    return posture === "STRAIGHT" ? "good-posture" : "bad-posture";
  };

  return (
    <div className="session-history">
      <div className="history-header">
        <h3>Session History</h3>
        <button 
          className="refresh-button"
          onClick={onRefresh}
          title="Refresh session history"
        >
          🔄
        </button>
      </div>

      <div className="history-content">
        {logs.length === 0 ? (
          <div className="no-history">
            <p>No posture changes recorded yet</p>
            <small>Start moving and your posture history will appear here</small>
          </div>
        ) : (
          <div className="history-list">
            {logs.slice(0, 10).map((log, index) => (
              <div key={index} className={`history-item ${getPostureClass(log.posture)}`}>
                <div className="posture-name">
                  {getPostureDisplayName(log.posture)}
                </div>
                <div className="posture-duration">
                  {formatDuration(log.duration)}
                </div>
              </div>
            ))}
            
            {logs.length > 10 && (
              <div className="more-items">
                + {logs.length - 10} more entries
              </div>
            )}
          </div>
        )}
      </div>

      <div className="history-summary">
        {logs.length > 0 && (
          <small>
            Total entries: {logs.length} | 
            Good posture time: {
              formatDuration(
                logs
                  .filter(log => log.posture === "STRAIGHT")
                  .reduce(
                    (total, log) => ({
                      secs: total.secs + log.duration.secs,
                      nanos: total.nanos + log.duration.nanos
                    }),
                    { secs: 0, nanos: 0 }
                  )
              )
            }
          </small>
        )}
      </div>
    </div>
  );
};

export default SessionHistory;