import React from "react";
import { WeeklyStats as WeeklyStatsType, DayStats } from "../types";

interface WeeklyStatsProps {
  stats: WeeklyStatsType;
  onRefresh: () => void;
}

const WeeklyStats: React.FC<WeeklyStatsProps> = ({ stats, onRefresh }) => {
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

  const getDurationInSeconds = (duration: { secs: number; nanos: number }): number => {
    return duration.secs + duration.nanos / 1_000_000_000;
  };

  const getDayName = (dateStr: string): string => {
    const date = new Date(dateStr);
    const today = new Date();
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);

    if (date.toDateString() === today.toDateString()) {
      return "Today";
    } else if (date.toDateString() === yesterday.toDateString()) {
      return "Yesterday";
    } else {
      return date.toLocaleDateString('en-US', { weekday: 'short' });
    }
  };

  // Find max total time for scaling bars
  const maxTotalTime = Math.max(
    ...stats.days.map(day => getDurationInSeconds(day.total_time)),
    1 // Minimum of 1 to avoid division by zero
  );

  const calculateBarHeight = (duration: { secs: number; nanos: number }): number => {
    const seconds = getDurationInSeconds(duration);
    return Math.max((seconds / maxTotalTime) * 100, 0); // Height as percentage
  };

  return (
    <div className="weekly-stats">
      <div className="stats-header">
        <h3>Weekly Overview</h3>
        <button 
          className="refresh-button"
          onClick={onRefresh}
          title="Refresh weekly stats"
        >
          🔄
        </button>
      </div>

      <div className="stats-content">
        <div className="bar-chart">
          {stats.days.map((day, index) => {
            const totalHeight = calculateBarHeight(day.total_time);
            const goodHeight = totalHeight > 0 ? (getDurationInSeconds(day.good_posture_time) / getDurationInSeconds(day.total_time)) * totalHeight : 0;
            const badHeight = totalHeight - goodHeight;
            
            return (
              <div key={index} className="bar-column">
                <div className="bar-container">
                  <div 
                    className="bar-stack"
                    style={{ height: `${Math.max(totalHeight, 2)}px` }} // Minimum 2px for visibility
                  >
                    {totalHeight > 0 ? (
                      <>
                        <div 
                          className="bar-segment bad-posture"
                          style={{ height: `${badHeight}px` }}
                          title={`Bad posture: ${formatDuration(day.bad_posture_time)}`}
                        />
                        <div 
                          className="bar-segment good-posture"
                          style={{ height: `${goodHeight}px` }}
                          title={`Good posture: ${formatDuration(day.good_posture_time)}`}
                        />
                      </>
                    ) : (
                      <div 
                        className="bar-segment no-data"
                        style={{ height: "2px" }}
                        title="No data for this day"
                      />
                    )}
                  </div>
                  <div className="time-label">
                    {totalHeight > 0 ? formatDuration(day.total_time) : "0m"}
                  </div>
                </div>
                <div className="day-label">
                  {getDayName(day.date)}
                </div>
              </div>
            );
          })}
        </div>

        <div className="stats-legend">
          <div className="legend-item">
            <div className="legend-color good-posture"></div>
            <span>Good Posture</span>
          </div>
          <div className="legend-item">
            <div className="legend-color bad-posture"></div>
            <span>Bad Posture</span>
          </div>
        </div>

        <div className="stats-summary">
          {stats.days.some(day => getDurationInSeconds(day.total_time) > 0) ? (
            <small>
              Weekly total: {
                formatDuration(
                  stats.days.reduce(
                    (total, day) => ({
                      secs: total.secs + day.total_time.secs,
                      nanos: total.nanos + day.total_time.nanos
                    }),
                    { secs: 0, nanos: 0 }
                  )
                )
              } | Good posture: {
                formatDuration(
                  stats.days.reduce(
                    (total, day) => ({
                      secs: total.secs + day.good_posture_time.secs,
                      nanos: total.nanos + day.good_posture_time.nanos
                    }),
                    { secs: 0, nanos: 0 }
                  )
                )
              }
            </small>
          ) : (
            <small>No posture data recorded this week</small>
          )}
        </div>
      </div>
    </div>
  );
};

export default WeeklyStats;