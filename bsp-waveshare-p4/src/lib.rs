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
