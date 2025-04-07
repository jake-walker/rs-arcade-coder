//! Simple fonts for displaying numbers
//!
//! Fonts are made up of an array of booleans representing on and off pixels.

pub type Font<'a> = &'a [&'a [bool]];

pub const FONT_5X5_SIZE: (usize, usize) = (5, 5);

/// A basic 5x5 pixel font
pub static FONT_5X5: Font = &[
    // 0
    &[
        false, true, true, true, false, true, false, false, false, true, true, false, false, false,
        true, true, false, false, false, true, false, true, true, true, false,
    ],
    // 1
    &[
        false, false, true, false, false, true, true, true, false, false, false, false, true,
        false, false, false, false, true, false, false, true, true, true, true, true,
    ],
    // 2
    &[
        true, true, true, true, false, false, false, false, false, true, false, true, true, true,
        false, true, false, false, false, false, true, true, true, true, true,
    ],
    // 3
    &[
        true, true, true, true, false, false, false, false, false, true, true, true, true, true,
        false, false, false, false, false, true, true, true, true, true, false,
    ],
    // 4
    &[
        true, false, false, true, false, true, false, false, true, false, true, false, false, true,
        false, true, true, true, true, true, false, false, false, true, false,
    ],
    // 5
    &[
        true, true, true, true, true, true, false, false, false, false, true, true, true, true,
        false, false, false, false, false, true, true, true, true, true, false,
    ],
    // 6
    &[
        false, true, true, true, false, true, false, false, false, false, true, true, true, true,
        false, true, false, false, false, true, false, true, true, true, false,
    ],
    // 7
    &[
        true, true, true, true, true, false, false, false, false, true, false, false, false, true,
        false, false, false, true, false, false, false, false, true, false, false,
    ],
    // 8
    &[
        false, true, true, true, false, true, false, false, false, true, false, true, true, true,
        false, true, false, false, false, true, false, true, true, true, false,
    ],
    // 9
    &[
        false, true, true, true, false, true, false, false, false, true, false, true, true, true,
        true, false, false, false, false, true, false, true, true, true, false,
    ],
];
