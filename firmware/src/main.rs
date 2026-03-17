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
