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
pub fn xyz_from_linear(linear: [f32; 3]) -> [f32; 3] {
    crate::maths::matrix_product(&XYZ_FROM_SRGB_MATRIX, linear)
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
pub fn linear_from_xyz(xyz: [f32; 3]) -> [f32; 3] {
    crate::maths::matrix_product(&SRGB_FROM_XYZ_MATRIX, xyz.into())
}


include!(concat!(env!("OUT_DIR"), "/xyz_constants.rs"));


#[cfg(test)]
mod test {
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
        for c in 0..(16 * 16 * 16) {
            let r = (c & 15) as f32 / 15.0;
            let g = ((c >> 4) & 15) as f32 / 15.0;
            let b = ((c >> 8) & 15) as f32 / 15.0;
            let src = [r, g, b];
            let xyz = super::xyz_from_linear(src);
            let dst = super::linear_from_xyz(xyz);
            approx::assert_abs_diff_eq!(&src[..], &dst[..], epsilon = 0.000001);
        }
    }
}
