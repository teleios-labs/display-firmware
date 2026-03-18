# ESP32-P4 Migration Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Retarget the display-firmware repo from ESP32-S3 to ESP32-P4 (Waveshare ESP32-P4-WIFI6-Touch-LCD-7B) and render pixels on screen using Slint.

**Architecture:** Cargo workspace with two crates — `bsp-waveshare-p4` (board support: MIPI DSI display init, pin constants, backlight) and `firmware` (Slint UI application). The BSP handles all hardware-specific ESP-IDF calls; the firmware crate owns the UI and event loop.

**Tech Stack:** Rust (esp toolchain), ESP-IDF v5.4.x, `esp-idf-svc`/`esp-idf-sys`, Slint (software renderer with custom Platform impl), target `riscv32imafc-esp-espidf`.

**Spec:** `notes/projects/display-firmware/2026-03-17-p4-migration-design.md`

**Hardware reference (from Waveshare BSP):**
- Display: EK79007, MIPI DSI 2-lane, 1024x600, `esp_lcd_ek79007` driver
- Touch: GT911, I2C at 0x5D (GPIO7 SDA, GPIO8 SCL) — Milestone 2
- Backlight: LEDC PWM on GPIO32, 5kHz, inverted output
- LCD Reset: GPIO33
- MIPI DSI PHY power: LDO channel 3, 2500mV
- WiFi: ESP32-C6 co-proc via SDIO (`esp_hosted`) — Milestone 2

---

## File Map

### New files

| File | Responsibility |
|------|---------------|
| `Cargo.toml` (root) | Workspace manifest, shared profiles |
| `bsp-waveshare-p4/Cargo.toml` | BSP crate dependencies |
| `bsp-waveshare-p4/src/lib.rs` | `Board` struct, `init()`, re-exports |
| `bsp-waveshare-p4/src/pins.rs` | Pin constants, I2C addresses, display timing |
| `bsp-waveshare-p4/src/display.rs` | MIPI DSI init via ESP-IDF, backlight, `DisplayHandle` |
| `firmware/ui/main.slint` | Slint UI definition (hello world) |
| `firmware/src/slint_backend.rs` | Slint `Platform` trait impl for ESP-IDF framebuffer |

### Moved files (from repo root → `firmware/`)

| From | To |
|------|----|
| `src/main.rs` | `firmware/src/main.rs` |
| `build.rs` | `firmware/build.rs` |
| `.cargo/config.toml` | `firmware/.cargo/config.toml` |
| `Cargo.toml` | `firmware/Cargo.toml` (rewritten) |
| `sdkconfig.defaults` | `firmware/sdkconfig.defaults` (rewritten for P4) |

### Files staying at root

| File | Change |
|------|--------|
| `rust-toolchain.toml` | Stays at root, unchanged (applies to all workspace members) |
| `CLAUDE.md` | Rewritten for P4 |
| `README.md` | Rewritten for P4 |
| `.gitignore` | Unchanged |

---

## Task 0: Validate Slint MCU Support for ESP32-P4

**This is a research gate. Complete before any code changes.**

**Files:** None (research only)

- [ ] **Step 1: Check Slint crate for MCU/software renderer support**

Check the `slint` crate on crates.io for these features:
- `renderer-software` — software renderer (no GPU needed)
- `compat-1-2` or similar compatibility features

Run: `cargo search slint --limit 5`

Verify that the `slint` crate publishes a `platform` module with:
- `slint::platform::Platform` trait
- `slint::platform::software_renderer::MinimalSoftwareWindow`
- `slint::platform::software_renderer::Rgb565Pixel` (or similar pixel type)

Check: https://docs.rs/slint/latest/slint/platform/index.html

- [ ] **Step 2: Check for existing ESP-IDF platform implementations**

Search GitHub for `slint platform esp-idf` and `slint esp32 mcu`:
- https://github.com/slint-ui/slint — check `examples/mcu-board-support/` directory
- Look for any `esp32p4` or `esp-idf` examples in the Slint repo
- Check if `slint::platform::software_renderer` can render into a raw `&mut [Rgb565Pixel]` buffer

