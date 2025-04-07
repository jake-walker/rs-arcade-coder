---
title: Buttons
---

## Rows

:::note
I tried using these pins with an internal pull up, pull down and no pull. They seems to be most reliable with an internal pull up, however they also worked fine without.
:::

The same rows as connected together for the display also share inputs for the buttons.

| ESP    | Rows   |
| ------ | ------ |
| GPIO39 | 1 & 7  |
| GPIO36 | 2 & 8  |
| GPIO35 | 3 & 9  |
| GPIO34 | 4 & 10 |
| GPIO33 | 5 & 11 |
| GPIO32 | 6 & 12 |

## Detecting Button Presses

:::note
Here you can also do the inverse, by having essentially a fully white display buffer, and removing the red colour from each pixel in turn. This results in a high value for a pressed button, and low without. However, I found this resulted in more noticable artifacts on the display.
:::

:::note
Ensure the latch and output enable are **low**, otherwise it doesn't seem to work.
:::

1. Detect button presses by checking for **high** values on the input pins. This indicates whether one or more buttons are being pressed in that row group.
1. Once you have a row group, cycle through each pixel in the row. Use the same logic & data format as the display to set this pixel to red. However, instead of drawing to the display as normal, instead send the data after setting **A0, A1 & A2 low**.
   _For reference, send `[0xFF] * 9` and set the relevant red component to 0 as you scan._
1. Read the same row group input pin again. If the current pixel is pressed, the value should now be **low**.
1. Continue 2 and 3 until a button has been found.
1. _You may wish to redraw to the display as normal after this to reduce artifacts shown._
