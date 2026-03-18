# Display Firmware

Rust firmware for ESP32 smart home display panels. Built with ESP-IDF and Slint.

## Hardware

**Waveshare ESP32-P4-WIFI6-Touch-LCD-7B**

- **MCU:** ESP32-P4 (RISC-V 32-bit dual-core, 240MHz)
- **Display:** EK79007, MIPI DSI 2-lane, 1024×600 @ 60Hz
- **Touch:** GT911 on I2C (GPIO7 SDA, GPIO8 SCL)
- **WiFi:** Built-in ESP32-C6 co-processor via SDIO
- **Backlight:** GPIO32 LEDC PWM, 10-bit, 5kHz
- **PSRAM:** 32MB octal, 200MHz
- **Flash:** 32MB QIO
- **USB:** Native USB-C on P4 (no kernel extension needed on macOS)

See `CLAUDE.md` for full pin map and technical details.

## Dev Environment Setup

### Prerequisites

- macOS (Apple Silicon or Intel) or Linux
- [Homebrew](https://brew.sh) (macOS)
- Rust 1.75+

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### 2. Install ESP32 Rust Toolchain

```bash
# Install the toolchain manager
cargo install espup

# Install toolchains (Xtensa for S3, RISC-V for P4/C6)
espup install

# Install flash tool and linker wrapper
cargo install ldproxy espflash
```

### 3. Set Environment Variables

Add to `~/.zshrc` (or equivalent for your shell):

```bash
# ESP32 Rust toolchain
[ -f ~/export-esp.sh ] && . ~/export-esp.sh
```

Or source before each build:

```bash
. ~/export-esp.sh
```

### 4. Verify Setup

```bash
rustup target list | grep riscv32
# Should show: riscv32imafc-esp-espidf (installed)

rustc --print=sysroot
# Should include path to esp toolchain
```

## Build & Flash

### Build

```bash
cd firmware && cargo build
```

First build downloads ESP-IDF (~1GB) and compiles the C SDK. Subsequent builds are incremental (~7s).

### Flash

Connect the device via USB-C. Put it in download mode if needed:

1. Hold **Boot** button
2. Tap **Reset** button
3. Release **Boot**
4. Device enters download mode (USB enumeration, firmware frozen)

Then flash:

```bash
cd firmware && cargo run  # builds, flashes, and opens serial monitor
```

Or use espflash directly:

```bash
espflash flash target/riscv32imafc-esp-espidf/debug/firmware --monitor
```

## Project Structure

```
display-firmware/
├── Cargo.toml              (workspace manifest)
├── rust-toolchain.toml     (channel = "esp")
├── README.md               (this file)
├── CLAUDE.md               (developer reference)
│
├── bsp-waveshare-p4/       (board support package)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          (public interface)
│       ├── pins.rs         (GPIO constants, I2C, LEDC config)
│       └── display.rs      (MIPI DSI initialization)
│
└── firmware/               (application binary)
    ├── Cargo.toml
    ├── build.rs            (ESP-IDF build integration)
    ├── sdkconfig.defaults  (ESP-IDF configuration)
    ├── .cargo/config.toml  (target, linker flags)
    ├── ui/
    │   └── main.slint      (Slint UI definition)
    └── src/
        ├── main.rs         (entry point, main loop)
        └── slint_backend.rs (Slint + display + input integration)
```

## Architecture

```
Application (Rust + Slint)
    ├── slint          (UI framework — software renderer)
    ├── esp-idf-svc    (WiFi, HTTP, MQTT — via C6 co-processor)
    ├── esp-idf-hal    (GPIO, I2C, LEDC, display drivers)
    └── esp-idf-sys    (Raw FFI bindings to ESP-IDF C SDK)
        └── ESP-IDF v5.4.3 (Espressif's C SDK, auto-compiled)
            └── FreeRTOS + lwIP + mbedTLS + drivers
                └── ESP32-P4 Hardware
```

## Pin Reference

See `bsp-waveshare-p4/src/pins.rs` for all constants. Key pins:

| Function | GPIO | I2C Address | Notes |
|----------|------|-------------|-------|
| I2C SDA | 7 | — | Shared bus: touch, audio |
| I2C SCL | 8 | — | Shared bus: touch, audio |
| LCD Reset | 33 | — | Active low |
| LCD Backlight | 32 | — | LEDC PWM, 5kHz, 10-bit |
| GT911 Touch | 7/8 | 0x5D | Milestone 2 |
| | | 0x14 | (backup address) |

**Display Timing (EK79007, 1024×600):**

```
HSYNC: back_porch=160, front_porch=160, pulse_width=1
VSYNC: back_porch=23, front_porch=12, pulse_width=1
Pixel Clock: 80 MHz
```

**MIPI DSI:**
```
Lanes: 2
Bitrate: 1000 Mbps/lane
LDO: Channel 3, 2500mV
```

## Compile Targets

The workspace uses `riscv32imafc-esp-espidf` as the default target. Verify in `.cargo/config.toml`:

```toml
[build]
target = "riscv32imafc-esp-espidf"

[target.riscv32imafc-esp-espidf]
runner = "espflash flash --monitor"
```

## Resources

- [The Rust on ESP Book](https://docs.esp-rs.org/book/)
- [ESP-IDF Programming Guide (v5.4)](https://docs.espressif.com/projects/esp-idf/en/stable/esp32p4/)
- [esp-idf-svc docs](https://docs.rs/esp-idf-svc/)
- [esp-idf-hal docs](https://docs.rs/esp-idf-hal/)
- [Slint Docs](https://slint.dev/docs)
- [Waveshare ESP32-P4 Components](https://github.com/waveshareteam/Waveshare-ESP32-components)
- [GT911 Touch Controller](https://www.goodix.com/en/product/detail/GT911)
- [EK79007 Display Panel](https://www.egyption-experts.com/EK79007/)
