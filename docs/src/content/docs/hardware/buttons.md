---
title: Buttons
---

## Rows

:::note
I tried using these pins with an internal pull up, pull down and no pull. They seems to be most reliable with an internal pull up, however they also worked without.
:::

The buttons use the same row layout as the display.

| ESP    | Rows   |
| ------ | ------ |
| GPIO39 | 1 & 7  |
| GPIO36 | 2 & 8  |
| GPIO35 | 3 & 9  |
| GPIO34 | 4 & 10 |
| GPIO33 | 5 & 11 |
| GPIO32 | 6 & 12 |

## Implementation Notes

This uses similar logic as the display where a test pattern is written out on one of the multiplexer channels. This seems to be applied to every row in the matrix.

For example, if we set the first red "pixel" to low, the input pins for the buttons would change depending on whether the first button in that row is pressed or not. To repeat the process, you scan along the entire column, checking the row inputs for each.

1. Create a buffer for `0xffffffff` for the "testing buffer".
1. Set the multiplexer to the input channel where A0, A1 and A2 are all high.
1. To test the first column, turn off the first "red" pixel in the testing buffer. More info on the format is on the display page.
1. Send the test buffer to the display as described on the display page. _For clarity, there is no need for delays here (other than the latch)._
1. Check the rows that are pressed by detecting inputs that are low.
1. Reset the buffer back by turning on the "red" pixel, then repeat for all other columns of the matrix.

You may need to write blank data after sending the test patterns to avoid red artifacts on the display. I noticed very dim red LEDs for the final 6 rows of the matrix on the last column when this was happening.
