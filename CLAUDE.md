# CLAUDE.md

## Repository Purpose

Custom Rust firmware for ESP32 smart home display panels. Targets the Elecrow CrowPanel 7" Advanced V1.3 (ESP32-S3). Will eventually replace the ESPHome-based prototype in the home-network repo.

## Language & Toolchain

- **Language:** Rust (2021 edition)
- **Toolchain:** `esp` (Xtensa fork of Rust, installed via `espup`)
- **Target:** `xtensa-esp32s3-espidf` (ESP32-S3 with ESP-IDF std support)
- **SDK:** ESP-IDF v5.3 (downloaded automatically by `embuild` during first build)
- **Build:** `cargo build` (uses `build-std` to compile std library from source)

## Building

```bash
# Source ESP environment (required every terminal session)
. ~/export-esp.sh

# Build
cargo build

# Build + flash + monitor (requires USB connection)
cargo run
```

First build takes several minutes (downloads ESP-IDF, compiles C SDK). Incremental builds are ~7 seconds.

## Flashing

The CrowPanel's CH341 USB-serial chip doesn't work on macOS without a kernel extension. Flash via the Pi instead:

```bash
rsync -av target/xtensa-esp32s3-espidf/debug/display-firmware pi:/tmp/
ssh pi "docker run --rm --entrypoint '' -v /tmp:/firmware --device=/dev/ttyUSB0 \
  esphome/esphome:2026.2.1 esptool.py --chip esp32s3 --port /dev/ttyUSB0 \
  --baud 460800 write_flash 0x0 /firmware/display-firmware"
```

## Hardware: CrowPanel 7" Advanced V1.3

Pin mapping discovered during ESPHome bring-up (see notes vault `projects/esphome-display/`):

- **I2C:** SDA=GPIO15, SCL=GPIO16 (NOT GPIO19/20 — those are the expansion port)
- **Backlight:** I2C write to 0x30 (STC8H1K28 MCU). Value 0=max, 245=off.
- **Touch:** GT911 at I2C 0x5D on same bus
- **Display:** Parallel RGB. DE=42, HSYNC=40, VSYNC=41, PCLK=39. See README for full data pin list.
- **PSRAM:** 8MB octal, 80MHz
- **Boot button:** Puts device in download mode (stops firmware). Press reset to recover. Do NOT press during normal operation.

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `esp-idf-svc` | High-level Rust wrappers for ESP-IDF services (WiFi, HTTP, MQTT, NVS) |
| `esp-idf-hal` | Hardware abstraction (GPIO, SPI, I2C, timers) — pulled in by esp-idf-svc |
| `esp-idf-sys` | Raw FFI bindings to ESP-IDF C functions — pulled in by esp-idf-hal |
| `log` | Rust logging facade, bridged to ESP-IDF logging |
| `embuild` | Build script helper for ESP-IDF integration |

Future dependencies (not yet added):
- `lvgl-rs` or raw LVGL via FFI — display UI
- MQTT client — Home Assistant integration

## Project Goals

1. Get "Hello World" on the display (current milestone)
2. Initialize display with LVGL, render text
3. Add touch input
4. Add WiFi + MQTT for Home Assistant integration
5. Build a door status panel matching the ESPHome prototype
6. Evaluate: is custom Rust firmware better than ESPHome for this use case?

## Conventions

- Follow standard Rust conventions (rustfmt, clippy)
- Use `log` crate for all logging (bridges to ESP-IDF logging automatically)
- Unsafe blocks only when calling ESP-IDF C functions via FFI
- Keep hardware-specific code (pin numbers, I2C addresses) in constants, not scattered through the code

## Related Repos

- `home-network/esphome/crowpanel-front-door.yaml` — ESPHome prototype (working)
- `notes/projects/esphome-display/` — Research docs (LVGL, Rust, display tech, product landscape)