- [ ] **Step 3: Go/no-go decision**

**Go** if: Slint crate has `Platform` trait + `MinimalSoftwareWindow` + software renderer that can render into a raw pixel buffer. We implement the Platform adapter ourselves.

**No-go** if: The MCU platform API is not published on crates.io, or requires nightly-only features incompatible with the `esp` toolchain. **Fallback:** Use raw ESP-IDF framebuffer fill (solid color) for Milestone 1, add Slint in a later milestone when support matures.

- [ ] **Step 4: Document findings**

Add a note to the spec with: Slint version to use, exact feature flags, pixel format the software renderer outputs, and whether a fallback was needed.

---

## Task 1: Workspace Restructure

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `firmware/Cargo.toml`
- Create: `firmware/.cargo/config.toml`
- Create: `firmware/build.rs`
- Create: `firmware/src/main.rs`
- Create: `bsp-waveshare-p4/Cargo.toml`
- Create: `bsp-waveshare-p4/src/lib.rs`
- Delete: `src/main.rs`, `build.rs`, `.cargo/config.toml`, `Cargo.toml` (old)
- Keep: `rust-toolchain.toml` at root

- [ ] **Step 1: Create workspace root `Cargo.toml`**

```toml
[workspace]
members = ["bsp-waveshare-p4", "firmware"]
resolver = "2"

[profile.release]
opt-level = "s"

[profile.dev]
debug = true
opt-level = "z"
```

- [ ] **Step 2: Create `bsp-waveshare-p4/Cargo.toml`**

```toml
[package]
name = "bsp-waveshare-p4"
version = "0.1.0"
edition = "2021"

[dependencies]
log = "0.4"
esp-idf-svc = "0.52"
```

- [ ] **Step 3: Create `bsp-waveshare-p4/src/lib.rs`**

```rust
pub mod pins;

/// Board peripherals. Constructed via `Board::init()`.
pub struct Board;

impl Board {
    /// Initialize board hardware.
    /// Currently a placeholder — display init added in Task 3.
    pub fn init() -> Result<Self, Box<dyn std::error::Error>> {
        log::info!("Board init (placeholder)");
        Ok(Self)
    }
}
```

- [ ] **Step 4: Create `bsp-waveshare-p4/src/pins.rs`**

```rust
// Placeholder — filled in during Task 2.
```

- [ ] **Step 5: Move and rewrite `firmware/` files**

Move existing files into `firmware/`:

`firmware/build.rs` — same content as current `build.rs`:
```rust
fn main() {
    embuild::espidf::sysenv::output();
}
```

`firmware/src/main.rs` — updated for P4:
```rust
use bsp_waveshare_p4::Board;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Display firmware starting on ESP32-P4...");

    let _board = Board::init().expect("board init failed");

    loop {
        log::info!("Still alive!");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
```

`firmware/Cargo.toml`:
```toml
[package]
name = "display-firmware"
version = "0.1.0"
edition = "2021"
resolver = "2"

[[bin]]
name = "display-firmware"
harness = false

[dependencies]
log = "0.4"
anyhow = "1"
esp-idf-svc = "0.52"
bsp-waveshare-p4 = { path = "../bsp-waveshare-p4" }

[build-dependencies]
embuild = "0.33"
```

- [ ] **Step 6: Create `firmware/.cargo/config.toml`**

```toml
[build]
target = "riscv32imafc-esp-espidf"

[target.riscv32imafc-esp-espidf]
linker = "ldproxy"
runner = "espflash flash --monitor"

[unstable]
build-std = ["std", "panic_abort"]

[env]
MCU = "esp32p4"
ESP_IDF_VERSION = "v5.4.1"
```

**Important:** Verify `v5.4.1` exists as a git tag in `https://github.com/espressif/esp-idf`. If not, use the latest `v5.4.x` tag. Run:
```bash
git ls-remote --tags https://github.com/espressif/esp-idf 'refs/tags/v5.4*'
```

