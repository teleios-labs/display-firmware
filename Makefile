# Display Firmware — Waveshare ESP32-P4-WIFI6-Touch-LCD-7B
#
# Usage:
#   make build      — compile firmware
#   make flash      — build + flash to board
#   make monitor    — read serial output
#   make run        — build + flash + monitor
#   make clean      — full clean (redownloads ESP-IDF on next build)

SHELL := /bin/zsh
PORT ?= /dev/cu.usbmodem1101
CHIP := esp32p4
TARGET := riscv32imafc-esp-espidf
PROFILE ?= debug
ELF := target/$(TARGET)/$(PROFILE)/display-firmware
MERGED := /tmp/firmware-merged.bin

# Find the ESP-IDF-built bootloader (required for pre-v3.0 chip revision)
BOOTLOADER = $(shell find target -name "bootloader.bin" -path "*/esp-idf-sys-*/out/build/bootloader/*" 2>/dev/null | head -1)

.PHONY: build flash monitor run clean

build:
	@echo "Building firmware..."
	@. ~/export-esp.sh && cd firmware && cargo build $(if $(filter release,$(PROFILE)),--release,)
	@echo "Done. Binary: $(ELF)"

flash: build
	@echo "Generating merged image with custom bootloader..."
	@. ~/export-esp.sh && espflash save-image \
		--chip $(CHIP) --flash-size 32mb --merge \
		--bootloader "$(BOOTLOADER)" \
		$(ELF) $(MERGED)
	@echo "Flashing via $(PORT)..."
	@. ~/export-esp.sh && esptool --port $(PORT) --chip $(CHIP) \
		--before default-reset --after hard-reset \
		write-flash 0x0 $(MERGED)
	@echo "Flash complete."

monitor:
	@echo "Monitoring $(PORT) — Ctrl+C to stop"
	@cat $(PORT)

run: flash
	@echo "Waiting for boot (15s)..."
	@sleep 15
	@echo "Reading serial output..."
	@cat $(PORT)

clean:
	@echo "Cleaning all build artifacts..."
	@cargo clean
	@echo "Clean. Next build will redownload ESP-IDF."
