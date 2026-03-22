;;; Directory Local Variables
;;; Configures rust-analyzer to check against the Xtensa ESP32-S3 target
;;; instead of the host (aarch64-apple-darwin), fixing cfg mismatches.
((rust-mode . ((lsp-rust-analyzer-cargo-target . "xtensa-esp32s3-espidf"))))
