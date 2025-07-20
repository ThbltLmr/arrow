import React from "react";
import { PostureUpdate, PostureType } from "../types";

interface PostureDisplayProps {
  postureUpdate: PostureUpdate | null;
}

const PostureDisplay: React.FC<PostureDisplayProps> = ({ postureUpdate }) => {
  const getPostureIcon = (posture: PostureType | null): string => {
    if (!posture || posture === "Unknown") {
      return "/bad_posture.svg";
    }
    
    return posture === "Straight" ? "/good_posture.svg" : "/bad_posture.svg";
  };

  const getPostureMessage = (): string => {
    if (!postureUpdate) {
      return "Waiting for posture data...";
    }
    return postureUpdate.message;
  };

  const getPostureStatus = (): "good" | "bad" | "unknown" => {
    if (!postureUpdate) return "unknown";
    return postureUpdate.posture === "Straight" ? "good" : "bad";
  };

  return (
    <div className="posture-display">
      <div className="posture-icon-container">
        <img 
          src={getPostureIcon(postureUpdate?.posture || null)}
          alt={`${getPostureStatus()} posture`}
          className={`posture-icon posture-${getPostureStatus()}`}
        />
      </div>
      
      <div className="posture-info">
        <h2 className={`posture-message posture-${getPostureStatus()}`}>
          {getPostureMessage()}
        </h2>
        
        {postureUpdate && (
          <div className="posture-details">
            <span className="posture-type">
              Status: {postureUpdate.posture.replace(/([A-Z])/g, ' $1').trim()}
            </span>
            
            {postureUpdate.metrics && (
              <div className="posture-metrics">
                <small>
                  Visibility: 
                  Ears ({Math.round((postureUpdate.metrics.left_ear.visibility + postureUpdate.metrics.right_ear.visibility) / 2 * 100)}%), 
                  Shoulders ({Math.round((postureUpdate.metrics.left_shoulder.visibility + postureUpdate.metrics.right_shoulder.visibility) / 2 * 100)}%)
                </small>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default PostureDisplay;