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
    let board = Board::init().expect("board init failed");
    log::info!("Board initialized — display online");

    // Render "Hello, World!" using simple bitmap font
    // Dark background, white text — drawn directly via draw_pixels_raw
    log::info!("Rendering Hello World...");
    render_hello_world(&board.display);
    log::info!("Hello World rendered!");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

/// Render "Hello, World!" centered on a dark blue background.
/// Uses a simple hardcoded bitmap approach — no font library needed.
fn render_hello_world(display: &bsp_waveshare_p4::Display) {
    let width = 1024usize;
    let height = 600usize;

    // Background color: dark blue (#1a1a2e) in RGB888
    let bg = [0x1a_u8, 0x1a, 0x2e];
    // Text color: coral red (#e94560) in RGB888
    let fg = [0xe9_u8, 0x45, 0x60];

    // Simple 8x16 bitmap font for "Hello, World!"
    // Each character is 8 pixels wide, 16 pixels tall
    // Represented as 16 rows of 8 bits each
    let chars: &[(&str, [u16; 16])] = &[
        ("H", [0x42, 0x42, 0x42, 0x42, 0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        ("e", [0x00, 0x00, 0x00, 0x00, 0x3C, 0x42, 0x7E, 0x40, 0x42, 0x3C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        ("l", [0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x1C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        ("l2",[0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x1C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        ("o", [0x00, 0x00, 0x00, 0x00, 0x3C, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        (",", [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x08, 0x10, 0x00, 0x00, 0x00, 0x00]),
        (" ", [0x00; 16]),
        ("W", [0x41, 0x41, 0x41, 0x41, 0x49, 0x49, 0x49, 0x55, 0x55, 0x22, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        ("o2",[0x00, 0x00, 0x00, 0x00, 0x3C, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        ("r", [0x00, 0x00, 0x00, 0x00, 0x5C, 0x62, 0x40, 0x40, 0x40, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        ("l3",[0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x1C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        ("d", [0x02, 0x02, 0x02, 0x02, 0x3E, 0x42, 0x42, 0x42, 0x42, 0x3E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        ("!", [0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
    ];

    let num_chars = chars.len();
    let scale = 6; // 6x scale: each character becomes 48x96 pixels
    let char_w = 8 * scale;
    let char_h = 10 * scale; // only use top 10 rows of the 16-row font
    let text_w = num_chars * char_w;
    let text_x = (width - text_w) / 2;
    let text_y = (height - char_h) / 2;

    let mut row_buf = vec![0u8; width * 3];

    for y in 0..height {
        // Fill row with background
        for x in 0..width {
            row_buf[x * 3] = bg[0];
            row_buf[x * 3 + 1] = bg[1];
            row_buf[x * 3 + 2] = bg[2];
        }

        // Check if this row intersects the text
        if y >= text_y && y < text_y + char_h {
            let font_row = (y - text_y) / scale;
            if font_row < 10 {
                for (ci, (_name, bitmap)) in chars.iter().enumerate() {
                    let cx = text_x + ci * char_w;
                    let bits = bitmap[font_row];
                    for bit in 0..8 {
                        if bits & (0x80 >> bit) != 0 {
                            // This pixel is "on" — fill scaled block
                            for sx in 0..scale {
                                let px = cx + bit * scale + sx;
                                if px < width {
                                    row_buf[px * 3] = fg[0];
                                    row_buf[px * 3 + 1] = fg[1];
                                    row_buf[px * 3 + 2] = fg[2];
                                }
                            }
                        }
                    }
                }
            }
        }

        let _ = display.draw_pixels_raw(y as u32, &row_buf);
    }
}
