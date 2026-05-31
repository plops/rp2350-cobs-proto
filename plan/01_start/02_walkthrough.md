# Walkthrough - RP2350 ADC Protobuf & COBS System

We have successfully implemented the RP2350 ADC reader with Protobuf serialization, COBS packet framing, a Python receiver script managed by `uv`, and a GitHub Actions workflow that compiles the firmware and sets up a `tmate` SSH debugging session on failure or manual trigger.

## Changes Made

### 1. Embedded Rust Firmware
We initialized the Rust project at `/home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf` with the following configuration:
*   [Cargo.toml](file:///home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/Cargo.toml): Uses the `embassy-executor` async runtime, `embassy-rp` HAL (configured with `rp235xa` flavor and `time-driver` features), `prost` for Protobuf serialization, `cobs` for packet stuffing, and `embedded-alloc` for registering a global heap allocator (satisfying `prost`'s compilation requirement under `#![no_std]`).
*   [build.rs](file:///home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/build.rs): Compiles the protobuf schema at build-time with `btree_map` configured for `no_std` environments.
*   [memory.x](file:///home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/memory.x): Defined the linker layout for the RP2350 microcontroller.
*   [.cargo/config.toml](file:///home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/.cargo/config.toml): Targets `thumbv8m.main-none-eabihf` (Cortex-M33 hard float) using `flip-link` for stack overflow protection.
*   [src/messages.proto](file:///home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/src/messages.proto): Protobuf schema defining the `AdcReading` message structure.
*   [src/main.rs](file:///home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/src/main.rs): Microcontroller entry point that:
    1.  Sets up the heap allocator safely without creating mutable static references.
    2.  Configures `PIN_26` as an analog input (`ADC0`).
    3.  Initializes UART0 TX on `PIN_0` in blocking mode running at 115,200 baud.
    4.  Samples the ADC, calculates raw and voltage values, serializes the data to Protobuf, frames it with COBS (terminated with a `0x00` byte), and writes it to the UART TX.

### 2. Python Host Receiver
We created the host scripts under `/home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/host`:
*   [host/pyproject.toml](file:///home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/host/pyproject.toml): Configures project metadata and lists python dependencies (`pyserial`, `protobuf`, `cobs`) for easy execution with the `uv` tool.
*   [host/read_adc.py](file:///home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/host/read_adc.py): A CLI program that listens on a serial port (auto-detecting it if not specified), processes bytes looking for the COBS `0x00` frame delimiter, unstuffs the packet, and decodes the Protobuf payload to print real-time timestamp, raw ADC, and voltage readings.

### 3. GitHub Actions Workflow
*   [.github/workflows/build.yml](file:///home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/.github/workflows/build.yml): A workflow that checks out the code, installs target compiler packages, compiles the Rust project, and starts a `tmate` session if the build fails or if triggered manually. This allows secure, interactive SSH debugging on the buildserver during development.

---

## Validation & Verification Results

### 1. Python Environment Setup
We successfully verified that the Python project can be set up and run using `uv`:
```bash
$ uv run python read_adc.py --help
Using CPython 3.13.12 interpreter at: /usr/lib/python-exec/python3.13/python3
Creating virtual environment at: .venv
   Building cobs==1.2.2
      Built cobs==1.2.2
Installed 3 packages in 11ms
usage: read_adc.py [-h] [-p PORT] [-b BAUD]

Read RP2350 ADC values via COBS/Protobuf over Serial
...
```

### 2. Firmware Compilation
We verified that the Rust project compiles successfully without any warnings or linker errors:
```bash
$ cargo build --release
   Compiling rp2350-adc-protobuf v0.1.0 (/home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf)
    Finished `release` profile [optimized + debuginfo] target(s) in 0.88s
```
The compiled output is located at: `target/thumbv8m.main-none-eabihf/release/rp2350-adc-protobuf`

---

## How to Run locally

### Microcontroller
1.  Connect your RP2350 board using a hardware debugger (e.g. Raspberry Pi Debug Probe) or put it in bootloader mode.
2.  Install `probe-rs` or `picotool`.
3.  Flash the binary:
    ```bash
    cargo run --release
    ```

### Host Receiver
1.  Connect the microcontroller's UART TX (`GPIO0`) to your host using a USB-to-UART bridge.
2.  Navigate to the `host` directory and launch the receiver script:
    ```bash
    cd host
    uv run python read_adc.py
    ```
