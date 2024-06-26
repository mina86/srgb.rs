/* This file is part of srgb crate.
 * Copyright 2021 by Michał Nazarewicz <mina86@mina86.com>
 *
 * srgb crate is free software: you can redistribute it and/or modify it under
 * the terms of the GNU Lesser General Public License as published by the Free
 * Software Foundation; either version 3 of the License, or (at your option) any
 * later version.
 *
 * srgb crate is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
 * A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * srgb crate.  If not, see <http://www.gnu.org/licenses/>. */

#![doc = include_str!("../README.md")]
#![allow(clippy::excessive_precision)]
#![allow(clippy::needless_doctest_main)]

pub mod gamma;
pub mod xyz;

mod maths;


/// Converts a 24-bit sRGB colour (also known as true colour) into normalised
/// representation.
///
/// Returns three components each normalised to the range 0–1.  This does
/// exactly what one might expect: divides each component by 255.
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
#[doc(hidden)]
pub fn normalised_from_u8(encoded: impl Into<[u8; 3]>) -> [f32; 3] {
    arr_map(encoded, |v| v as f32 / 255.0)
}

/// Converts an sRGB colour in normalised representation into a 24-bit (also
/// known as true colour).
///
/// That is, converts sRGB representation where each component is a number in
/// the range from zero to one to one where each component is an 8-bit unsigned
/// integer.  Components in source colour are clamped to the valid range.  This
/// is roughly equivalent to multiplying each component by 255.
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
#[doc(hidden)]
pub fn u8_from_normalised(normalised: impl Into<[f32; 3]>) -> [u8; 3] {
    // Adding 0.5 is for rounding.
    arr_map(normalised, |v| v.clamp(0.0, 1.0).mul_add(255.0, 0.5) as u8)
}


/// Converts a colour in an XYZ colour space into 24-bit sRGB representation.
///
/// This is just a convenience function which wraps gamma (see [`gamma`] module)
/// and XYZ (see [`xyz`] module) conversions function together.
pub fn u8_from_xyz(xyz: impl Into<[f32; 3]>) -> [u8; 3] {
    gamma::u8_from_linear(xyz::linear_from_xyz(xyz))
}

/// Converts a 24-bit sRGB colour into XYZ colour space.
///
/// This is just a convenience function which wraps gamma (see [`gamma`] module)
/// and XYZ (see [`xyz`] module) conversions function together.
pub fn xyz_from_u8(rgb: impl Into<[u8; 3]>) -> [f32; 3] {
    xyz::xyz_from_linear(gamma::linear_from_u8(rgb))
}

/// Converts a colour in an XYZ colour space into a normalised sRGB
/// representation.
///
/// This is just a convenience function which wraps gamma (see [`gamma`] module)
/// and XYZ (see [`xyz`] module) conversions function together.
pub fn normalised_from_xyz(xyz: impl Into<[f32; 3]>) -> [f32; 3] {
    gamma::normalised_from_linear(xyz::linear_from_xyz(xyz))
}

/// Converts a normalised representation of a sRGB colour into XYZ colour space.
///
/// This is just a convenience function which wraps gamma (see [`gamma`] module)
/// and XYZ (see [`xyz`] module) conversions function together.
pub fn xyz_from_normalised(rgb: impl Into<[f32; 3]>) -> [f32; 3] {
    xyz::xyz_from_linear(gamma::linear_from_normalised(rgb))
}


pub(crate) fn arr_map<F: Copy, T: Copy, Fun: Fn(F) -> T>(
    arr: impl Into<[F; 3]>,
    f: Fun,
) -> [T; 3] {
    let arr = arr.into();
    [f(arr[0]), f(arr[1]), f(arr[2])]
}


#[cfg(test)]
mod test {
    use kahan::KahanSummator;

    const WHITE_X: f64 = 0.312713;
    const WHITE_Y: f64 = 0.329016;

    fn measure_grey_chromaticity_error(f: impl Fn(u8) -> [f32; 3]) -> f64 {
        // Grey colours should have chromaticity equal white point’s
        // chromaticity.
        let mut error = kahan::KahanSum::new();
        for i in 1..=255 {
            let [x, y, z] = f(i);
            let d: f64 =
                [x as f64, y as f64, z as f64].iter().kahan_sum().sum();
            let x = x as f64 / d - WHITE_X;
            let y = y as f64 / d - WHITE_Y;
            error += x * x;
            error += y * y;
        }
        error.sum() * 1e15
    }

    #[test]
    fn test_grey_chromaticity_error_u8() {
        assert_eq!(
            48.99296021015466,
            measure_grey_chromaticity_error(|i| {
                super::xyz_from_u8([i, i, i])
            })
        );
    }

    #[test]
    fn test_grey_chromaticity_error_normalised() {
        assert_eq!(
            39.81365168327802,
            measure_grey_chromaticity_error(|i| {
                let v = i as f32 / 255.0;
                super::xyz_from_normalised([v, v, v])
            })
        );
    }

    #[test]
    fn test_grey_chromaticity_error_linear() {
        assert_eq!(
            50.927415198412874,
            measure_grey_chromaticity_error(|i| {
                let v = i as f32 / 255.0;
                crate::xyz::xyz_from_linear([v, v, v])
            })
        );
    }
}
