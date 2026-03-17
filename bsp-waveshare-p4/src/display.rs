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

/// Handle to an initialized display.
/// Task 3 adds the panel handle and draw_pixels method.
pub struct Display {
    _private: (),
}

impl Display {
    /// Initialize the MIPI DSI display and backlight.
    pub fn init() -> Result<Self, DisplayError> {
        log::info!(
            "Display init: EK79007 {}x{} via {}-lane MIPI DSI @ {} Mbps",
            pins::LCD_H_RES,
            pins::LCD_V_RES,
            pins::DSI_LANE_COUNT,
            pins::DSI_LANE_BITRATE_MBPS,
        );
        todo!("MIPI DSI init — implemented in Task 3")
    }
}
