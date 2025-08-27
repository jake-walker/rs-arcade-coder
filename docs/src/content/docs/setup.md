---
title: Rust Setup
---

1. Ensure you have Rust installed, ideally using rustup
1. Install espup and install the necessary toolchains
   ```bash
   cargo install espup
   espup install
   ```

When developing, you will need to run `export ~/export-esp.sh` (or the equivalent for your OS) to apply the required environment variables. Without this you'll get an error like `` linker `xtensa-esp32-elf-gcc` not found ``.

_More info on setting up and developing Rust for ESP32 is [available here](https://docs.esp-rs.org/book/introduction.html)._

- Flash the example project by changing directory to `scoreboard` and running:
  ```bash
  cargo run --release`
  ```
