import cv2

# Open the webcam (0 is usually the default camera)
cap = cv2.VideoCapture(0)

# Wait for the camera to initialize
ret, frame = cap.read()

if ret:
    # Save the captured frame to a file
    cv2.imwrite('captured_image.jpg', frame)
    print("Image saved as captured_image.jpg")
else:
    print("Failed to capture image")

# Release the camera
cap.release()