- [ ] **Step 7: Create `firmware/sdkconfig.defaults`**

Source from Waveshare BSP. Key entries for P4 + MIPI DSI:

```
# ESP32-P4 target
CONFIG_IDF_TARGET="esp32p4"

# Flash
CONFIG_ESPTOOLPY_FLASHMODE_QIO=y
CONFIG_ESPTOOLPY_FLASHSIZE_32MB=y

# PSRAM (required for display framebuffer)
CONFIG_SPIRAM=y
CONFIG_SPIRAM_SPEED_200M=y

# Cache (MIPI DSI DMA performance)
CONFIG_CACHE_L2_CACHE_256KB=y
CONFIG_CACHE_L2_CACHE_LINE_128B=y

# Required for MIPI DSI
CONFIG_IDF_EXPERIMENTAL_FEATURES=y

# Stack size (Slint needs headroom)
CONFIG_ESP_MAIN_TASK_STACK_SIZE=10240

# Performance
CONFIG_COMPILER_OPTIMIZATION_PERF=y
CONFIG_FREERTOS_HZ=1000
```

- [ ] **Step 8: Delete old root-level source files**

Remove:
- `src/main.rs`
- `src/` directory
- `build.rs`
- `.cargo/config.toml`
- `.cargo/` directory
- Old `Cargo.toml` (replaced by workspace manifest)
- `sdkconfig.defaults` (if it exists at root)

Keep at root: `rust-toolchain.toml`, `CLAUDE.md`, `README.md`, `.gitignore`

- [ ] **Step 9: Verify build compiles**

```bash
. ~/export-esp.sh
cd firmware && cargo build 2>&1 | head -50
```

First build will download ESP-IDF v5.4.x and compile — this takes several minutes. Success = no errors. The binary won't do anything useful yet.

**If `v5.4.1` tag doesn't exist**, update `ESP_IDF_VERSION` in `firmware/.cargo/config.toml` to the correct tag from Step 6 and retry.

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "refactor: restructure as Cargo workspace targeting ESP32-P4

Move from single-crate ESP32-S3 hello-world to a workspace with:
- bsp-waveshare-p4: board support package (placeholder)
- firmware: application binary

Retarget from xtensa-esp32s3-espidf to riscv32imafc-esp-espidf.
ESP-IDF bumped to v5.4.x for MIPI DSI support."
```

---

## Task 2: BSP Pin Constants and Display Timing

**Files:**
- Modify: `bsp-waveshare-p4/src/pins.rs`
- Create: `bsp-waveshare-p4/src/display.rs`
- Modify: `bsp-waveshare-p4/src/lib.rs`

- [ ] **Step 1: Fill in `bsp-waveshare-p4/src/pins.rs`**

All values from Waveshare BSP header `esp32_p4_wifi6_touch_lcd_7b.h`:

```rust
//! Pin assignments and hardware constants for Waveshare ESP32-P4-WIFI6-Touch-LCD-7B.
//! Source: https://github.com/waveshareteam/Waveshare-ESP32-components

// I2C bus (shared: touch GT911, audio ES8311/ES7210)
pub const I2C_SDA: i32 = 7;
pub const I2C_SCL: i32 = 8;

// Display
pub const LCD_RESET: i32 = 33;
pub const LCD_BACKLIGHT: i32 = 32;

// MIPI DSI
pub const DSI_LANE_COUNT: u8 = 2;
pub const DSI_LANE_BITRATE_MBPS: u32 = 1000;

// Display timing (EK79007, 1024x600 @ 60Hz)
pub const LCD_H_RES: u32 = 1024;
pub const LCD_V_RES: u32 = 600;
pub const LCD_HSYNC_BACK_PORCH: u32 = 160;
pub const LCD_HSYNC_FRONT_PORCH: u32 = 160;
pub const LCD_HSYNC_PULSE_WIDTH: u32 = 1;  // from EK79007 macro
pub const LCD_VSYNC_BACK_PORCH: u32 = 23;
pub const LCD_VSYNC_FRONT_PORCH: u32 = 12;
pub const LCD_VSYNC_PULSE_WIDTH: u32 = 1;  // from EK79007 macro
pub const LCD_PIXEL_CLOCK_MHZ: u32 = 80;

