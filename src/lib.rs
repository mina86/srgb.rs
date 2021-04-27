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
#[doc(hidden)]
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
#[doc(hidden)]
pub fn u8_from_normalised(normalised: [f32; 3]) -> [u8; 3] {
    // Adding 0.5 is for rounding.
    arr_map(normalised, |v| {
        maths::mul_add(v.clamp(0.0, 1.0), 255.0, 0.5) as u8
    })
}


/// Converts a colour in an XYZ colour space into 24-bit sRGB representation.
///
/// This is just a convenience function which wraps gamma (see ['gamma'] module)
/// and XYZ (see ['xyz'] module) conversions function together.
pub fn u8_from_xyz(xyz: [f32; 3]) -> [u8; 3] {
    gamma::u8_from_linear(xyz::linear_from_xyz(xyz))
}

/// Converts a 24-bit sRGB colour into XYZ colour space.
///
/// This is just a convenience function which wraps gamma (see ['gamma'] module)
/// and XYZ (see ['xyz'] module) conversions function together.
pub fn xyz_from_u8(rgb: [u8; 3]) -> [f32; 3] {
    xyz::xyz_from_linear(gamma::linear_from_u8(rgb))
}

/// Converts a colour in an XYZ colour space into a normalised sRGB
/// representation.
///
/// This is just a convenience function which wraps gamma (see ['gamma'] module)
/// and XYZ (see ['xyz'] module) conversions function together.
pub fn normalised_from_xyz(xyz: [f32; 3]) -> [f32; 3] {
    gamma::normalised_from_linear(xyz::linear_from_xyz(xyz))
}

/// Converts a normalised representation of a sRGB colour into XYZ colour space.
///
/// This is just a convenience function which wraps gamma (see ['gamma'] module)
/// and XYZ (see ['xyz'] module) conversions function together.
pub fn xyz_from_normalised(rgb: [f32; 3]) -> [f32; 3] {
    xyz::xyz_from_linear(gamma::linear_from_normalised(rgb))
}


#[inline]
pub(crate) fn arr_map<F: Copy, T: Copy, Fun: Fn(F) -> T>(
    arr: [F; 3],
    f: Fun,
) -> [T; 3] {
    [f(arr[0]), f(arr[1]), f(arr[2])]
}
