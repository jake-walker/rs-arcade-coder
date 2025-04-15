## Rust Setup

1. Ensure you have Rust and rustup installed
1. Install espup and install the necessary toolchains
   ```
   cargo install espup
   espup install
   ```

When developing, you will need to run `export ~/export-esp.sh` (or the equivalent for your OS) to apply the required environment variables. Without this you'll get an error like `` linker `xtensa-esp32-elf-gcc` not found ``.

_More info on setting up and developing Rust for ESP32 is [available here](https://docs.esp-rs.org/book/introduction.html)._
