# Display Firmware — Waveshare ESP32-P4-WIFI6-Touch-LCD-7B
#
# Targets:
#   make build     Compile firmware. PROFILE=release for release.
#   make flash     Build and flash. No serial monitor.
#   make run       Build, flash, and stream serial output (same as `cargo run`).
#   make monitor   Stream serial output. Ctrl+C to exit.
#   make clean     Remove all build artifacts. Next build redownloads ESP-IDF.
#
# Overrides:
#   make flash PORT=/dev/cu.usbmodemXXX   Pin the serial port.
#   make build PROFILE=release            Build the release profile.
#
# The flash pipeline lives in firmware/scripts/flash.sh — that's the single
# source of truth. It builds a merged image with the ESP-IDF bootloader
# (required for this board's pre-v3.0 P4 silicon; espflash's bundled
# bootloader crashes at do_global_ctors) and writes it via esptool.
#
# ----------------------------------------------------------------------------
# Troubleshooting
# ----------------------------------------------------------------------------
#
# Wrong port / "could not open port" / "no such file":
#   Run `ls /dev/cu.usbmodem*` to see what's available. Going through a USB
#   hub changes the port suffix (macOS derives it from USB topology), so the
#   default usbmodem101 / usbmodem1101 won't match. Pin the port explicitly:
#       make flash PORT=/dev/cu.usbmodemXXX
#
# Device resets mid-flash, brownouts, or display flickers off:
#   Unpowered hubs can't deliver enough current for the P4 + 7" panel — the
#   backlight and PSRAM are the hungry parts. Use a powered hub, or plug the
#   board directly into the Mac.
#
# Illegal-instruction panic at do_global_ctors / "invalid header" at boot:
#   You booted with the wrong bootloader. Always flash via this Makefile or
#   `cargo run` (which calls firmware/scripts/flash.sh). Never run
#   `espflash flash` directly — its bundled bootloader targets v3.0+ silicon
#   and this board is v1.3.
#
# Device won't enter download mode:
#   Hold Boot, tap Reset, release Boot. The P4 comes up in ROM download mode
#   and the next flash will succeed.

PROFILE ?= debug
PORT    ?=
TARGET  := riscv32imafc-esp-espidf
ELF     := target/$(TARGET)/$(PROFILE)/display-firmware

# Forwarded to firmware/scripts/flash.sh. Empty = let the script auto-detect.
export ESPFLASH_PORT := $(PORT)

CARGO_PROFILE_FLAG := $(if $(filter release,$(PROFILE)),--release,)

.PHONY: build flash run monitor clean

build:
	@. ~/export-esp.sh && cd firmware && cargo build $(CARGO_PROFILE_FLAG)

flash: build
	@FLASH_NO_MONITOR=1 ./firmware/scripts/flash.sh $(ELF)

run:
	@. ~/export-esp.sh && cd firmware && cargo run $(CARGO_PROFILE_FLAG)

monitor:
	@PORT_RESOLVED=$${ESPFLASH_PORT:-$$(ls /dev/cu.usbmodem101 /dev/cu.usbmodem1101 2>/dev/null | head -1)}; \
	 if [ -z "$$PORT_RESOLVED" ]; then \
	   echo "error: no serial port found; set PORT=/dev/cu.usbmodemXXX" >&2; exit 1; \
	 fi; \
	 echo "Monitoring $$PORT_RESOLVED — Ctrl+C to stop"; \
	 stty -f "$$PORT_RESOLVED" 115200 raw -echo 2>/dev/null || true; \
	 cat "$$PORT_RESOLVED"

clean:
	@cargo clean
