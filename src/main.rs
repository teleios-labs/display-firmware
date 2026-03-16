fn main() {
    // Link ESP-IDF runtime patches
    esp_idf_svc::sys::link_patches();

    // Initialize ESP logging → Rust log crate
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello from Rust on ESP32-S3!");
    log::info!("Display firmware starting...");

    loop {
        log::info!("Still alive!");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
