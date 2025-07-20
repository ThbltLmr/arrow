#!/usr/bin/env python3
"""
Test server for Arrow posture monitoring - sends fake posture data for testing
"""

import socket
import time
import random

HOST = '127.0.0.1'
PORT = 9876

def generate_fake_metrics():
    """Generate fake posture metrics for testing"""
    # Generate fake coordinates for good/bad posture
    posture_type = random.choice(['good', 'bad'])
    
    if posture_type == 'good':
        # Straight posture - ears above shoulders
        left_ear = {'x': 0.3, 'y': 0.2, 'z': 0.5, 'visibility': 0.95}
        right_ear = {'x': 0.7, 'y': 0.2, 'z': 0.5, 'visibility': 0.95}
        left_shoulder = {'x': 0.3, 'y': 0.4, 'z': 0.0, 'visibility': 0.95}
        right_shoulder = {'x': 0.7, 'y': 0.4, 'z': 0.0, 'visibility': 0.95}
    else:
        # Bad posture - slouching
        left_ear = {'x': 0.3, 'y': 0.3, 'z': -0.1, 'visibility': 0.95}
        right_ear = {'x': 0.7, 'y': 0.3, 'z': -0.1, 'visibility': 0.95}
        left_shoulder = {'x': 0.3, 'y': 0.4, 'z': 0.2, 'visibility': 0.95}
        right_shoulder = {'x': 0.7, 'y': 0.4, 'z': 0.2, 'visibility': 0.95}
    
    # Format as expected by the client
    metrics = (
        f"{left_ear['x']:.4f}|{left_ear['y']:.4f}|{left_ear['z']:.4f}|{left_ear['visibility']:.4f}|"
        f"{right_ear['x']:.4f}|{right_ear['y']:.4f}|{right_ear['z']:.4f}|{right_ear['visibility']:.4f}|"
        f"{left_shoulder['x']:.4f}|{left_shoulder['y']:.4f}|{left_shoulder['z']:.4f}|{left_shoulder['visibility']:.4f}|"
        f"{right_shoulder['x']:.4f}|{right_shoulder['y']:.4f}|{right_shoulder['z']:.4f}|{right_shoulder['visibility']:.4f}\r\n"
    )
    
    return metrics, posture_type

def main():
    print("Starting test server for Arrow posture monitoring...")
    print(f"Listening on {HOST}:{PORT}")
    
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        s.bind((HOST, PORT))
        s.listen()
        print("Server listening...")
        
        try:
            while True:
                conn, addr = s.accept()
                print(f"Connected by {addr}")
                
                with conn:
                    try:
                        # Send test data every 2 seconds
                        while True:
                            metrics, posture_type = generate_fake_metrics()
                            print(f"Sending {posture_type} posture data")
                            conn.sendall(metrics.encode("utf-8"))
                            time.sleep(2)
                            
                    except (ConnectionResetError, BrokenPipeError):
                        print("Client disconnected")
                    except KeyboardInterrupt:
                        print("Stopping server...")
                        break
                        
        except KeyboardInterrupt:
            print("Server stopped by user")

if __name__ == "__main__":
    main()