import serial
import time

# Protocol constants
START_MARKER = 0xF0
END_MARKER = 0xF1
ESCAPE_CHAR = 0xF2
ACK = 0x06

# Open serial port
ser = serial.Serial('/dev/tty.usbserial-A5069RR4', 115200)  # Adjust port as needed
time.sleep(2)  # Wait for connection to establish

def send_transaction(tx_data):
    # Send start marker
    ser.write(bytes([START_MARKER]))
    
    # Send data with escaping
    for byte in tx_data:
        if byte in [START_MARKER, END_MARKER, ESCAPE_CHAR]:
            ser.write(bytes([ESCAPE_CHAR, byte]))
        else:
            ser.write(bytes([byte]))
    
    # Send end marker
    ser.write(bytes([END_MARKER]))
    
    # Wait for acknowledgment
    while True:
        if ser.in_waiting > 0:
            response = ser.read()
            if response[0] == ACK:
                print("Transaction acknowledged by device")
                break
            else:
                print(f"{response.decode('ascii', errors='ignore')}", end='')

# Example Bitcoin transaction (this would be your actual transaction bytes)
mock_tx = bytes.fromhex("0100000001a15d57094aa7a21a28cb20b59aab8fc7d1149a3bdbcddba9c622e4f5f6a99ece010000006c493046022100f93bb0e7d8db7bd46e40132d1f8242026e045f03a0efe71bbb8e3f475e970d790221009337cd7f1f929f00cc6ff01f03729b069a7c21b59b1736ddfee5db5946c5da8c0121033b9b137ee87d5a812d6f506efdd37f0affa7ffc310711c06c7f3e097c9447c52ffffffff0100e1f505000000001976a9140389035a9225b3839e2bbf32d826a1e222031fd888ac00000000")

# Send the transaction
send_transaction(mock_tx)

# Close the connection
ser.close()
