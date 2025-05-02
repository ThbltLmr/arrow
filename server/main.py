import cv2
import mediapipe as mp
import socket

HOST = '0.0.0.0'
PORT = 9876

POSTURES = ["STRAIGHT", "SLOUCHING", "HEAD_TILT", "BODY_TILT"]

def get_depth_diff(left_ear, right_ear, left_shoulder, right_shoulder):
    avg_ear_depth = (left_ear.z + right_ear.z) / 2
    avg_shoulder_depth = (left_shoulder.z + right_shoulder.z) / 2
    depth_diff = abs(avg_ear_depth - avg_shoulder_depth)
    return depth_diff

def get_tilt(left_ear, right_ear, left_shoulder, right_shoulder):
    ear_slope = (left_ear.y - right_ear.y) / (left_ear.x - right_ear.x)
    shoulder_slope = (left_shoulder.y - right_shoulder.y) / (left_shoulder.x - right_shoulder.x)
    return ear_slope - shoulder_slope

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
        
                    # Should be < 0.33
                    depth_diff = get_depth_diff(left_ear, right_ear, left_shoulder, right_shoulder)
                    # Should be < 0.1
                    slope_diff = get_tilt(left_ear, right_ear, left_shoulder, right_shoulder)
                    # Should be < 0.05
                    shoulder_slope = (left_shoulder.y - right_shoulder.y) / (left_shoulder.x - right_shoulder.x)

                    message = POSTURES[0]
                    
                    if depth_diff > 0.33:
                        message = POSTURES[1]
                    elif slope_diff > 0.1:
                        message = POSTURES[2]
                    elif shoulder_slope > 0.05:
                        message = POSTURES[3]

                    print(message)
                    conn.sendall(f"{message}\n".encode("utf-8"))

                if cv2.waitKey(1) & 0xFF == ord('q'):
                    break

    cap.release()

if __name__ == "__main__":
    main()

