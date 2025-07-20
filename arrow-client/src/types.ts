// Posture types
export type PostureType = 
  | "ShouldersNotVisible"
  | "HeadNotVisible" 
  | "SlouchingBack"
  | "LeaningIn"
  | "HeadTiltLeft"
  | "HeadTiltRight"
  | "BodyTiltLeft"
  | "BodyTiltRight"
  | "Straight"
  | "Unknown";

export interface Posture {
  posture: PostureType;
  message: string;
}

export interface Point3D {
  x: number;
  y: number;
  z: number;
  visibility: number;
}

export interface PostureMetrics {
  left_ear: Point3D;
  right_ear: Point3D;
  left_shoulder: Point3D;
  right_shoulder: Point3D;
}

export interface PostureUpdate {
  posture: PostureType;
  message: string;
  metrics?: PostureMetrics;
}

export interface ConnectionStatus {
  connected: boolean;
  message: string;
}

export interface PostureLog {
  posture: string;
  duration: {
    secs: number;
    nanos: number;
  };
}

export interface SessionLogsUpdate {
  logs: PostureLog[];
}

export interface NotificationEvent {
  posture: string;
  message: string;
  is_good_posture: boolean;
}