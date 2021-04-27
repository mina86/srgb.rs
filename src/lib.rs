//! The crate provides primitives for manipulating colours in sRGB colour space.
//! Specifically, it provides functions for converting between sRGB space,
//! linear sRGB space and XYZ colour space; as well as exposes the definition of
//! D65 reference white point as well as XYZ colour conversion matrices.
//!
//! The crate intents to provide low-level primitives needed to work with sRGB
//! colour space.  Those primitives can be used by other libraries which need to
//! convert between sRGB and other colour spaces (if the conversion requires
//! going through XYZ colour space) or blend colours together (which requires
//! performing gamma correction).
//!
//! Functions provided in the main module implement conversions between sRGB and
//! XYZ colour spaces while providing routines for intermediate conversions.
//! Functions in [`gamma`] submodule provide functions for doing gamma
//! compression and expansion; they operate on a single colour component.
//! Lastly, [`xyz`] submodule provides functions for converting between linear
//! sRGB and XYZ colour spaces as well as constants exposing the matrices used
//! by those functions.

pub mod gamma;
pub mod xyz;

mod maths;

pub use xyz::linear_from_xyz;
pub use xyz::xyz_from_linear;


/// Converts a 24-bit sRGB colour (also known as true colour) into normalised
/// representation.  Returns three components each normalised to the range 0–1.
///
/// This does exactly what one might expect: divides each component by 255.
///
/// # Example
/// ```
/// assert_eq!(
///     [0.9137255, 0.9098039, 0.90588236],
///     srgb::normalised_from_u8([233, 232, 231])
/// );
/// assert_eq!(
///     [0.83137256, 0.12941177, 0.23921569],
///     srgb::normalised_from_u8([212, 33, 61])
/// );
/// ```
#[inline]
pub fn normalised_from_u8(encoded: [u8; 3]) -> [f32; 3] {
    arr_map(encoded, |v| v as f32 / 255.0)
}

/// Converts an sRGB colour in normalised representation into a 24-bit (also
/// known as true colour).  That is, converts sRGB representation where each
/// component is a number in the range from zero to one to one where each
/// component is an 8-bit unsigned integer.  Components in source colour are
/// clamped to the valid range.
///
/// This is morally equivalent to multiplying each component by 255.
///
/// # Example
/// ```
/// assert_eq!(
///     [233, 232, 231],
///     srgb::u8_from_normalised([0.9137255, 0.9098039, 0.90588236])
/// );
/// assert_eq!(
///     [212, 33, 61],
///     srgb::u8_from_normalised([0.83137256, 0.12941177, 0.23921569])
/// );
/// ```
#[inline]
pub fn u8_from_normalised(normalised: [f32; 3]) -> [u8; 3] {
    // Adding 0.5 is for rounding.
    arr_map(normalised, |v| {
        maths::mul_add(v.clamp(0.0, 1.0), 255.0, 0.5) as u8
    })
}


/// Converts a 24-bit sRGB colour (also known as true colour) into linear space.
/// That is, performs gamma expansion on each component and returns the colour
/// in linear sRGB space with each component normalised to the range 0–1.
///
/// This is just a convenience wrapper around [`gamma::expand_u8()`] function.
///
/// # Example
/// ```
/// assert_eq!(
///     [0.8148465, 0.80695224, 0.7991027],
///     srgb::linear_from_u8([233, 232, 231])
/// );
/// assert_eq!(
///     [0.6583748, 0.015208514, 0.046665084],
///     srgb::linear_from_u8([212, 33, 61])
/// );
/// ```
#[inline]
pub fn linear_from_u8(encoded: [u8; 3]) -> [f32; 3] {
    arr_map(encoded, gamma::expand_u8)
}

/// Converts an sRGB colour in linear space to a 24-bit sRGB colour (also known
/// as true colour).  That is, performs gamma compression on each component and
/// encodes each component as an 8-bit integer.
///
/// This is just a convenience wrapper around [`gamma::compress_u8()`] function.
///
/// # Example
/// ```
/// assert_eq!(
///     [233, 232, 231],
///     srgb::u8_from_linear([0.8148465, 0.80695224, 0.7991027])
/// );
/// assert_eq!(
///     [212, 33, 61],
///     srgb::u8_from_linear([0.6583748, 0.015208514, 0.046665084])
/// );
/// ```
#[inline]
pub fn u8_from_linear(linear: [f32; 3]) -> [u8; 3] {
    arr_map(linear, gamma::compress_u8)
}


/// Converts an sRGB colour in normalised representation into linear space.
/// That is, performs gamma expansion on each component (which should be in 0–1
/// range) and returns the colour in linear space.
///
/// This is just a convenience wrapper around [`gamma::expand_normalised()`]
/// function.
///
/// # Example
/// ```
/// assert_eq!(
///     [0.8148467, 0.80695236, 0.79910284],
///     srgb::linear_from_normalised([0.9137255, 0.9098039, 0.90588236])
/// );
/// assert_eq!(
///     [0.65837485, 0.015208514, 0.046665084],
///     srgb::linear_from_normalised([0.83137256, 0.12941177, 0.23921569])
/// );
/// ```
#[inline]
pub fn linear_from_normalised(normalised: [f32; 3]) -> [f32; 3] {
    arr_map(normalised, gamma::expand_normalised)
}

/// Converts an sRGB colour in linear space to normalised space.  That is,
/// performs gamma compression on each component (which should be in 0–1 range)
/// and encodes each component as an 8-bit integer.
///
/// This is just a convenience wrapper around [`gamma::compress_normalised()`]
/// function.
///
/// # Example
/// ```
/// assert_eq!(
///     [0.8148467, 0.80695236, 0.79910284],
///     srgb::linear_from_normalised([0.9137255, 0.9098039, 0.90588236])
/// );
/// assert_eq!(
///     [0.65837485, 0.015208514, 0.046665084],
///     srgb::linear_from_normalised([0.83137256, 0.12941177, 0.23921569])
/// );
/// ```
#[inline]
pub fn normalised_from_linear(linear: [f32; 3]) -> [f32; 3] {
    arr_map(linear, gamma::compress_normalised)
}



#[inline]
fn arr_map<F: Copy, T: Copy, Fun: Fn(F) -> T>(arr: [F; 3], f: Fun) -> [T; 3] {
    [f(arr[0]), f(arr[1]), f(arr[2])]
}
