---
title: Display
---

The display on the Arcade Coder is a 12x12 LED matrix. Each pixel works in 3-bit colour (giving 8 total colours). Each pixel also serves as a [button](./buttons.md).

The matrix is split into groups by row. The multiplexer selects the rows to be drawn to, and the data is sent to the rows via the shift register.

## Pins

### HC595 Shift Register

| ESP    | Shift Register |
| ------ | -------------- |
| GPIO5  | Data In        |
| GPIO17 | Clock          |
| GPIO16 | Latch          |
| GPIO4  | Output Enable  |

For the output enable pin, a high signal enables output and a low signal disables it. Practically, set high when writing data.

### ICN2012 Multiplexer

The rows are selected by setting different combinations on the multiplexer.

| ESP    | Multiplexer |
| ------ | ----------- |
| GPIO19 | A0          |
| GPIO18 | A1          |
| GPIO21 | A2          |

| A0  | A1  | A2  | Rows            |
| --- | --- | --- | --------------- |
| 0   | 0   | 0   | _Unknown/Off_   |
| 0   | 0   | 1   | 4 & 9           |
| 0   | 1   | 0   | 1 & 6           |
| 0   | 1   | 1   | 6 & 12          |
| 1   | 0   | 0   | 5 & 10          |
| 1   | 0   | 1   | 3 & 8           |
| 1   | 1   | 0   | 2 & 7           |
| 1   | 1   | 1   | _Input Reading_ |

## Data Format

The data sent to each of the two rows is 9 bytes (or 72 bits). The bits are inversed, with a 0 representing the **on** state and a 1 representing **off**.

For testing, sending 9 bytes of `0xffffffff` should result in all white LEDs, and `0x00000000` should turn off all the LEDs.

_The table below is ordered with most significant bit first._

<table>
  <thead>
    <tr>
      <th>Byte</th>
      <th>8</th>
      <th>7</th>
      <th>6</th>
      <th>5</th>
      <th>4</th>
      <th>3</th>
      <th>2</th>
      <th>1</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>1</td>
      <td colspan="8"  style="background-color:#d3f9d8;">Green, pixels 5-12, top row</td>
    </tr>
    <tr>
      <td>2</td>
      <td colspan="8" style="background-color:#ffe3e3;">Red, pixels 5-12, top row</td>
    </tr>
    <tr>
      <td>3</td>
      <td colspan="8" style="background-color:#d0ebff;">Blue, pixels 5-12, top row</td>
    </tr>
    <tr>
      <td>4</td>
      <td colspan="4" style="background-color:#d3f9d8; border-right: 1px solid var(--sl-color-gray-5, '#000');">Green, pixels 1-4, bottom row</td>
      <td colspan="4" style="background-color:#d3f9d8;">Green, pixels 1-4, top row</td>
    </tr>
    <tr>
      <td>5</td>
      <td colspan="4" style="background-color:#ffe3e3; border-right: 1px solid var(--sl-color-gray-5, '#000');">Red, pixels 1-4, bottom row</td>
      <td colspan="4" style="background-color:#ffe3e3;">Red, pixels 1-4, top row</td>
    </tr>
    <tr>
      <td>6</td>
      <td colspan="4" style="background-color:#d0ebff; border-right: 1px solid var(--sl-color-gray-5, '#000');">Blue, pixels 1-4, bottom row</td>
      <td colspan="4" style="background-color:#d0ebff;">Blue, pixels 1-4, top row</td>
    </tr>
    <tr>
      <td>7</td>
      <td colspan="8" style="background-color:#d3f9d8;">Green, pixels 5-12, bottom row</td>
    </tr>
    <tr>
      <td>8</td>
      <td colspan="8" style="background-color:#ffe3e3;">Red, pixels 5-12, bottom row</td>
    </tr>
    <tr>
      <td>9</td>
      <td colspan="8" style="background-color:#d0ebff;">Blue, pixels 5-12, bottom row</td>
    </tr>
  </tbody>
</table>

## Implementation Notes

In a nutshell, to send data to the display:

1. Set multiplexers to the row to be displayed to.
1. Send data via shift registers to the selected row.
1. Loop over rows as necessary.

### Setting Multiplexers

1. Using the table above, set the multiplexer pins A0, A1 & A2 to the relevant values to select a row.
1. Wait a short delay to wait for the output to stabilize (e.g. 5µs).

### Sending Data to Shift Registers

1. Set **output enable high** and **latch low** to begin transmission.
1. Write data out using the SPI bus for accurate timing. I used an SPI speed of 8MHz.
   _Otherwise, send each bit (MSB), setting the **clock high**, then low after each bit with a short delay._
1. Set **latch high** to send the data out from the shift register to the row.
1. Wait a short delay (e.g. 2µs).
1. Set **latch low** and **output enable low**.

### Additional Notes

- If you see a ghosting effect, you can write a blank row after writing each row. I have done this after reading button presses so that the red artifacts from the button test patterns don't appear on the display.
- I have added a delay after writing data to where I start reading the buttons for that row. Writing display data too fast can result in dimmed LEDs (although maybe you want that?). From my AI companion, for a target of 60Hz, you would work out the time to wait by doing (1 ÷ 60Hz) ÷ 6 rows = 0.001388 = 2777ms. I have used a value of 1388ms as I had a flickering effect with it any higher.
