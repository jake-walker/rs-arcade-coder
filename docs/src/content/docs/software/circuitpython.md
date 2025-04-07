---
title: CircuitPython
---

:::note
The CircuitPython build used here is generic for the same chip as used in the Arcade Coder. There won't be any nice features or libraries that you may get with other boards.

The ESP32 does not have support for the CIRCUITPY drive, so you will need to program using the REPL with a UART adapter, or with the web-based code editor.
:::

1. Download the BIN file for the [ESP32-DevKitC-V4-WROOM-32E](https://circuitpython.org/board/espressif_esp32_devkitc_v4_wroom_32e/) CircuitPython board.
1. Connect a UART to USB adapter to your board ([see here](./connection.md)). You will need to put the board into programming mode before continuing by shorting GPIO0 to ground when turning on the board for a short while.
1. Find your UART adapter's serial port this will be `COM...` on Windows and `/dev/tty.usb...` on Linux/macOS.
1. Install [esptool](https://github.com/espressif/esptool)
1. Erase the board's flash using the following command:
   ```bash
   esptool.py -p [PORT] erase_flash
   ```
1. Flash the CircuitPython firmware:
   ```bash
   esptool.py -p [PORT] write_flash -z 0x0 adafruit-circuitpython-espressif_esp32_devkitc_v4_wroom_32e-en_GB-9.2.6.bin
   ```

[This guide](https://learn.adafruit.com/welcome-to-circuitpython/overview) is a good overview to getting started.

Use [this guide](https://learn.adafruit.com/getting-started-with-web-workflow-using-the-code-editor/overview) for setting up the web-based code editor, allowing wireless programming over Wi-Fi.
