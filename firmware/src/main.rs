mod slint_backend;

slint::include_modules!();

use bsp_waveshare_p4::Board;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Display firmware starting on ESP32-P4...");

    let board = Board::init().expect("board init failed");
    log::info!("Board initialized — display online");

    // Set up Slint platform.
    // set_platform() takes ownership, so we extract the window handle first.
    let (platform, window) = slint_backend::EspDisplayPlatform::new();
    slint::platform::set_platform(Box::new(platform)).expect("set_platform failed");

    // Create and show the UI window
    let ui = MainWindow::new().expect("MainWindow creation failed");
    ui.show().expect("show failed");

    // Render loop — never returns
    slint_backend::run_event_loop(&window, &board.display);
}
