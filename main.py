import cv2
import mediapipe as mp

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
    if results.pose_landmarks:
        print("Pose landmarks detected:")
        print(results.pose_landmarks)
        # You can optionally save the original image too if needed
        # cv2.imwrite('captured_image.jpg', frame)
    else:
        print("No pose detected in the image.")

else:
    print("Failed to capture image")

# Release the camera and MediaPipe Pose resources
cap.release()
pose.close() # Close the pose model

