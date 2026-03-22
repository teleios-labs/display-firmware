//! Slint platform backend for ESP-IDF with MIPI DSI display.
//!
//! Implements the Slint `Platform` trait to render UI into the EK79007 display
//! via the software renderer. Uses `render_by_line` to avoid a full RGB565
//! framebuffer in DRAM (saves ~1.2 MB for 1024×600).

use std::rc::Rc;

use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType, Rgb565Pixel};
use slint::platform::{Platform, WindowAdapter};

use bsp_waveshare_p4::{pins, Display};

pub struct EspDisplayPlatform {
    window: Rc<MinimalSoftwareWindow>,
}

impl EspDisplayPlatform {
    /// Create the platform and return a clone of the window handle.
    ///
    /// `set_platform()` takes ownership of the `Platform`, so we clone the
    /// `MinimalSoftwareWindow` before handing it over.
    pub fn new() -> (Self, Rc<MinimalSoftwareWindow>) {
        let window = MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
        window.set_size(slint::PhysicalSize::new(pins::LCD_H_RES, pins::LCD_V_RES));
        let window_clone = window.clone();
        (Self { window }, window_clone)
    }
}

impl Platform for EspDisplayPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> core::time::Duration {
        // Use ESP-IDF's monotonic microsecond timer
        let micros = unsafe { esp_idf_svc::sys::esp_timer_get_time() };
        core::time::Duration::from_micros(micros as u64)
    }
}

/// Render loop — call after `slint::platform::set_platform()` and after creating the UI.
/// Renders Slint frames into the display line by line. Never returns.
pub fn run_event_loop(window: &Rc<MinimalSoftwareWindow>, display: &Display) -> ! {
    // Single-line RGB565 buffer — avoids allocating a full framebuffer in DRAM
    let mut line_buffer: Vec<Rgb565Pixel> = vec![Rgb565Pixel::default(); pins::LCD_H_RES as usize];

    loop {
        slint::platform::update_timers_and_animations();

        window.draw_if_needed(|renderer| {
            struct LineBufferProvider<'a> {
                display: &'a Display,
                line_buffer: &'a mut Vec<Rgb565Pixel>,
            }

            impl slint::platform::software_renderer::LineBufferProvider
                for LineBufferProvider<'_>
            {
                type TargetPixel = Rgb565Pixel;

                fn process_line(
                    &mut self,
                    line: usize,
                    range: core::ops::Range<usize>,
                    render_fn: impl FnOnce(&mut [Rgb565Pixel]),
                ) {
                    // Render this line into the buffer
                    render_fn(&mut self.line_buffer[range.clone()]);

                    // Cast &[Rgb565Pixel] to raw bytes for draw_pixels_raw
                    let pixels_bytes: &[u8] = unsafe {
                        std::slice::from_raw_parts(
                            self.line_buffer[range.clone()].as_ptr() as *const u8,
                            range.len() * 2, // 2 bytes per Rgb565Pixel
                        )
                    };

                    // Push to display (ignore draw errors during render — log only)
                    if let Err(e) = self.display.draw_pixels_raw(line as u32, pixels_bytes) {
                        log::error!("draw_pixels row {line}: {e}");
                    }
                }
            }

            renderer.render_by_line(LineBufferProvider {
                display,
                line_buffer: &mut line_buffer,
            });
        });

        // ~60fps frame pacing
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