// MIPI DSI PHY power (on-chip LDO)
pub const DSI_LDO_CHANNEL: i32 = 3;
pub const DSI_LDO_VOLTAGE_MV: i32 = 2500;

// Touch (GT911) — Milestone 2
pub const TOUCH_I2C_ADDR: u8 = 0x5D;
pub const TOUCH_I2C_ADDR_BACKUP: u8 = 0x14;

// Backlight LEDC config
pub const BACKLIGHT_LEDC_FREQ_HZ: u32 = 5000;
pub const BACKLIGHT_LEDC_RESOLUTION_BITS: u32 = 10;
pub const BACKLIGHT_OUTPUT_INVERT: bool = true;
```

- [ ] **Step 2: Create `bsp-waveshare-p4/src/display.rs` with type stubs**

```rust
//! MIPI DSI display initialization for EK79007 panel.

use crate::pins;

/// Errors during display initialization.
#[derive(Debug)]
pub enum DisplayError {
    LdoInit(esp_idf_svc::sys::EspError),
    DsiBusInit(esp_idf_svc::sys::EspError),
    PanelInit(esp_idf_svc::sys::EspError),
    BacklightInit(esp_idf_svc::sys::EspError),
}

impl std::fmt::Display for DisplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LdoInit(e) => write!(f, "DSI LDO init failed: {e}"),
            Self::DsiBusInit(e) => write!(f, "DSI bus init failed: {e}"),
            Self::PanelInit(e) => write!(f, "LCD panel init failed: {e}"),
            Self::BacklightInit(e) => write!(f, "Backlight init failed: {e}"),
        }
    }
}

impl std::error::Error for DisplayError {}

/// Handle to an initialized display with framebuffer access.
pub struct Display {
    // Fields added during Task 3 when we implement init()
}

impl Display {
    /// Initialize the MIPI DSI display and backlight.
    /// Returns a Display with framebuffer access for rendering.
    pub fn init() -> Result<Self, DisplayError> {
        log::info!(
            "Display init: EK79007 {}x{} via {}-lane MIPI DSI",
            pins::LCD_H_RES, pins::LCD_V_RES, pins::DSI_LANE_COUNT,
        );
        todo!("MIPI DSI init — implemented in Task 3")
    }
}
```

- [ ] **Step 3: Update `bsp-waveshare-p4/src/lib.rs`**

```rust
pub mod display;
pub mod pins;

pub use display::Display;

/// Board peripherals.
pub struct Board {
    pub display: Display,
}

impl Board {
    pub fn init() -> Result<Self, display::DisplayError> {
        let display = Display::init()?;
        Ok(Self { display })
    }
}
```

- [ ] **Step 4: Update `firmware/src/main.rs` error handling**

```rust
use bsp_waveshare_p4::Board;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Display firmware starting on ESP32-P4...");

    let _board = match Board::init() {
        Ok(board) => {
            log::info!("Board initialized successfully");
            board
        }
        Err(e) => {
            log::error!("Board init failed: {e}");
            panic!("Cannot continue without board: {e}");
        }
    };

    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
```

**Note:** The `Cargo.lock` at the repo root will be regenerated by the first `cargo build` with significant churn — this is expected when switching workspace layout and target. Commit the new lock file as part of this task.
```

- [ ] **Step 5: Verify build still compiles**

```bash
cd firmware && cargo build 2>&1 | tail -5
```

Expected: compiles (the `todo!()` in `Display::init()` compiles fine, it panics at runtime).

- [ ] **Step 6: Commit**

```bash
git add bsp-waveshare-p4/src/ firmware/src/main.rs
git commit -m "feat(bsp): add pin constants, display timing, and Display type stubs

EK79007 panel at 1024x600, 2-lane MIPI DSI, 1000 Mbps.
Pin map sourced from Waveshare BSP."
```

---

