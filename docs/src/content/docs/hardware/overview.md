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
