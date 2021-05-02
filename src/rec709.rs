//! Functions for normalising component values given in
//! [Rec.709](https://www.itu.int/rec/R-REC-BT.709-6-201506-I/en) coding.
//!
//! Rec.709 defines 8-bit and 10-bit coding for the R, G and B component
//! values.  In the 8-bit coding the range for nominal values is [16, 235].  In
//! 10-bit coding the range for nominal values is [64, 940].
//!
//! This module provides functions for convening between those encodings and
//! normalised representation (that is one with floating point values in the
//! range [0.0, 1.0]).
//!
//! Note: To perform gamma compression or expansion on Rec.709-compatible values
//! use functions is the [`crate::gamma`] module.

/// Converts 8-bit encoded Rec.709 value into normalised form.
///
/// # Example
///
/// ```
/// assert_eq!(0.0, srgb::rec709::decode_rec709_8bit(0));
/// assert_eq!(0.0, srgb::rec709::decode_rec709_8bit(16));
/// assert_eq!(0.5022831, srgb::rec709::decode_rec709_8bit(126));
/// assert_eq!(1.0, srgb::rec709::decode_rec709_8bit(235));
/// assert_eq!(1.0, srgb::rec709::decode_rec709_8bit(255));
/// ```
#[inline]
pub fn decode_rec709_8bit(value: u8) -> f32 {
    (value.clamp(16, 235) - 16) as f32 / (235 - 16) as f32
}

/// Converts 10-bit encoded Rec.709 value into normalised form.
///
/// # Example
///
/// ```
/// assert_eq!(0.0, srgb::rec709::decode_rec709_10bit(0));
/// assert_eq!(0.0, srgb::rec709::decode_rec709_10bit(64));
/// assert_eq!(0.50570774, srgb::rec709::decode_rec709_10bit(507));
/// assert_eq!(1.0, srgb::rec709::decode_rec709_10bit(940));
/// assert_eq!(1.0, srgb::rec709::decode_rec709_10bit(940));
/// ```
#[inline]
pub fn decode_rec709_10bit(value: u16) -> f32 {
    (value.clamp(64, 940) - 64) as f32 / (940 - 64) as f32
}

/// Converts normalised value into 8-bit Rec.709 coding.
#[inline]
pub fn encode_rec709_8bit(value: f32) -> u8 {
    value.clamp(0.0, 1.0).mul_add((235 - 16) as f32, 16.5) as u8
}

/// Converts normalised value into 10-bit Rec.709 coding.
#[inline]
pub fn encode_rec709_10bit(value: f32) -> u16 {
    value.clamp(0.0, 1.0).mul_add((940 - 64) as f32, 64.5) as u16
}
