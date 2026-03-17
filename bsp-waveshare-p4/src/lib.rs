pub mod display;
pub mod pins;

pub use display::{Display, DisplayError};

/// Initialized board peripherals.
pub struct Board {
    pub display: Display,
}

impl Board {
    /// Initialize all board hardware.
    pub fn init() -> Result<Self, DisplayError> {
        let display = Display::init()?;
        Ok(Self { display })
    }
}
