#!/usr/bin/env bash
# Flash wrapper for the Waveshare ESP32-P4 board.
#
# espflash's bundled bootloader targets v3.0+ silicon and crashes at
# do_global_ctors on this board's pre-v3.0 chips. We build a merged
# image with the ESP-IDF-built bootloader instead, then flash via
# esptool. See the display-firmware skill for background.
#
# Override the serial port via ESPFLASH_PORT; otherwise auto-detects.
# Set FLASH_NO_MONITOR=1 to skip the post-flash serial monitor.

set -euo pipefail

ELF="${1:?usage: flash.sh <elf-path>}"

TARGET_DIR="$(dirname "$ELF")"
BOOTLOADER=$(find "$TARGET_DIR/build" -type f -name bootloader.bin -path '*/esp-idf-sys-*/out/build/bootloader/*' 2>/dev/null | head -1)
if [[ -z "$BOOTLOADER" ]]; then
  echo "error: ESP-IDF bootloader not found under $TARGET_DIR/build" >&2
  echo "       expected build/esp-idf-sys-*/out/build/bootloader/bootloader.bin" >&2
  exit 1
fi

PORT="${ESPFLASH_PORT:-}"
if [[ -z "$PORT" ]]; then
  for candidate in /dev/cu.usbmodem101 /dev/cu.usbmodem1101; do
    if [[ -e "$candidate" ]]; then
      PORT="$candidate"
      break
    fi
  done
  if [[ -z "$PORT" ]]; then
    PORT=$(ls /dev/cu.usbmodem* 2>/dev/null | head -1 || true)
  fi
fi
if [[ -z "$PORT" || ! -e "$PORT" ]]; then
  echo "error: no ESP32 serial port found; set ESPFLASH_PORT or connect the board" >&2
  exit 1
fi

MERGED=$(mktemp)
trap 'rm -f "$MERGED"' EXIT

echo "==> Building merged image (bootloader + parts + app)"
espflash save-image \
  --chip esp32p4 \
  --flash-size 32mb \
  --merge \
  --bootloader "$BOOTLOADER" \
  "$ELF" "$MERGED"

echo "==> Flashing $PORT"
esptool --port "$PORT" --chip esp32p4 \
  --before default-reset --after hard-reset \
  write-flash 0x0 "$MERGED"

rm -f "$MERGED"
trap - EXIT

if [[ "${FLASH_NO_MONITOR:-}" == "1" ]]; then
  exit 0
fi

echo "==> Monitoring $PORT (Ctrl+C to exit)"
for _ in 1 2 3 4 5; do
  [[ -e "$PORT" ]] && break
  sleep 1
done
stty -f "$PORT" 115200 raw -echo 2>/dev/null || true
exec cat "$PORT"