## Task 3: MIPI DSI Display Init via ESP-IDF

This is the core hardware bringup task. We call ESP-IDF's LCD panel API via `esp-idf-sys` FFI to initialize the MIPI DSI display. The Waveshare C BSP at `esp32_p4_wifi6_touch_lcd_7b.c` is the reference implementation.

**Files:**
- Modify: `bsp-waveshare-p4/src/display.rs`
- Modify: `bsp-waveshare-p4/Cargo.toml`

- [ ] **Step 1: Add ESP-IDF component dependencies**

The EK79007 driver is an ESP-IDF component, not a Rust crate. We need `esp-idf-sys` to pull it in via `idf_component.yml` or `sdkconfig`.

Check if `esp-idf-svc` re-exports the raw LCD panel types. If not, use `esp-idf-svc::sys` (which is `esp-idf-sys`) directly.

Add to `bsp-waveshare-p4/Cargo.toml` if needed:
```toml
[dependencies]
log = "0.4"
esp-idf-svc = "0.52"
anyhow = "1"
```

- [ ] **Step 2: Research the ESP-IDF MIPI DSI init sequence**

The C init sequence from Waveshare BSP (reference, translate to Rust FFI):

```c
// 1. Enable LDO for DSI PHY power
esp_ldo_channel_config_t ldo_cfg = { .chan_id = 3, .voltage_mv = 2500 };
esp_ldo_acquire_channel(&ldo_cfg, &ldo_chan);

// 2. Create MIPI DSI bus
esp_lcd_dsi_bus_config_t bus_cfg = {
    .bus_id = 0,
    .num_data_lanes = 2,
    .phy_clk_src = MIPI_DSI_PHY_CLK_SRC_DEFAULT,
    .lane_bit_rate_mbps = 1000,
};
esp_lcd_new_dsi_bus(&bus_cfg, &dsi_bus);

// 3. Create DPI panel (display pixel interface on DSI)
esp_lcd_dpi_panel_config_t dpi_cfg = {
    .dpi_clk_src = MIPI_DSI_DPI_CLK_SRC_DEFAULT,
    .dpi_clock_freq_mhz = 80,
    .virtual_channel = 0,
    .pixel_format = LCD_COLOR_PIXEL_FORMAT_RGB565,
    .video_timing = { /* h_res, v_res, hsync, vsync, porch values */ },
};
esp_lcd_new_panel_dpi(dsi_bus, &dpi_cfg, &panel);

// 4. Init panel (sends DCS commands, enables display)
esp_lcd_panel_init(panel);

// 5. Enable backlight via LEDC PWM
ledc_timer_config_t timer = { .freq_hz = 5000, .duty_resolution = 10 };
ledc_channel_config_t channel = { .gpio_num = 32, .duty = 1023 };
```

- [ ] **Step 3: Implement `Display::init()` in Rust**

Translate the C sequence to Rust using `esp_idf_svc::sys` FFI calls. Each ESP-IDF function call is `unsafe`. Wrap each in a helper that converts the `esp_err_t` return to `Result`.

This is the most hardware-dependent code in the project. Reference the Waveshare C BSP line by line. Key considerations:
- Use `esp_idf_svc::sys::*` for all ESP-IDF type definitions
- The framebuffer is allocated in PSRAM by ESP-IDF (configured via sdkconfig `CONFIG_SPIRAM=y`)
- The pixel format should be RGB565 for Slint software renderer compatibility
- Store the `esp_lcd_panel_handle_t` in the `Display` struct for later rendering

```rust
pub struct Display {
    panel: esp_idf_svc::sys::esp_lcd_panel_handle_t,
    // Framebuffer pointer for Slint to render into (if DPI panel exposes it)
}
```

- [ ] **Step 4: Implement backlight control**

```rust
impl Display {
    pub fn set_backlight(&self, brightness_pct: u8) -> Result<(), DisplayError> {
        // LEDC duty: 0-1023 (10-bit), inverted output
        // 100% brightness = duty 0 (inverted)
        let duty = ((100 - brightness_pct.min(100)) as u32 * 1023) / 100;
        unsafe {
            // ledc_set_duty + ledc_update_duty
        }
        Ok(())
    }
}
```

