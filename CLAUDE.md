# CLAUDE.md

## Repository Purpose

Custom Rust firmware for ESP32 smart home display panels. Targets the **Waveshare ESP32-P4-WIFI6-Touch-LCD-7B** with a 7" MIPI DSI display (1024×600 EK79007), GT911 touch input, and built-in WiFi (ESP32-C6 co-processor). Renders UI with the Slint framework.

## Language & Toolchain

- **Language:** Rust (2021 edition)
- **Toolchain:** `esp` (installed via `espup` — supports both Xtensa and RISC-V)
- **Target:** `riscv32imafc-esp-espidf` (ESP32-P4 with ESP-IDF std support)
- **SDK:** ESP-IDF v5.4.3 (downloaded automatically by `embuild` during first build)
- **Build:** `cd firmware && cargo build` (uses `build-std` to compile std library from source)

## Building

```bash
# Source ESP environment (required every terminal session)
. ~/export-esp.sh

# Build
cd firmware && cargo build

# Build + flash + monitor (requires USB connection)
cd firmware && cargo run
```

First build takes several minutes (downloads ESP-IDF, compiles C SDK). Incremental builds are ~7 seconds.

## Flashing

The Waveshare board has native USB on the ESP32-P4 chip. No kernel extension or external relay needed.

```bash
cd firmware && cargo run  # builds, flashes, and opens serial monitor
```

If the device won't flash, put it in download mode:
1. Hold the **Boot** button
2. Tap the **Reset** button
3. Release **Boot**
4. Device will appear in download mode; `cargo run` will flash it

## Workspace Layout

```
display-firmware/
├── Cargo.toml              (workspace manifest)
├── rust-toolchain.toml     (channel = "esp")
├── bsp-waveshare-p4/       (board support package — pins, display config)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          (public interface)
│       ├── pins.rs         (pin constants: GPIO, I2C, LEDC)
│       └── display.rs      (MIPI DSI + framebuffer init)
└── firmware/               (application binary)
    ├── Cargo.toml
    ├── build.rs            (ESP-IDF build script)
    ├── sdkconfig.defaults  (ESP-IDF configuration)
    ├── .cargo/config.toml  (target, linker, build-std)
    ├── ui/
    │   └── main.slint      (Slint UI definition)
    └── src/
        ├── main.rs         (application entry point)
        └── slint_backend.rs (display + input integration)
```

## Hardware: Waveshare ESP32-P4-WIFI6-Touch-LCD-7B

Full pin map in `bsp-waveshare-p4/src/pins.rs`:

**I2C (GPIO7 SDA, GPIO8 SCL):**
- 0x5D — GT911 touch controller (Milestone 2)
- Audio co-processors (ES8311/ES7210) — future

**Display:**
- **LCD Reset:** GPIO33
- **LCD Backlight:** GPIO32 (LEDC PWM, 5kHz, 10-bit)
- **Panel:** EK79007 MIPI DSI, 1024×600 @ 60Hz
- **DSI Interface:** 2 lanes, 1000 Mbps/lane
- **Timing:** HSYNC back=160, front=160, pulse=1; VSYNC back=23, front=12, pulse=1; PCLK=80MHz

**System:**
- **WiFi:** ESP32-C6 co-processor via SDIO (esp_hosted) — Milestone 2
- **PSRAM:** 32MB octal, 200MHz
- **Flash:** 32MB QIO
- **USB:** Native USB-C, direct on P4 chip

**Boot Mode:**
- Pressing **Boot** and tapping **Reset** puts device in download mode (USB enumeration only, firmware frozen). Release Boot or press Reset again to exit.

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `esp-idf-svc = "0.52"` | High-level Rust wrappers for ESP-IDF services (WiFi via C6, HTTP, MQTT, NVS) |
| `esp-idf-hal` | Hardware abstraction (GPIO, I2C, LEDC PWM, display drivers) — pulled in by esp-idf-svc |
| `esp-idf-sys` | Raw FFI bindings to ESP-IDF C functions — pulled in by esp-idf-hal |
| `slint = "1.15"` | Cross-platform UI framework (software renderer via framebuffer) |
| `log` | Rust logging facade, bridged to ESP-IDF logging |
| `embuild = "0.33"` | Build script helper for ESP-IDF integration |

Future dependencies:
- `esp_hosted` — C6 co-processor WiFi driver (Milestone 2)
- MQTT client — Home Assistant integration (Milestone 3)

## Project Goals (Milestones)

1. **Pixels on screen** (current) ✓
   - Slint "Hello World" rendered via MIPI DSI to display
2. **OTA updates over WiFi** (next)
   - WiFi via C6 co-processor + ESP-IDF WiFi stack
   - HTTP server for firmware updates
3. **Home Assistant integration**
   - MQTT client, door/sensor status, remote control
4. **Production panel**
   - Full UI, reliability testing, field deployment

## Conventions

- Follow standard Rust conventions (`rustfmt`, `clippy`)
- Pin constants live in `bsp-waveshare-p4/src/pins.rs`, not scattered through application code
- Unsafe blocks only when calling ESP-IDF C functions via FFI
- Use `log` crate for all logging (bridges to ESP-IDF logging automatically)
- Slint UI definitions in `.slint` files under `firmware/ui/`; keep logic in Rust

## Related Repos

- `home-network` — Home Assistant integration, MQTT broker, network config
- `notes/projects/esphome-display/` — Research docs (display tech, Rust ecosystem, product comparison)
