//! Pin assignments and hardware constants for Waveshare ESP32-P4-WIFI6-Touch-LCD-7B.
//! Source: https://github.com/waveshareteam/Waveshare-ESP32-components

// I2C bus (shared: touch GT911, audio ES8311/ES7210)
pub const I2C_SDA: i32 = 7;
pub const I2C_SCL: i32 = 8;

// Display
pub const LCD_RESET: i32 = 33;
pub const LCD_BACKLIGHT: i32 = 32;

// MIPI DSI
pub const DSI_LANE_COUNT: u8 = 2;
pub const DSI_LANE_BITRATE_MBPS: u32 = 1000;

// Display timing (EK79007, 1024x600 @ 60Hz)
pub const LCD_H_RES: u32 = 1024;
pub const LCD_V_RES: u32 = 600;
pub const LCD_HSYNC_BACK_PORCH: u32 = 160;
pub const LCD_HSYNC_FRONT_PORCH: u32 = 160;
pub const LCD_HSYNC_PULSE_WIDTH: u32 = 1;
pub const LCD_VSYNC_BACK_PORCH: u32 = 23;
pub const LCD_VSYNC_FRONT_PORCH: u32 = 12;
pub const LCD_VSYNC_PULSE_WIDTH: u32 = 1;
pub const LCD_PIXEL_CLOCK_MHZ: u32 = 80;

// MIPI DSI PHY power (on-chip LDO)
pub const DSI_LDO_CHANNEL: i32 = 3;
pub const DSI_LDO_VOLTAGE_MV: i32 = 2500;

// Touch (GT911) — Milestone 2
pub const TOUCH_I2C_ADDR: u8 = 0x5D;
pub const TOUCH_I2C_ADDR_BACKUP: u8 = 0x14;

// Backlight LEDC config
pub const BACKLIGHT_LEDC_FREQ_HZ: u32 = 5000;
pub const BACKLIGHT_LEDC_RESOLUTION_BITS: u32 = 10;
pub const BACKLIGHT_OUTPUT_INVERT: bool = true;