- [ ] **Step 5: Build and verify compilation**

```bash
cd firmware && cargo build 2>&1 | tail -20
```

Fix any FFI type mismatches. The ESP-IDF types are generated by bindgen and can be finicky — expect iteration here.

- [ ] **Step 6: Flash to hardware and verify display powers on**

```bash
cd firmware && cargo run
```

At this point the display should power on (backlight visible). It may show garbage or a solid color — that's fine. The panel is initialized.

Check serial output for:
```
Display init: EK79007 1024x600 via 2-lane MIPI DSI
Board initialized successfully
```

If the display doesn't power on, check:
1. Is LDO channel 3 enabled? (DSI PHY needs power)
2. Is GPIO33 toggling correctly for panel reset?
3. Is the backlight LEDC channel outputting?

- [ ] **Step 7: Commit**

```bash
git add bsp-waveshare-p4/
git commit -m "feat(bsp): implement MIPI DSI display init for EK79007

Initialize DSI bus, DPI panel, and LEDC backlight.
Translated from Waveshare C BSP to Rust FFI."
```

---

## Task 4: Slint Platform Backend (or Framebuffer Fallback)

**Depends on:** Task 0 (Slint validation) and Task 3 (display init).

If Task 0 resulted in **go**: implement the Slint Platform adapter.
If Task 0 resulted in **no-go**: skip to Task 4b (framebuffer fallback).

### Task 4a: Slint Platform Backend

**Files:**
- Create: `firmware/src/slint_backend.rs`
- Create: `firmware/ui/main.slint`
- Modify: `firmware/src/main.rs`
- Modify: `firmware/Cargo.toml`

- [ ] **Step 1: Add Slint dependency to `firmware/Cargo.toml`**

```toml
[dependencies]
# ... existing deps ...
slint = { version = "1.x", default-features = false, features = [
    "compat-1-2",
    "renderer-software",
] }
```

Adjust version and features based on Task 0 findings. The `renderer-software` feature enables the MCU software renderer. `default-features = false` avoids pulling in desktop backends.

- [ ] **Step 2: Create `firmware/src/slint_backend.rs`**

Implement `slint::platform::Platform` trait. The key method is providing a window adapter that renders into the MIPI DSI framebuffer.

```rust
//! Slint platform backend for ESP-IDF with MIPI DSI display.

use slint::platform::software_renderer::{MinimalSoftwareWindow, Rgb565Pixel};
use slint::platform::{Platform, WindowAdapter};
use std::rc::Rc;

use bsp_waveshare_p4::pins;

pub struct EspDisplayPlatform {
    window: Rc<MinimalSoftwareWindow>,
}

impl EspDisplayPlatform {
    /// Create the platform and return a clone of the window handle.
    /// The window handle is needed for the render loop after `set_platform`
    /// takes ownership of the Platform.
    pub fn new() -> (Self, Rc<MinimalSoftwareWindow>) {
        let window = MinimalSoftwareWindow::new(
            slint::platform::software_renderer::RepaintBufferType::ReusedBuffer,
        );
        window.set_size(slint::PhysicalSize::new(
            pins::LCD_H_RES,
            pins::LCD_V_RES,
        ));
        let window_clone = window.clone();
        (Self { window }, window_clone)
    }
}

/// Render loop — call after `set_platform` and `MainWindow::new()`.
/// Renders Slint frames into the display framebuffer. Never returns.
pub fn run_event_loop(
    window: &Rc<MinimalSoftwareWindow>,
    display: &bsp_waveshare_p4::Display,
) -> ! {
    loop {
        slint::platform::update_timers_and_animations();

        // Render dirty regions line by line
        window.draw_if_needed(|renderer| {
            renderer.render_by_line(|line, row, _info| {
                // Copy rendered line to display framebuffer
                // This requires Display to expose a method like:
                //   display.draw_pixels(row, &line_buffer);
                // Implementation depends on how esp_lcd_panel_draw_bitmap works
            });
        });

        // Sleep until next frame (~16ms for 60fps)
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

impl Platform for EspDisplayPlatform {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> core::time::Duration {
        // Use ESP-IDF's monotonic clock
        let ticks = unsafe { esp_idf_svc::sys::esp_timer_get_time() };
        core::time::Duration::from_micros(ticks as u64)
    }
}
```

