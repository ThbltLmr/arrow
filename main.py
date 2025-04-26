import cv2
import mediapipe as mp

def main():
    # Initialize MediaPipe Pose
    mp_pose = mp.solutions.pose
    pose = mp_pose.Pose(static_image_mode=True, min_detection_confidence=0.5)
    
    # Open the webcam (0 is usually the default camera)
    cap = cv2.VideoCapture(0)
    
    # Wait for the camera to initialize and capture a frame
    ret, frame = cap.read()
    
    if ret:
        # Convert the BGR image to RGB before processing
        image_rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
    
        # Process the image and find pose landmarks
        results = pose.process(image_rgb)
    
        # Print pose landmarks if detected
        landmarks = results.pose_landmarks
        if landmarks:
            # Access landmarks by their index based on the MediaPipe Pose documentation
            left_shoulder = landmarks.landmark[mp_pose.PoseLandmark.LEFT_SHOULDER.value]
            right_shoulder = landmarks.landmark[mp_pose.PoseLandmark.RIGHT_SHOULDER.value]
            left_ear = landmarks.landmark[mp_pose.PoseLandmark.LEFT_EAR.value]
            right_ear = landmarks.landmark[mp_pose.PoseLandmark.RIGHT_EAR.value]
        
            print(f"Left Shoulder: x={left_shoulder.x}, y={left_shoulder.y}, z={left_shoulder.z}")
            print(f"Right Shoulder: x={right_shoulder.x}, y={right_shoulder.y}, z={right_shoulder.z}")
            print(f"Left Ear: x={left_ear.x}, y={left_ear.y}, z={left_ear.z}")
            print(f"Right Ear: x={right_ear.x}, y={right_ear.y}, z={right_ear.z}")
        else:
            print("No pose detected in the image.")
    
    else:
        print("Failed to capture image")
    
    # Release the camera and MediaPipe Pose resources
    cap.release()
    pose.close()

if __name__ == "__main__":
    main()
