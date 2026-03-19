mod slint_backend;

slint::include_modules!();

use bsp_waveshare_p4::Board;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // Force output via printf to bypass any log buffering
    unsafe {
        esp_idf_svc::sys::printf(b">>> RUST FIRMWARE ALIVE <<<\n\0".as_ptr() as *const _);
    }

    log::info!("========================================");
    log::info!("Display firmware starting on ESP32-P4...");
    log::info!("========================================");

    log::info!("About to call Board::init()...");
    let board = Board::init().expect("board init failed");
    log::info!("Board initialized — display online");

    log::info!("Testing display with solid color fill...");
    // Fill screen with bright red to test display pipeline
    let red_pixel: u16 = 0xF800; // RGB565 red
    let row: Vec<u16> = vec![red_pixel; 1024];
    for y in 0..600u32 {
        if let Err(e) = board.display.draw_pixels(y, &row) {
            log::error!("draw_pixels failed at row {y}: {e}");
            break;
        }
    }
    log::info!("Solid red fill complete — check display!");

    // Pause so we can see the result before Slint takes over
    std::thread::sleep(std::time::Duration::from_secs(10));

    log::info!("Starting Slint UI...");

    // Set up Slint platform.
    let (platform, window) = slint_backend::EspDisplayPlatform::new();
    slint::platform::set_platform(Box::new(platform)).expect("set_platform failed");

    // Create and show the UI window
    let ui = MainWindow::new().expect("MainWindow creation failed");
    ui.show().expect("show failed");

    // Render loop — never returns
    slint_backend::run_event_loop(&window, &board.display);
}
