//! MIPI DSI display initialization for EK79007 panel.

use crate::pins;
use esp_idf_svc::sys::*;

/// Errors during display initialization.
#[derive(Debug)]
pub enum DisplayError {
    LdoInit(EspError),
    DsiBusInit(EspError),
    PanelInit(EspError),
    BacklightInit(EspError),
    DrawError(EspError),
}

impl std::fmt::Display for DisplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LdoInit(e) => write!(f, "DSI LDO init failed: {e}"),
            Self::DsiBusInit(e) => write!(f, "DSI bus init failed: {e}"),
            Self::PanelInit(e) => write!(f, "LCD panel init failed: {e}"),
            Self::BacklightInit(e) => write!(f, "Backlight init failed: {e}"),
            Self::DrawError(e) => write!(f, "Display draw failed: {e}"),
        }
    }
}

impl std::error::Error for DisplayError {}

/// Convert an ESP-IDF error code to a Result.
fn esp_check(err: esp_err_t) -> Result<(), EspError> {
    EspError::check_and_return(err, ())
}

/// Handle to an initialized display.
pub struct Display {
    panel: esp_lcd_panel_handle_t,
    #[allow(dead_code)]
    ldo_handle: esp_ldo_channel_handle_t,
}

// Safety: The ESP-IDF panel and LDO handles are thread-safe (protected by internal locks).
unsafe impl Send for Display {}

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

        // 1. Enable LDO for DSI PHY power
        log::info!("Enabling LDO ch{} at {}mV for DSI PHY", pins::DSI_LDO_CHANNEL, pins::DSI_LDO_VOLTAGE_MV);
        let ldo_cfg = esp_ldo_channel_config_t {
            chan_id: pins::DSI_LDO_CHANNEL,
            voltage_mv: pins::DSI_LDO_VOLTAGE_MV,
            flags: esp_ldo_channel_config_t_ldo_extra_flags::default(),
        };
        let mut ldo_handle: esp_ldo_channel_handle_t = std::ptr::null_mut();
        unsafe {
            esp_check(esp_ldo_acquire_channel(&ldo_cfg, &mut ldo_handle))
                .map_err(DisplayError::LdoInit)?;
        }

        // 2. Create MIPI DSI bus
        log::info!("Creating MIPI DSI bus ({} lanes)", pins::DSI_LANE_COUNT);
        let dsi_bus_cfg = esp_lcd_dsi_bus_config_t {
            bus_id: 0,
            num_data_lanes: pins::DSI_LANE_COUNT,
            phy_clk_src: soc_periph_mipi_dsi_phy_clk_src_t_MIPI_DSI_PHY_CLK_SRC_DEFAULT,
            lane_bit_rate_mbps: pins::DSI_LANE_BITRATE_MBPS,
        };
        let mut dsi_bus: esp_lcd_dsi_bus_handle_t = std::ptr::null_mut();
        unsafe {
            esp_check(esp_lcd_new_dsi_bus(&dsi_bus_cfg, &mut dsi_bus))
                .map_err(DisplayError::DsiBusInit)?;
        }

        // 3. Create DPI panel
        log::info!("Creating DPI panel ({}x{} RGB565 @ {}MHz pclk)",
            pins::LCD_H_RES, pins::LCD_V_RES, pins::LCD_PIXEL_CLOCK_MHZ);
        let video_timing = esp_lcd_video_timing_t {
            h_size: pins::LCD_H_RES,
            v_size: pins::LCD_V_RES,
            hsync_pulse_width: pins::LCD_HSYNC_PULSE_WIDTH,
            hsync_back_porch: pins::LCD_HSYNC_BACK_PORCH,
            hsync_front_porch: pins::LCD_HSYNC_FRONT_PORCH,
            vsync_pulse_width: pins::LCD_VSYNC_PULSE_WIDTH,
            vsync_back_porch: pins::LCD_VSYNC_BACK_PORCH,
            vsync_front_porch: pins::LCD_VSYNC_FRONT_PORCH,
        };

        let mut dpi_cfg = esp_lcd_dpi_panel_config_t {
            virtual_channel: 0,
            dpi_clk_src: soc_periph_mipi_dsi_dpi_clk_src_t_MIPI_DSI_DPI_CLK_SRC_DEFAULT,
            dpi_clock_freq_mhz: pins::LCD_PIXEL_CLOCK_MHZ,
            pixel_format: lcd_color_rgb_pixel_format_t_LCD_COLOR_PIXEL_FORMAT_RGB565,
            in_color_format: lcd_color_format_t_LCD_COLOR_FMT_RGB565,
            out_color_format: lcd_color_format_t_LCD_COLOR_FMT_RGB565,
            video_timing,
            ..Default::default()
        };
        // Use 1 framebuffer (minimum)
        dpi_cfg.num_fbs = 1;

        let mut panel: esp_lcd_panel_handle_t = std::ptr::null_mut();
        unsafe {
            esp_check(esp_lcd_new_panel_dpi(dsi_bus, &dpi_cfg, &mut panel))
                .map_err(DisplayError::PanelInit)?;
        }

        // 4. Init panel and turn on
        log::info!("Initializing and enabling panel");
        unsafe {
            esp_check(esp_lcd_panel_reset(panel)).map_err(DisplayError::PanelInit)?;
            esp_check(esp_lcd_panel_init(panel)).map_err(DisplayError::PanelInit)?;
            esp_check(esp_lcd_panel_disp_on_off(panel, true)).map_err(DisplayError::PanelInit)?;
        }

        // 5. Configure backlight (LEDC PWM)
        log::info!("Configuring backlight PWM on GPIO{} (inverted, {}Hz)",
            pins::LCD_BACKLIGHT, pins::BACKLIGHT_LEDC_FREQ_HZ);
        let timer_cfg = ledc_timer_config_t {
            speed_mode: ledc_mode_t_LEDC_LOW_SPEED_MODE,
            duty_resolution: ledc_timer_bit_t_LEDC_TIMER_10_BIT,
            timer_num: ledc_timer_t_LEDC_TIMER_1,
            freq_hz: pins::BACKLIGHT_LEDC_FREQ_HZ,
            clk_cfg: soc_periph_ledc_clk_src_legacy_t_LEDC_AUTO_CLK,
            ..Default::default()
        };
        unsafe {
            esp_check(ledc_timer_config(&timer_cfg)).map_err(DisplayError::BacklightInit)?;
        }

        // Channel config — inverted: duty 0 = full brightness
        let mut channel_cfg = ledc_channel_config_t {
            gpio_num: pins::LCD_BACKLIGHT,
            speed_mode: ledc_mode_t_LEDC_LOW_SPEED_MODE,
            channel: ledc_channel_t_LEDC_CHANNEL_1,
            intr_type: ledc_intr_type_t_LEDC_INTR_DISABLE,
            timer_sel: ledc_timer_t_LEDC_TIMER_1,
            duty: 0, // inverted: 0 = max brightness
            hpoint: 0,
            ..Default::default()
        };
        // Set output_invert flag for inverted backlight
        if pins::BACKLIGHT_OUTPUT_INVERT {
            channel_cfg.flags.set_output_invert(1);
        }
        unsafe {
            esp_check(ledc_channel_config(&channel_cfg)).map_err(DisplayError::BacklightInit)?;
        }

        log::info!("Display initialized successfully");
        Ok(Display { panel, ldo_handle })
    }

    /// Draw a horizontal line of RGB565 pixels at the given y coordinate.
    /// Called by the Slint render loop to push rendered lines to the display.
    pub fn draw_pixels(&self, y: u32, pixels: &[u16]) -> Result<(), DisplayError> {
        debug_assert_eq!(
            pixels.len(),
            crate::pins::LCD_H_RES as usize,
            "draw_pixels: pixel slice must be exactly LCD_H_RES ({}) pixels wide",
            crate::pins::LCD_H_RES
        );

        let x_end = pixels.len() as i32;
        unsafe {
            esp_check(esp_lcd_panel_draw_bitmap(
                self.panel,
                0,
                y as i32,
                x_end,
                (y + 1) as i32,
                pixels.as_ptr() as *const _,
            ))
            .map_err(DisplayError::DrawError)?;
        }
        Ok(())
    }
}
