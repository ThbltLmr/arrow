import React from "react";
import { ConnectionStatus } from "../types";

interface ConnectionIndicatorProps {
  status: ConnectionStatus;
  onRefresh: () => void;
}

const ConnectionIndicator: React.FC<ConnectionIndicatorProps> = ({ status, onRefresh }) => {
  const getStatusClass = (): string => {
    return status.connected ? "connected" : "disconnected";
  };

  const getStatusIcon = (): string => {
    return status.connected ? "🟢" : "🔴";
  };

  return (
    <div className={`connection-indicator ${getStatusClass()}`}>
      <div className="connection-status">
        <span className="status-icon">{getStatusIcon()}</span>
        <span className="status-text">
          {status.connected ? "Connected" : "Disconnected"}
        </span>
      </div>
      
      <div className="connection-message">
        {status.message}
      </div>
      
      <button 
        className="refresh-button"
        onClick={onRefresh}
        title="Refresh connection status"
      >
        🔄
      </button>
    </div>
  );
};

export default ConnectionIndicator;