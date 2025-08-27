---
title: Rust API
---
# Crate Documentation

**Version:** 0.1.0

**Format Version:** 45

# Module `arcadecoder_hw`

Simple Rust library for working with the [Tech Will Save Us](https://en.wikipedia.org/wiki/Technology_Will_Save_Us) Arcade Coder.

More projects, info and credits on Arcade Coder are [available here](https://github.com/padraigfl/awesome-arcade-coder), and hardware documentation is available [here](https://github.com/padraigfl/awesome-arcade-coder/wiki).

Currently the display and single button presses work.

## Modules

## Module `font`

Simple fonts for displaying numbers

Fonts are made up of an array of booleans representing on and off pixels.

```rust
pub mod font { /* ... */ }
```

### Types

#### Type Alias `Font`

```rust
pub type Font<''a> = &''a [&''a [bool]];
```

### Constants and Statics

#### Constant `FONT_5X5_SIZE`

```rust
pub const FONT_5X5_SIZE: (usize, usize) = _;
```

#### Static `FONT_5X5`

A basic 5x5 pixel font

```rust
pub static FONT_5X5: Font<''_> = _;
```

## Types

### Type Alias `Coordinates`

Display coordinates

```rust
pub type Coordinates = (usize, usize);
```

### Type Alias `Color`

3-bit color

```rust
pub type Color = (bool, bool, bool);
```

### Enum `ButtonEvent`

```rust
pub enum ButtonEvent {
    Pressed(u8, u8),
    Released(u8, u8),
}
```

#### Variants

##### `Pressed`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `u8` |  |
| 1 | `u8` |  |

##### `Released`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `u8` |  |
| 1 | `u8` |  |

#### Implementations

##### Trait Implementations

- **Freeze**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> ButtonEvent { /* ... */ }
    ```

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Sync**
- **Unpin**
- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **Copy**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **RefUnwindSafe**
- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **Send**
- **UnwindSafe**
- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Same**
### Struct `ArcadeCoder`

```rust
pub struct ArcadeCoder<''a> {
    pub channel_select_delay: embassy_time::Duration,
    pub latch_delay: embassy_time::Duration,
    pub display_buffer: [[u8; 9]; 6],
    pub button_presses: [[bool; 12]; 12],
    pub channel_on_time: embassy_time::Duration,
    pub debounce_reads: u8,
    // Some fields omitted
}
```

#### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `channel_select_delay` | `embassy_time::Duration` | The time to wait after switching channels for inputs to settle. |
| `latch_delay` | `embassy_time::Duration` | The time to wait after latching. |
| `display_buffer` | `[[u8; 9]; 6]` | The current display buffer. |
| `button_presses` | `[[bool; 12]; 12]` | A matrix of button presses corresponding to the physical layout. |
| `channel_on_time` | `embassy_time::Duration` | The time to wait after displaying a row on the display. |
| `debounce_reads` | `u8` | The number of reads required for a button press to register. |
| *private fields* | ... | *Some fields have been omitted* |

#### Implementations

##### Methods

- ```rust
  pub fn new</* synthetic */ impl OutputPin + 'a: OutputPin + ''a, /* synthetic */ impl OutputPin + 'a: OutputPin + ''a, /* synthetic */ impl OutputPin + 'a: OutputPin + ''a, /* synthetic */ impl OutputPin + 'a: OutputPin + ''a, /* synthetic */ impl OutputPin + 'a: OutputPin + ''a, /* synthetic */ impl OutputPin + 'a: OutputPin + ''a, /* synthetic */ impl OutputPin + 'a: OutputPin + ''a, /* synthetic */ impl InputPin + 'a: InputPin + ''a, /* synthetic */ impl InputPin + 'a: InputPin + ''a, /* synthetic */ impl InputPin + 'a: InputPin + ''a, /* synthetic */ impl InputPin + 'a: InputPin + ''a, /* synthetic */ impl InputPin + 'a: InputPin + ''a, /* synthetic */ impl InputPin + 'a: InputPin + ''a>(spi_bus: SPI2<''a>, pin_a0: impl OutputPin + ''a, pin_a1: impl OutputPin + ''a, pin_a2: impl OutputPin + ''a, pin_oe: impl OutputPin + ''a, pin_latch: impl OutputPin + ''a, pin_data: impl OutputPin + ''a, pin_clock: impl OutputPin + ''a, inputs_1_7: impl InputPin + ''a, inputs_2_8: impl InputPin + ''a, inputs_3_9: impl InputPin + ''a, inputs_4_10: impl InputPin + ''a, inputs_5_11: impl InputPin + ''a, inputs_6_12: impl InputPin + ''a) -> Self { /* ... */ }
  ```
  Create a new instance of the Arcade Coder.

- ```rust
  pub fn clear(self: &mut Self) { /* ... */ }
  ```
  Clear the display buffer to make the screen blank.

- ```rust
  pub fn set_pixel(self: &mut Self, pos: Coordinates, color: Color) { /* ... */ }
  ```
  Set a pixel to a color

- ```rust
  pub fn draw_rect(self: &mut Self, pos1: Coordinates, pos2: Coordinates, color: Color) { /* ... */ }
  ```

- ```rust
  pub fn draw_digit(self: &mut Self, n: u32, font: &[&[bool]], font_size: (usize, usize), start_pos: Coordinates, color: Color) { /* ... */ }
  ```
  Draw a digit from a font

- ```rust
  pub fn draw_char(self: &mut Self, character: char, font: &[&[bool]], font_size: (usize, usize), start_pos: Coordinates, color: Color) { /* ... */ }
  ```
  Draw a character from a font

- ```rust
  pub fn handle_input_events<F>(self: &mut Self, handler: F)
where
    F: FnMut(ButtonEvent) { /* ... */ }
  ```
  Handle button press events. This takes care of debouncing inputs and returns an event for button presses and releases.

- ```rust
  pub fn handle_input_events_to_channel<const N: usize>(self: &mut Self, ch: &embassy_sync::channel::Channel<embassy_sync::blocking_mutex::raw::NoopRawMutex, ButtonEvent, N>) { /* ... */ }
  ```
  Handle button press events. This takes care of debouncing inputs and returns an event for button presses and releases.

- ```rust
  pub async fn scan(self: &mut Self) { /* ... */ }
  ```
  Update the display while also scanning for button inputs.

##### Trait Implementations

- **RefUnwindSafe**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **UnwindSafe**
- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **Unpin**
- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Freeze**
- **Send**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Same**
- **Sync**
## Constants and Statics

### Constant `WHITE`

```rust
pub const WHITE: Color = _;
```

### Constant `YELLOW`

```rust
pub const YELLOW: Color = _;
```

### Constant `CYAN`

```rust
pub const CYAN: Color = _;
```

### Constant `RED`

```rust
pub const RED: Color = _;
```

### Constant `MAGENTA`

```rust
pub const MAGENTA: Color = _;
```

### Constant `GREEN`

```rust
pub const GREEN: Color = _;
```

### Constant `BLUE`

```rust
pub const BLUE: Color = _;
```

### Constant `BLACK`

```rust
pub const BLACK: Color = _;
```

