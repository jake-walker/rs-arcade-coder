---
title: Programming Connection
---

The board does not seem to have any data connections on the USB port so it must be programmed via the UART connections.

The board also does not contain the circuitry to automatically go into programming mode. This can be done manually by shorting the GPIO0 pin (located at the bottom of the left edge) to ground. This only needs to be done for a short time while the board is turned on.

Looking at the back of the board, there are 6 pads at the bottom-left of the board. The pins seem to be unconnected other than TX, RX and ground. TX and RX are the labelled pads - pad 2 and 3 respectively. The final pad is ground.

Hook up a USB to UART adapter to the pins for programming.

## Programming Reset Hack

To aid with programming, you can create jumpers from the GPIO0 and EN pads (located at the bottom of the left edge) to any of the unused pads.

![](../../../assets/serial.webp)

If using PlatformIO, you can set the option `upload_resetmethod = ck`. On your UART adapter, connect **RTS** to **EN** on the board and **DTR** to **GPIO0**. When uploading, PlatformIO will now automatically put the board into programming mode, and reset it afterwards.

_You may need to add the options `monitor_dtr = 0` and `monitor_rts = 0` so that you can use the serial monitor without it putting the board into programming mode?_
