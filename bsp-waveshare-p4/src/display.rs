//! MIPI DSI display initialization for EK79007 panel.
//!
//! Uses the official `esp_lcd_ek79007` ESP-IDF component driver, which handles
//! all DCS vendor commands, timing, and panel init internally.

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

fn esp_check(err: esp_err_t) -> Result<(), EspError> {
    EspError::check_and_return(err, ())
}

fn log_step(msg: &str) {
    log::info!("{msg}");
    std::thread::sleep(std::time::Duration::from_millis(50));
}

/// Handle to an initialized display.
pub struct Display {
    panel: esp_lcd_panel_handle_t,
    #[allow(dead_code)]
    ldo_handle: esp_ldo_channel_handle_t,
}

unsafe impl Send for Display {}

impl Display {
    /// Initialize the MIPI DSI display using the official EK79007 driver.
    pub fn init() -> Result<Self, DisplayError> {
        log_step("=== Display init (EK79007 driver) ===");

        // 1. LDO enable (DSI PHY power)
        log_step("Step 1: LDO enable");
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

        // 2. DSI bus
        log_step("Step 2: DSI bus");
        let dsi_bus_cfg = esp_lcd_dsi_bus_config_t {
            bus_id: 0,
            num_data_lanes: pins::DSI_LANE_COUNT,
            phy_clk_src: 0,
            lane_bit_rate_mbps: pins::DSI_LANE_BITRATE_MBPS,
        };
        let mut dsi_bus: esp_lcd_dsi_bus_handle_t = std::ptr::null_mut();
        unsafe {
            esp_check(esp_lcd_new_dsi_bus(&dsi_bus_cfg, &mut dsi_bus))
                .map_err(DisplayError::DsiBusInit)?;
        }

        // 3. DBI command IO (the EK79007 driver uses this to send DCS commands)
        log_step("Step 3: DBI IO");
        let dbi_cfg = esp_lcd_dbi_io_config_t {
            virtual_channel: 0,
            lcd_cmd_bits: 8,
            lcd_param_bits: 8,
        };
        let mut dbi_io: esp_lcd_panel_io_handle_t = std::ptr::null_mut();
        unsafe {
            esp_check(esp_lcd_new_panel_io_dbi(dsi_bus, &dbi_cfg, &mut dbi_io))
                .map_err(DisplayError::PanelInit)?;
        }

        // 4. Create EK79007 panel via the official driver
        // This internally creates the DPI panel AND stores the DCS command sequence.
        log_step("Step 4: EK79007 panel create");

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

        let dpi_cfg = esp_lcd_dpi_panel_config_t {
            virtual_channel: 0,
            dpi_clk_src: 0,
            dpi_clock_freq_mhz: pins::LCD_PIXEL_CLOCK_MHZ,
            pixel_format: lcd_color_rgb_pixel_format_t_LCD_COLOR_PIXEL_FORMAT_RGB888,
            in_color_format: lcd_color_format_t_LCD_COLOR_FMT_RGB888,
            out_color_format: lcd_color_format_t_LCD_COLOR_FMT_RGB888,
            video_timing,
            num_fbs: 1,
            ..Default::default()
        };

        // Vendor config tells the EK79007 driver about the DSI bus and DPI settings.
        // The driver creates the DPI panel internally.
        let mut vendor_config = ek79007_vendor_config_t {
            init_cmds: std::ptr::null(), // use default init commands
            init_cmds_size: 0,
            mipi_config: ek79007_vendor_config_t__bindgen_ty_1 {
                dsi_bus,
                dpi_config: &dpi_cfg,
                lane_num: pins::DSI_LANE_COUNT,
            },
        };

        let panel_dev_config = esp_lcd_panel_dev_config_t {
            reset_gpio_num: pins::LCD_RESET,
            bits_per_pixel: 24, // RGB888
            vendor_config: &mut vendor_config as *mut _ as *mut _,
            ..Default::default()
        };

        let mut panel: esp_lcd_panel_handle_t = std::ptr::null_mut();
        unsafe {
            esp_check(esp_lcd_new_panel_ek79007(dbi_io, &panel_dev_config, &mut panel))
                .map_err(DisplayError::PanelInit)?;
        }

        // 5. Reset + init the panel (driver sends all DCS commands + starts DPI stream)
        log_step("Step 5: Panel reset + init");
        unsafe {
            esp_check(esp_lcd_panel_reset(panel)).map_err(DisplayError::PanelInit)?;
            esp_check(esp_lcd_panel_init(panel)).map_err(DisplayError::PanelInit)?;
        }

        // 6. Backlight
        log_step("Step 6: Backlight");
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
        let channel_cfg = ledc_channel_config_t {
            gpio_num: pins::LCD_BACKLIGHT,
            speed_mode: ledc_mode_t_LEDC_LOW_SPEED_MODE,
            channel: ledc_channel_t_LEDC_CHANNEL_1,
            intr_type: ledc_intr_type_t_LEDC_INTR_DISABLE,
            timer_sel: ledc_timer_t_LEDC_TIMER_1,
            duty: 0, // try 0 — hardware may be active-low (0 = bright, 1023 = off)
            hpoint: 0,
            ..Default::default()
        };
        unsafe {
            esp_check(ledc_channel_config(&channel_cfg)).map_err(DisplayError::BacklightInit)?;
            esp_check(ledc_update_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1))
                .map_err(DisplayError::BacklightInit)?;
        }

        log_step("=== Display init COMPLETE ===");
        Ok(Display { panel, ldo_handle })
    }

    /// Draw a horizontal line of raw pixel bytes at the given y coordinate.
    pub fn draw_pixels_raw(&self, y: u32, data: &[u8]) -> Result<(), DisplayError> {
        unsafe {
            esp_check(esp_lcd_panel_draw_bitmap(
                self.panel, 0, y as i32,
                crate::pins::LCD_H_RES as i32, (y + 1) as i32,
                data.as_ptr() as *const _,
            ))
            .map_err(DisplayError::DrawError)?;
        }
        Ok(())
    }
}
