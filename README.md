# Display Firmware

Rust firmware for ESP32 smart home display panels. Built with ESP-IDF and LVGL.

## Hardware

Currently targeting **Elecrow CrowPanel 7" Advanced V1.3** (ESP32-S3, 800x480 IPS, GT911 touch).

See the [home-network repo](https://github.com/teleios-labs/home-network) for the ESPHome-based prototype that this project aims to replace with custom Rust firmware.

## Dev Environment Setup

### Prerequisites

- macOS (Apple Silicon or Intel)
- [Homebrew](https://brew.sh)

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### 2. Install ESP32 Rust Toolchain

```bash
# Install the toolchain manager
cargo install espup

# Install Xtensa (ESP32-S3) and RISC-V (ESP32-P4/C6) toolchains
espup install

# Install the linker wrapper and flash tool
cargo install ldproxy espflash
```

### 3. Set Environment Variables

Every terminal session needs the ESP environment. Add to your `~/.zshrc`:

```bash
# ESP32 Rust toolchain
[ -f ~/export-esp.sh ] && . ~/export-esp.sh
```

Or source it manually before building:

```bash
. ~/export-esp.sh
```

### 4. Build

```bash
cargo build
```

First build downloads ESP-IDF (~1GB) and compiles the entire SDK. Subsequent builds are incremental (~7s).

### 5. Flash

With the device connected via USB:

```bash
cargo run  # builds, flashes, and opens serial monitor
```

Or flash manually:

```bash
espflash flash target/xtensa-esp32s3-espidf/debug/display-firmware --monitor
```

#### Flashing via Pi (no USB driver on Mac)

The CrowPanel uses a CH341 USB-serial chip that requires a kernel extension on macOS. Easier to flash through the Pi:

```bash
# Copy binary to Pi
rsync -av target/xtensa-esp32s3-espidf/debug/display-firmware pi:/tmp/

# Flash from Pi (device on /dev/ttyUSB0)
ssh pi "docker run --rm --entrypoint '' -v /tmp:/firmware --device=/dev/ttyUSB0 \
  esphome/esphome:2026.2.1 esptool.py --chip esp32s3 --port /dev/ttyUSB0 \
  --baud 460800 write_flash 0x0 /firmware/display-firmware"
```

## Project Structure

```
display-firmware/
├── .cargo/config.toml    # Build target, linker, ESP-IDF version
├── Cargo.toml            # Rust dependencies
├── build.rs              # ESP-IDF build script integration
├── rust-toolchain.toml   # Use 'esp' Xtensa toolchain
├── sdkconfig.defaults    # ESP-IDF chip configuration
└── src/
    └── main.rs           # Entry point
```

## Architecture

```
Rust Application Code
    ├── esp-idf-svc    (Rust bindings to ESP-IDF services: WiFi, HTTP, MQTT)
    ├── esp-idf-hal    (Rust bindings to ESP-IDF hardware: GPIO, SPI, I2C)
    └── esp-idf-sys    (Raw FFI bindings to ESP-IDF C SDK)
        └── ESP-IDF v5.3 (Espressif's C SDK, compiled automatically)
            └── FreeRTOS + lwIP + mbedTLS + drivers
                └── ESP32-S3 Hardware
```

## CrowPanel 7" V1.3 Pin Reference

```
I2C Bus: SDA=GPIO15, SCL=GPIO16
  0x30 — STC8H1K28 backlight (write 0 for max, 245 for off)
  0x5D — GT911 touch controller

Display (parallel RGB):
  DE=GPIO42, HSYNC=GPIO40, VSYNC=GPIO41, PCLK=GPIO39
  Blue:  GPIO21, GPIO47, GPIO48, GPIO45, GPIO38
  Green: GPIO9, GPIO10, GPIO11, GPIO12, GPIO13, GPIO14
  Red:   GPIO7, GPIO17, GPIO18, GPIO3, GPIO46
```

## Resources

- [The Rust on ESP Book](https://docs.esp-rs.org/book/)
- [esp-idf-svc docs](https://docs.rs/esp-idf-svc)
- [esp-idf-hal docs](https://docs.rs/esp-idf-hal)
- [LVGL Rust bindings](https://github.com/rafaelcaricio/lvgl-rs)
- [ESP-IDF Programming Guide](https://docs.espressif.com/projects/esp-idf/en/stable/esp32s3/)
