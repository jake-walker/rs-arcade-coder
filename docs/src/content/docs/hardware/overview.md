---
title: Hardware Overview
description: A guide in my new Starlight docs site.
---

Pádraig has a great summary on the hardware [here](https://github.com/padraigfl/twsu-arcade-coder-esp32?tab=readme-ov-file#how-it-works). This is based off their fantastic work and some extra bits I've found while implementing a Rust library.

| Component                          | Description                                                              |
| ---------------------------------- | ------------------------------------------------------------------------ |
| Microcontroller                    | Espressif ESP32-WROOM-32D                                                |
| [Display (LED Matrix)](../display) | 24x24 LED matrix driven by ICN2012 multiplexer and HC595 shift registers |
| [Matrix Buttons](../buttons)       |                                                                          |
| Home Button                        |                                                                          |
| Status LEDs                        | Blue LED on GPIO 22, Red LED on GPIO 23                                  |

## Pin Overview

| #   | Label     | Description                                         |
| --- | --------- | --------------------------------------------------- |
| 1   | GND       | ---                                                 |
| 2   | 3V3       | ---                                                 |
| 3   | EN        | ---                                                 |
| 4   | SENSOR_VP | (GPIO36) Input Rows 2 & 8 ([Buttons](./buttons.md)) |
| 5   | SENSOR_VN | (GPIO39) Input Rows 1 & 7 ([Buttons](./buttons.md)) |
| 6   | IO34      | Input Rows 4 & 10 ([Buttons](./buttons.md))         |
| 7   | IO35      | Input Rows 3 & 9 ([Buttons](./buttons.md))          |
| 8   | IO32      | Input Rows 6 & 12 ([Buttons](./buttons.md))         |
| 9   | IO33      | Input Rows 5 & 11 ([Buttons](./buttons.md))         |
| 10  | IO25      |                                                     |
| 11  | IO26      | I2C (Motion Sensor)                                 |
| 12  | IO27      | I2C (Motion Sensor)                                 |
| 13  | IO14      |                                                     |
| 14  | IO12      |                                                     |
| 15  | GND       | ---                                                 |
| 16  | IO13      |                                                     |
| 17  | SD2       |                                                     |
| 18  | SD3       |                                                     |
| 19  | CMD       |                                                     |
| 20  | CLK       |                                                     |
| 21  | SD0       |                                                     |
| 22  | SD1       |                                                     |
| 23  | IO15      |                                                     |
| 24  | IO2       | Home Button                                         |
| 25  | IO0       |                                                     |
| 26  | IO4       | HC595 Output Enable ([Display](./display.md))       |
| 27  | IO16      | HC595 Latch ([Display](./display.md))               |
| 28  | IO17      | HC595 Clock ([Display](./display.md))               |
| 29  | IO5       | HC595 Data ([Display](./display.md))                |
| 30  | IO18      | ICN2012 A1 ([Display](./display.md))                |
| 31  | IO19      | ICN2012 A0 ([Display](./display.md))                |
| 32  | NC        | ---                                                 |
| 33  | IO21      | ICN2012 A2 ([Display](./display.md))                |
| 34  | RXD0      |                                                     |
| 35  | TXD1      |                                                     |
| 36  | IO22      | Blue LED                                            |
| 37  | IO23      | Red LED                                             |
| 38  | GND       | ---                                                 |
