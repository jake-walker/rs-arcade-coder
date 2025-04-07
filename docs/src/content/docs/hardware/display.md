---
title: Display
---

The display on the Arcade Coder is a 24x24 LED matrix. Each pixel works in 3-bit colour (giving 8 total colours). Each pixel also serves as a [button](../button).

The matrix is split into groups by row. The multiplexer selects the rows to be drawn to, and the data is sent to the rows via the shift register.

## Pins

### HC595 Shift Register

| ESP    | Shift Register |
| ------ | -------------- |
| GPIO5  | Data In        |
| GPIO17 | Clock          |
| GPIO16 | Latch          |
| GPIO4  | Output Enable  |

### ICN2012 Multiplexer

The rows are selected by setting different combinations on the multiplexer.

| ESP    | Multiplexer |
| ------ | ----------- |
| GPIO19 | A0          |
| GPIO18 | A1          |
| GPIO21 | A2          |

| A0  | A1  | A2  | Rows   |
| --- | --- | --- | ------ |
| 0   | 0   | 0   | _?_    |
| 0   | 0   | 1   | 4 & 9  |
| 0   | 1   | 0   | 1 & 6  |
| 0   | 1   | 1   | 6 & 12 |
| 1   | 0   | 0   | 5 & 10 |
| 1   | 0   | 1   | 3 & 8  |
| 1   | 1   | 0   | 2 & 7  |
| 1   | 1   | 1   | _?_    |

## Driving

:::note
I used an SPI speed of 200kHz. Much faster introduces flickering, and much slower doesn't work at all.
:::

1. For each row, set the multiplexer (A0, A1 & A2) according to the table above to select the rows to be drawn to. _Consider adding a short (e.g. 50µs) delay afterwards for the output to stabilize._
1. Set **output enable low**, which is left low for the entire process, and set **latch low**.
1. Write data out using the SPI bus for accurate timing. Otherwise, send each bit (MSB), setting the **clock high**, then low after each bit with a short delay.
1. Set **latch high**, wait for a short time (e.g. 50µs), then set **latch low**.
1. Repeat for other rows.

Ideally, you want to run this loop as fast as possible (≥ 30 times per second) for a proper persistence of vision effect.

### Data Format

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
      <td colspan="4" style="background-color:#d3f9d8;">Green, pixels 1-4, bottom row</td>
      <td colspan="4" style="background-color:#d3f9d8;">Green, pixels 1-4, top row</td>
    </tr>
    <tr>
      <td>5</td>
      <td colspan="4" style="background-color:#ffe3e3;">Red, pixels 1-4, bottom row</td>
      <td colspan="4" style="background-color:#ffe3e3;">Red, pixels 1-4, top row</td>
    </tr>
    <tr>
      <td>6</td>
      <td colspan="4" style="background-color:#d0ebff;">Blue, pixels 1-4, bottom row</td>
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
