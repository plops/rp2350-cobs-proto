# RP2350 ADC Protobuf & COBS Communication

This project implements a complete system to read analog-to-digital converter (ADC) values on a Raspberry Pi RP2350 microcontroller and transmit the readings over serial to a Python program running on a Linux host. The data transmission uses Protocol Buffers (Protobuf) for serialization and Consistent Overhead Byte Stuffing (COBS) for frame packetization. It also sets up a GitHub Action workflow to build the Rust firmware on a remote server with support for interactive SSH debugging during development.

## User Review Required

> [!IMPORTANT]
> The target chip is the RP2350 (Cortex-M33). The target Rust triple is `thumbv8m.main-none-eabihf`.
> The GitHub Action setup uses `mxschmitt/action-tmate` for interactive SSH debugging. For security, access is limited to the actor triggering the workflow.

## Open Questions

> [!NOTE]
> 1. Which ADC pin would you like to use? The default implementation will read from GPIO26 (`ADC0`), but this is easily adjustable in `main.rs`.
> 2. What UART pins will be connected to the host? We will default to UART0 on GPIO0 (TX) and GPIO1 (RX), running at 115,200 baud.

## Proposed Changes

We will create a new directory `rp2350-adc-protobuf` inside `/home/kiel/.gemini/antigravity/scratch/` containing the Rust microcontroller project, the Python host receiver, and the GitHub workflow config.

---

### Rust Firmware Component

#### [NEW] [Cargo.toml](file:///home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/Cargo.toml)
Declares the workspace / crate and its dependencies (`embassy-rp`, `prost`, `cobs`, `embedded-alloc`, etc.) using exact stable versions.

#### [NEW] [build.rs](file:///home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/build.rs)
Configures `prost-build` to automatically compile `src/messages.proto` into Rust modules.

#### [NEW] [memory.x](file:///home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/memory.x)
Linker layout for RP2350 with 4MB of Flash (origin `0x10000000`) and 520KB of SRAM (origin `0x20000000`).

#### [NEW] [.cargo/config.toml](file:///home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/.cargo/config.toml)
Defines target `thumbv8m.main-none-eabihf`, compiler flags for `flip-link` and linker scripts, and the default runner.

#### [NEW] [src/messages.proto](file:///home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/src/messages.proto)
Protobuf schema defining the structure of the ADC readings (timestamp, raw reading, voltage).

#### [NEW] [src/main.rs](file:///home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/src/main.rs)
Microcontroller entry point utilizing the `embassy-executor` async runtime. It configures the ADC pin, UART transmitter, sets up the global memory allocator, and enters a reading-transmitting loop.

#### [NEW] [deps.md](file:///home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/deps.md)
Contains the GitHub organization/repo paths for the dependencies of our rust crate to adhere to the repository convention.

---

### Python Host Component

#### [NEW] [host/pyproject.toml](file:///home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/host/pyproject.toml)
Python project dependencies (`pyserial`, `protobuf`, `cobs`) configured for management using `uv`.

#### [NEW] [host/read_adc.py](file:///home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/host/read_adc.py)
Host script that listens on the serial port, searches for COBS frame delimiters, decodes the packages, deserializes the Protobuf, and prints the data.

---

### GitHub Action Component

#### [NEW] [.github/workflows/build.yml](file:///home/kiel/.gemini/antigravity/scratch/rp2350-adc-protobuf/.github/workflows/build.yml)
GitHub Action workflow file that builds the Rust firmware for `thumbv8m.main-none-eabihf` and starts an interactive `tmate` session for SSH debugging.

---

## Verification Plan

### Automated Build Verification
1. We will verify the Rust project compiles locally (or compile check it for target `thumbv8m.main-none-eabihf` using cargo).
2. We will check that the python host environment can be initialized and dependencies installed with `uv`.

### Manual Hardware Verification
1. Flash the compiled ELF binary onto the RP2350 board.
2. Run the python script on the host to monitor incoming sensor data.
