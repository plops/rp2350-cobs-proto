#!/usr/bin/env python3
import sys
import time
import argparse
from cobs import cobs
import serial
from serial.tools import list_ports

# Import the generated protobuf message
try:
    import messages_pb2
except ImportError:
    print("Error: messages_pb2.py not found. Please compile the protobuf schema first:")
    print("  protoc --python_out=. messages.proto")
    sys.exit(1)

def find_serial_port():
    """Finds available serial ports and returns the most likely micro-controller port."""
    ports = list(list_ports.comports())
    if not ports:
        return None
    
    # Sort to prefer ACM or USB ports
    ports.sort(key=lambda p: ("ACM" in p.device or "USB" in p.device), reverse=True)
    return ports[0].device

def main():
    parser = argparse.ArgumentParser(description="Read RP2350 ADC values via COBS/Protobuf over Serial")
    parser.add_argument("-p", "--port", help="Serial port (e.g. /dev/ttyACM0, COM3). Auto-detects if omitted.")
    parser.add_argument("-b", "--baud", type=int, default=115200, help="Baud rate (default: 115200)")
    args = parser.parse_args()

    port = args.port
    if not port:
        port = find_serial_port()
        if not port:
            print("Error: No serial ports found. Is your RP2350 connected?")
            sys.exit(1)
        print(f"Auto-detected serial port: {port}")

    print(f"Connecting to {port} at {args.baud} baud...")
    try:
        ser = serial.Serial(port, args.baud, timeout=1.0)
    except Exception as e:
        print(f"Failed to open serial port {port}: {e}")
        sys.exit(1)

    print("Listening for ADC data. Press Ctrl+C to stop.")
    print("-" * 60)
    print(f"{'Timestamp (ms)':<15} | {'Raw ADC (12-bit)':<18} | {'Voltage (V)':<12}")
    print("-" * 60)

    raw_buffer = bytearray()
    
    try:
        while True:
            # Read whatever bytes are available
            data = ser.read(100)
            if not data:
                continue

            for byte in data:
                if byte == 0x00:
                    # We hit the COBS frame delimiter
                    if len(raw_buffer) > 0:
                        try:
                            # Decode COBS
                            decoded_data = cobs.decode(raw_buffer)
                            
                            # Parse Protobuf
                            reading = messages_pb2.AdcReading()
                            reading.ParseFromString(decoded_data)
                            
                            # Print in formatted layout
                            print(f"{reading.timestamp_ms:<15} | {reading.adc_raw:<18} | {reading.voltage:<12.4f}")
                            sys.stdout.flush()
                        except cobs.DecodeError:
                            print(f"[Warn] COBS decoding failed for packet length {len(raw_buffer)}", file=sys.stderr)
                        except Exception as e:
                            print(f"[Warn] Failed to parse Protobuf payload: {e}", file=sys.stderr)
                        
                        raw_buffer.clear()
                else:
                    raw_buffer.append(byte)
                    
    except KeyboardInterrupt:
        print("\nStopping receiver.")
    finally:
        ser.close()

if __name__ == "__main__":
    main()
