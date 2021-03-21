//! Functions and constant handling and related to conversion between linear
//! sRGB space and CIE XYZ colour space.

/// Converts a colour in linear sRGB space into an XYZ colour space.  The colour
/// is given as three components each in the range from zero to one.  Resulting
/// XYZ space is one where white colour has Y coordinate equal one.
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
    matrix_product(&XYZ_FROM_SRGB_MATRIX, linear)
}

/// Converts a colour in an XYZ space into a linear sRGB colour space.  The
/// colour is given as three floating point components.  The source XYZ space
/// should be such where white colour has Y coordinate equal one.  The result
/// will be given as a three numbers in the range from zero to one.
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
    matrix_product(&SRGB_FROM_XYZ_MATRIX, xyz.into())
}


/// xyY coordinates of the D65 reference white-point used in sRGB colour space.
#[allow(non_upper_case_globals)]
pub const D65_xyY: [f32; 3] = [
    D65_x_NUM as f32 / D65_DENOM as f32,
    D65_y_NUM as f32 / D65_DENOM as f32,
    1.0,
];

/* X = x/y
 * Y = 1
 * Z = (1 - x - y) / y
 */
/// XYZ coordinates of the D65 reference white-point used in sRGB colour space.
pub const D65_XYZ: [f32; 3] = [
    D65_x_NUM as f32 / D65_y_NUM as f32,
    1.0,
    (D65_DENOM - D65_x_NUM - D65_y_NUM) as f32 / D65_y_NUM as f32,
];


/// The basis conversion matrix from moving from linear sRGB space to XYZ colour
/// space.  To perform the conversion it’s typically more convenient to use the
/// xyz_from_linear() function instead of accessing this constant.
///
/// The matrix is built with the assumption that colours are represented as
/// one-column matrices.  With that, converting from sRGB to XYZ is done by the
/// following formula: `XYZ = XYZ_FROM_SRGB_MATRIX ✕ RGB`.
#[rustfmt::skip]
pub const XYZ_FROM_SRGB_MATRIX: [[f32; 3]; 3] = [
    [0.4124108464885388,   0.3575845678529519,  0.18045380393360833],
    [0.21264934272065283,  0.7151691357059038,  0.07218152157344333],
    [0.019331758429150258, 0.11919485595098397, 0.9503900340503373],
];

/// The basis conversion matrix from moving from XYZ to linear sRGB colour
/// space.  To perform the conversion it’s typically more convenient to use the
/// linear_from_xyz() function instead of accessing this constant.
///
/// The matrix is built with the assumption that colours are represented as
/// one-column matrices.  With that, converting from XYZ to sRGB is done by the
/// following formula: `RGB = SRGB_FROM_XYZ_MATRIX ✕ XYZ`.
#[rustfmt::skip]
pub const SRGB_FROM_XYZ_MATRIX: [[f32; 3]; 3] = [
    [ 3.240812398895283,    -1.5373084456298136, -0.4985865229069666],
    [-0.9692430170086407,    1.8759663029085742,  0.04155503085668564],
    [ 0.055638398436112804, -0.20400746093241362, 1.0571295702861434],
];


#[allow(non_upper_case_globals)]
const D65_x_NUM: u32 = 312713;
#[allow(non_upper_case_globals)]
const D65_y_NUM: u32 = 329016;
#[allow(non_upper_case_globals)]
const D65_DENOM: u32 = 1000000;


#[inline(always)]
fn matrix_product(matrix: &[[f32; 3]; 3], column: [f32; 3]) -> [f32; 3] {
    [
        crate::maths::dot_product(&matrix[0], &column),
        crate::maths::dot_product(&matrix[1], &column),
        crate::maths::dot_product(&matrix[2], &column),
    ]
}



#[cfg(test)]
mod test {
    use approx::assert_abs_diff_eq;
    use approx::assert_ulps_eq;

    #[derive(Debug, PartialEq)]
    struct Arr([f32; 3]);

    impl approx::AbsDiffEq for Arr {
        type Epsilon = <f32 as approx::AbsDiffEq>::Epsilon;

        fn default_epsilon() -> Self::Epsilon { 0.000001 }
        fn abs_diff_eq(&self, rhs: &Self, epsilon: Self::Epsilon) -> bool {
            self.0
                .iter()
                .zip(rhs.0.iter())
                .all(|(a, b)| a.abs_diff_eq(b, epsilon))
        }
    }

    impl approx::UlpsEq for Arr {
        fn default_max_ulps() -> u32 { f32::default_max_ulps() }
        fn ulps_eq(
            &self,
            rhs: &Self,
            epsilon: Self::Epsilon,
            max_ulps: u32,
        ) -> bool {
            self.0
                .iter()
                .zip(rhs.0.iter())
                .all(|(a, b)| a.ulps_eq(b, epsilon, max_ulps))
        }
    }

    #[test]
    fn test_d65() {
        let [x, y, _] = super::D65_xyY;
        let want = [x / y, 1.0, (1.0 - x - y) / y];
        let got = super::D65_XYZ;
        for (w, g) in want.iter().zip(got.iter()) {
            assert_ulps_eq!(w, g, max_ulps = 1)
        }
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
            assert_abs_diff_eq!(Arr(src), Arr(dst));
        }
    }
}
