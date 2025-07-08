import socket

HOST = '127.0.0.1'  # Localhost
PORT = 65432        # Non-privileged port > 1023

def start_tcp_server():
    # Create a socket using IPv4 and TCP
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind((HOST, PORT))  # Bind the socket to the address
        s.listen()            # Enable the server to accept connections
        print(f"Server listening on {HOST}:{PORT}")

        while True:
            conn, addr = s.accept()  # Accept a new connection
            with conn:
                print(f"Connected by {addr}")
                while True:
                    data = conn.recv(1024)
                    if not data:
                        break  # Break loop if no data is received
                    print(f"Received: {data.decode()}")
                    conn.sendall(data)  # Echo the received data back

if __name__ == '__main__':
    start_tcp_server()
