# RP2350 ADC Protobuf & COBS System

This repository contains:
1.  **Rust Firmware**: Embedded Rust code that reads the analog-to-digital converter (ADC) on the RP2350 microcontroller and transmits values over serial using Protobuf serialization and COBS framing.
2.  **Python Host Receiver**: CLI program that reads from the serial port, decodes the COBS frames, and parses the Protobuf payload.
3.  **Release Automation**: A `scripts/release.sh` script to automate semantic version tagging.
4.  **GitHub Actions CI**: Workflow configuring automated compilation and SSH debugging with tmate.

---

## Running the GitHub Actions Workflow

The GitHub Actions CI/CD workflow is defined in `.github/workflows/build.yml`. It automatically builds the firmware target and provides interactive SSH debugging support.

### Triggering the Action
There are three ways to trigger the workflow on GitHub:
1.  **On Push / Pull Request**: Every time you push a commit or open a pull request targeting `main` or `master`, the workflow runs.
2.  **On Release Tag**: The release script (`release.sh`) tags your releases, which triggers release builds when pushed.
3.  **Manual Trigger (Workflow Dispatch)**:
    *   Navigate to the **Actions** tab on your GitHub repository page.
    *   Select the **Rust Firmware Build** workflow in the left sidebar.
    *   Click the **Run workflow** dropdown button.
    *   Select the branch (e.g., `main`) and click the **Run workflow** button.

### SSH Debugging the Build Server (tmate)
If the compilation fails or if the workflow was triggered manually (via *Workflow Dispatch*), the runner launches an interactive `tmate` session allowing you to SSH into the build server to inspect the environment, compiler, or build artifacts.

To access the build server via SSH:
1.  Wait for the workflow run to start.
2.  Click on the active run under the **Actions** tab.
3.  Open the logs for the **build** job and click on the **Setup tmate session for SSH debugging** step.
4.  Copy the printed SSH connection string (looks like `ssh -p <port> <session-id>@ubuntu-latest.tmate.io` or similar).
5.  Run that command in your local terminal. You will be connected directly to the bash shell on the GitHub runner container.
6.  *Security note:* By default, the workflow sets `limit-access-to-actor: true`, meaning only the GitHub user who triggered the run can connect using their registered SSH public keys.

---

## Local Setup & Quick Start

### 1. Embedded Firmware (Rust)
Ensure you have the Rust toolchain installed:
```bash
# Add the target architecture
rustup target add thumbv8m.main-none-eabihf

# Install flip-link for stack protection
cargo install flip-link

# Build the firmware
cargo build --release
```
The compiled binary will be located at `target/thumbv8m.main-none-eabihf/release/rp2350-adc-protobuf`.

### 2. Host Program (Python)
We use `uv` to manage Python environments and execute the receiver:
```bash
cd host
# View command usage/options
uv run python read_adc.py --help

# Run the receiver (auto-detects serial ports)
uv run python read_adc.py
```

### 3. Creating a Release
Use the release script to automate version bumping and tag creation:
```bash
./scripts/release.sh 0.1.0
```
Then push the tags to your GitHub repository to trigger the action:
```bash
git push origin main --tags
```