**Note:** The exact rendering integration (how Slint pixels get to the MIPI DSI framebuffer) depends on whether `esp_lcd_panel_draw_bitmap` works or whether we need direct framebuffer access. Add a `draw_pixels` method to `Display` in the BSP crate to abstract this.

- [ ] **Step 3: Add `draw_pixels` to BSP `Display`**

In `bsp-waveshare-p4/src/display.rs`:

```rust
impl Display {
    /// Draw a row of RGB565 pixels at the given y coordinate.
    pub fn draw_pixels(&self, y: u32, pixels: &[u16]) -> Result<(), DisplayError> {
        unsafe {
            // Call esp_lcd_panel_draw_bitmap(self.panel, 0, y, LCD_H_RES, y+1, pixels.as_ptr())
        }
        Ok(())
    }
}
```

- [ ] **Step 4: Create `firmware/ui/main.slint`**

```slint
export component MainWindow inherits Window {
    background: #1a1a2e;

    VerticalLayout {
        alignment: center;

        Text {
            text: "Hello from ESP32-P4!";
            color: #e94560;
            font-size: 48px;
            horizontal-alignment: center;
        }

        Text {
            text: "Slint + Rust + MIPI DSI";
            color: #0f3460;
            font-size: 24px;
            horizontal-alignment: center;
        }
    }
}
```

- [ ] **Step 5: Wire up `firmware/src/main.rs`**

```rust
mod slint_backend;

slint::include_modules!();

use bsp_waveshare_p4::Board;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Display firmware starting on ESP32-P4...");

    let board = Board::init().expect("board init failed");
    log::info!("Board initialized, starting Slint UI...");

    // Set up Slint platform and get back the window for the render loop.
    // set_platform() takes ownership of the Platform, so we extract what
    // we need (the MinimalSoftwareWindow) before handing it over.
    let (platform, window) = slint_backend::EspDisplayPlatform::new();
    slint::platform::set_platform(Box::new(platform)).expect("set platform failed");

    // Create and show the UI
    let _ui = MainWindow::new().expect("window creation failed");

    // This blocks forever, rendering frames into the display
    slint_backend::run_event_loop(&window, &board.display);
}
```

- [ ] **Step 6: Add Slint build integration to `firmware/Cargo.toml`**

```toml
[build-dependencies]
embuild = "0.33"
slint-build = "1.x"  # MUST match the slint runtime crate version exactly (released in lockstep)
```

Update `firmware/build.rs`:
```rust
fn main() {
    embuild::espidf::sysenv::output();
    slint_build::compile("ui/main.slint").unwrap();
}
```

- [ ] **Step 7: Build and iterate**

```bash
cd firmware && cargo build 2>&1 | tail -30
```

Expect compilation errors on first attempt — the Slint platform API and ESP-IDF FFI types need to align. Iterate until it compiles.

- [ ] **Step 8: Flash and verify pixels**

```bash
cd firmware && cargo run
```

Expected: the 7" display shows dark blue background (#1a1a2e) with red text "Hello from ESP32-P4!" centered on screen.

If text renders but looks garbled: check pixel format (RGB565 vs RGB888 mismatch).
If nothing renders but backlight is on: check that `draw_pixels` is correctly calling `esp_lcd_panel_draw_bitmap`.
If display is completely dark: check backlight GPIO and LEDC config.

- [ ] **Step 9: Commit**

```bash
git add firmware/
git commit -m "feat: Slint hello world on MIPI DSI display

Implement Slint Platform backend for ESP-IDF.
Renders 'Hello from ESP32-P4!' at 1024x600 via software renderer."
```

