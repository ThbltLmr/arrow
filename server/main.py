import cv2
import mediapipe as mp
import socket

HOST = '127.0.0.1'
PORT = 9876

POSTURES = ["STRAIGHT", "SLOUCHING_BACK", "LEANING_IN", "HEAD_TILT_RIGHT", "HEAD_TILT_LEFT", "BODY_TILT_RIGHT", "BODY_TILT_LEFT"]

def get_posture(left_ear, right_ear, left_shoulder, right_shoulder):
    avg_ear_depth = (left_ear.z + right_ear.z) / 2
    avg_shoulder_depth = (left_shoulder.z + right_shoulder.z) / 2
    
    # Check slouching
    if avg_ear_depth + 0.2 < avg_shoulder_depth and avg_shoulder_depth > -0.33:
        return POSTURES[1]
    if avg_ear_depth + 0.33 < avg_shoulder_depth:
        return POSTURES[2]

    # Check head tilt
    ear_slope = (left_ear.y - right_ear.y) / (left_ear.x - right_ear.x)
    if ear_slope > 0.05:
        return POSTURES[3]
    if ear_slope < -0.05:
        return POSTURES[4]

    # Check body tilt
    shoulder_slope = (left_shoulder.y - right_shoulder.y) / (left_shoulder.x - right_shoulder.x)
    if shoulder_slope > 0.05:
        return POSTURES[5]
    if shoulder_slope < -0.05:
        return POSTURES[6]

    # Default to STRAIGHT
    return POSTURES[0]

def main():
    # Setup MediaPipe Pose
    mp_pose = mp.solutions.pose
    pose = mp_pose.Pose()

    # Open Webcam
    cap = cv2.VideoCapture(0)

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind((HOST, PORT))
        s.listen()
        print("Server listening...")
        conn, addr = s.accept()
        with conn:
            print(f"Connected by {addr}")
            while cap.isOpened():
                ret, frame = cap.read()
                if not ret:
                    print("Failed to grab frame")
                    break

                # Flip the frame horizontally for a mirror-like effect
                frame = cv2.flip(frame, 1)

                # Convert to RGB
                image_rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
                image_rgb.flags.writeable = False

                # Process the frame
                results = pose.process(image_rgb)
        
                # Convert back to BGR
                image_rgb.flags.writeable = True
                frame = cv2.cvtColor(image_rgb, cv2.COLOR_RGB2BGR)
        
                if results.pose_landmarks:
                    landmarks = results.pose_landmarks.landmark

                    # Get coordinates
                    left_ear = landmarks[mp_pose.PoseLandmark.LEFT_EAR]
                    right_ear = landmarks[mp_pose.PoseLandmark.RIGHT_EAR]
                    left_shoulder = landmarks[mp_pose.PoseLandmark.LEFT_SHOULDER]
                    right_shoulder = landmarks[mp_pose.PoseLandmark.RIGHT_SHOULDER]
        
                    metrics = (
                        f"{left_ear.x:.4f}|{left_ear.y:.4f}|{left_ear.z:.4f}|{left_ear.visibility:.4f}|"
                        f"{right_ear.x:.4f}|{right_ear.y:.4f}|{right_ear.z:.4f}|{right_ear.visibility:.4f}|"
                        f"{left_shoulder.x:.4f}|{left_shoulder.y:.4f}|{left_shoulder.z:.4f}|{left_shoulder.visibility:.4f}|"
                        f"{right_shoulder.x:.4f}|{right_shoulder.y:.4f}|{right_shoulder.z:.4f}|{right_shoulder.visibility:.4f}\r\n"
                    )

                    print(metrics)

                    conn.sendall(metrics.encode("utf-8"))

                if cv2.waitKey(1) & 0xFF == ord('q'):
                    break

    cap.release()

if __name__ == "__main__":
    main()

