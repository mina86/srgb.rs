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

//! Functions and constant handling and related to conversion between linear
//! sRGB space and CIE XYZ colour space.

/// Converts a colour in linear sRGB space into an XYZ colour space.
///
/// The colour is given as three components each in the range from zero to one.
/// Resulting XYZ space is one where white colour has Y coordinate equal one.
///
/// # Example
/// ```
/// use srgb::xyz::xyz_from_linear;
///
/// let white = [0.88901603, 0.7947985, 0.8663711];
/// let red = [0.69039214, 0.013060069, 0.053315595];
///
/// assert_eq!([0.8071875, 0.82000005, 0.9353126], xyz_from_linear(white));
/// assert_eq!([0.2990163, 0.16, 0.0655738], xyz_from_linear(red));
/// ```
pub fn xyz_from_linear(linear: impl Into<[f32; 3]>) -> [f32; 3] {
    crate::maths::matrix_product(&XYZ_FROM_SRGB_MATRIX, linear.into())
}

/// Converts a colour in an XYZ space into a linear sRGB colour space.
///
/// The colour is given as three floating point components.  The source XYZ
/// space should be such where white colour has Y coordinate equal one.  The
/// result will be given as a three numbers in the range from zero to one.
///
/// # Example
/// ```
/// use srgb::xyz::linear_from_xyz;
///
/// let white = [0.8071875, 0.82, 0.9353125];
/// let red = [0.2990163, 0.16, 0.0655738];
///
/// assert_eq!([0.88901603, 0.7947985, 0.8663711], linear_from_xyz(white));
/// assert_eq!([0.69039214, 0.013060069, 0.053315595], linear_from_xyz(red));
/// ```
pub fn linear_from_xyz(xyz: impl Into<[f32; 3]>) -> [f32; 3] {
    crate::maths::matrix_product(&SRGB_FROM_XYZ_MATRIX, xyz.into())
}


include!(concat!(env!("OUT_DIR"), "/xyz_constants.rs"));


#[cfg(test)]
mod test {
    use xsum::Xsum;

    #[test]
    fn test_d65() {
        let [x, y, _] = super::D65_xyY;
        assert_eq!((0.312713, 0.329016), (x, y));

        let want = [
            (x as f64 / y as f64) as f32,
            1.0,
            ((1.0 - x as f64 - y as f64) / y as f64) as f32,
        ];
        let got = super::D65_XYZ;
        assert_eq!(&want[..], &got[..]);
    }

    #[test]
    fn test_reversible_conversion() {
        let mut error = xsum::XsumSmall::default();
        for c in 0..(16 * 16 * 16) {
            let r = (c & 15) as f32 / 15.0;
            let g = ((c >> 4) & 15) as f32 / 15.0;
            let b = ((c >> 8) & 15) as f32 / 15.0;
            let src = [r, g, b];
            let dst = super::linear_from_xyz(super::xyz_from_linear(src));
            approx::assert_abs_diff_eq!(&src[..], &dst[..], epsilon = 0.000001);

            let r = r as f64 - dst[0] as f64;
            let g = g as f64 - dst[1] as f64;
            let b = b as f64 - dst[2] as f64;
            error.add(r * r);
            error.add(g * g);
            error.add(b * b);
        }
        assert_eq!(62.71521153793259, error.sum() * 1e12);
    }
}
