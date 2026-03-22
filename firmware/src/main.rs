mod slint_backend;

slint::include_modules!();

use bsp_waveshare_p4::Board;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // Wait for USB Serial/JTAG to enumerate on host.
    // 10s is generous — gives time to start `cat` on the serial port.
    std::thread::sleep(std::time::Duration::from_secs(10));

    unsafe {
        esp_idf_svc::sys::printf(b">>> RUST FIRMWARE ALIVE <<<\n\0".as_ptr() as *const _);
    }
    log::info!("Display firmware starting on ESP32-P4...");

    log::info!("About to call Board::init()...");
    match Board::init() {
        Ok(board) => {
            log::info!("!!! Board init SUCCESS !!!");

            log::info!("Testing green fill (RGB888)...");
            // Try bright green — unambiguous regardless of RGB/BGR order
            let mut row: Vec<u8> = vec![0u8; 1024 * 3];
            for i in (0..row.len()).step_by(3) {
                row[i] = 0x00;
                row[i + 1] = 0xFF; // Green — same position in both RGB and BGR
                row[i + 2] = 0x00;
            }
            for y in 0..600u32 {
                if let Err(e) = board.display.draw_pixels_raw(y, &row) {
                    log::error!("draw_pixels row {y}: {e}");
                    break;
                }
                if y % 100 == 0 {
                    log::info!("  fill row {y}/600");
                }
            }
            log::info!("Red fill done — check display!");
        }
        Err(e) => {
            log::error!("!!! Board init FAILED: {e} !!!");
        }
    }

    log::info!("Entering heartbeat loop");
    loop {
        log::info!("heartbeat");
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
}