### Task 4b: Framebuffer Fallback (only if Slint is not viable)

**Files:**
- Modify: `firmware/src/main.rs`

- [ ] **Step 1: Fill screen with solid color via `esp_lcd_panel_draw_bitmap`**

```rust
fn main() {
    // ... board init ...

    // Fill framebuffer with a solid blue color (RGB565)
    let blue_pixel: u16 = 0x001F; // RGB565 blue
    let row: Vec<u16> = vec![blue_pixel; 1024];

    for y in 0..600 {
        board.display.draw_pixels(y, &row).expect("draw failed");
    }

    log::info!("Solid blue screen rendered!");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
```

- [ ] **Step 2: Flash and verify pixels**

```bash
cd firmware && cargo run
```

Expected: solid blue screen. This confirms the display pipeline works end to end.

- [ ] **Step 3: Commit**

```bash
git add firmware/src/main.rs
git commit -m "feat: solid color test on MIPI DSI display

Framebuffer fallback — fills screen with blue via esp_lcd_panel_draw_bitmap.
Slint integration deferred to a later milestone."
```

---

## Task 5: Update Documentation

**Files:**
- Modify: `CLAUDE.md`
- Modify: `README.md`

- [ ] **Step 1: Rewrite `CLAUDE.md` for P4**

Update all sections:
- Repository Purpose: ESP32-P4 (Waveshare board), Slint UI
- Toolchain: RISC-V, `riscv32imafc-esp-espidf`
- Building: `cd firmware && cargo build`
- Flashing: `cd firmware && cargo run` (native USB, no kernel extension)
- Hardware section: EK79007 display, GT911 touch, MIPI DSI pin map from `pins.rs`
- Project Goals: updated milestones
- Key Dependencies: add Slint

Remove all CrowPanel / ESP32-S3 references.

- [ ] **Step 2: Rewrite `README.md` for P4**

Update:
- Hardware target: Waveshare ESP32-P4-WIFI6-Touch-LCD-7B
- Dev setup: same (espup, ldproxy, espflash) but note RISC-V target
- Build: `cd firmware && cargo build`
- Flash: `cd firmware && cargo run`
- Project structure: workspace layout
- Pin reference: from `pins.rs`

Remove CrowPanel / S3 references and Pi flashing workaround.

- [ ] **Step 3: Commit**

```bash
git add CLAUDE.md README.md
git commit -m "docs: update CLAUDE.md and README.md for ESP32-P4 migration

Retarget all documentation from CrowPanel S3 to Waveshare P4.
Update build commands for workspace layout."
```

---

## Task 6: Clean Up and Open PR

- [ ] **Step 1: Remove stale build artifacts**

```bash
rm -rf target/ .embuild/
```

The old S3 build cache is invalid for P4. Fresh builds will recreate these.

- [ ] **Step 2: Update `.gitignore` if needed**

Verify `.gitignore` includes (note `**` prefix — build artifacts now land under `firmware/`):
```
**/target/
**/.embuild/
```

- [ ] **Step 3: Push branch and open PR**

```bash
git push -u origin initial-setup
gh pr create --title "Retarget to ESP32-P4 with Slint UI" --body "$(cat <<'EOF'
## Summary
- Restructure as Cargo workspace: `bsp-waveshare-p4` + `firmware`
- Retarget from ESP32-S3 (Xtensa) to ESP32-P4 (RISC-V)
- Initialize MIPI DSI display (EK79007, 1024x600)
- Render hello-world via Slint software renderer (or solid color fallback)

## Hardware
Waveshare ESP32-P4-WIFI6-Touch-LCD-7B

## Test plan
- [ ] `cargo build` succeeds on clean checkout
- [ ] `cargo run` flashes to hardware and displays pixels
- [ ] Serial monitor shows successful board init

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 4: Update spec with findings**

Update the spec at `notes/projects/display-firmware/2026-03-17-p4-migration-design.md`:
- Resolve open questions answered during implementation
- Record Slint go/no-go decision and rationale
- Note any deviations from the plan
