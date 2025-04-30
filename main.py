import cv2
import mediapipe as mp
import math

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
    mp_drawing = mp.solutions.drawing_utils

    # Open Webcam
    cap = cv2.VideoCapture(0)

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

            # Set default status
            color = (0, 255, 0)  # Green

            # Evaluate posture
            if depth_diff > 0.33:
                color = (0, 0, 255)  # Red
            elif depth_diff > 0.3:
                color = (0, 255, 255)  # Yellow

            # Draw slouch meter bar
            bar_length = int(min(300, max(0, (1.0 + depth_diff) * 150)))  # scale for visualization


            cv2.rectangle(frame, (50, 50), (50 + bar_length, 80), color, -1)
            cv2.putText(frame, "slouch: {slouch:.2f}, tilt: {tilt:.2f}, shoulder: {shoulder:.2f}".format(slouch=depth_diff, tilt=slope_diff, shoulder=shoulder_slope), (50, 45), cv2.FONT_HERSHEY_SIMPLEX, 0.5, color, 2)

            # Optional: Draw pose landmarks
            mp_drawing.draw_landmarks(
                frame, results.pose_landmarks, mp_pose.POSE_CONNECTIONS)

        # Show frame
        cv2.imshow('Slouch Meter', frame)

        if cv2.waitKey(1) & 0xFF == ord('q'):
            break

    cap.release()
    cv2.destroyAllWindows()

if __name__ == "__main__":
    main()

