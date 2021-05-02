use std::io::Write;

use num::One;
use num::Zero;

type Scalar = num::BigRational;
type Chromacity = rgb_derivation::Chromacity<Scalar>;


fn scalar(numer: i64, denom: i64) -> num::BigRational {
    num::BigRational::new(numer.into(), denom.into())
}

fn chromacity(x: (i64, i64), y: (i64, i64)) -> Chromacity {
    Chromacity::new(scalar(x.0, x.1), scalar(y.0, y.1)).unwrap()
}

/// Formats scalar as a floating point number.  If denominator isn’t one, the
/// number is formatted as `n / d` string (where `n` and `d` are integers
/// written as floating point numbers); otherwise just the numerator is written.
fn fmt_scalar(scalar: &Scalar) -> std::string::String {
    let (numer, denom) = (scalar.numer(), scalar.denom());
    if numer.is_zero() || denom.is_one() {
        format!("{}.0", numer.to_str_radix(10))
    } else {
        format!(
            "{}.0 / {}.0",
            numer.to_str_radix(10),
            denom.to_str_radix(10)
        )
    }
}

fn fmt_vector(vec: &[Scalar; 3]) -> std::string::String {
    format!(
        "[{}, {}, {}]",
        fmt_scalar(&vec[0]),
        fmt_scalar(&vec[1]),
        fmt_scalar(&vec[2])
    )
}

fn fmt_matrix<T, D: std::fmt::Display>(
    matrix: &[T; 3],
    fmt: impl Fn(&T) -> D,
) -> std::string::String {
    format!(
        r#"[
    {},
    {},
    {},
]"#,
        fmt(&matrix[0]),
        fmt(&matrix[1]),
        fmt(&matrix[2])
    )
}

fn fmt_chromacity(ch: &Chromacity) -> std::string::String {
    fmt_vector(&[ch.x().clone(), ch.y().clone(), One::one()])
}


fn gamma_compress_lin_part<T: num::traits::Float + num::traits::NumRef>(
    x: &T,
) -> T {
    T::from(12.92_f64).unwrap() * x
}

fn gamma_compress_exp_part<T: num::traits::Float + num::traits::NumRef>(
    x: &T,
) -> T {
    let exponent = T::from(5.0_f64 / 12.0_f64).unwrap();
    x.powf(exponent) * T::from(1.055).unwrap() - T::from(0.055).unwrap()
}

pub fn calc_gamma_threshold<
    T: PartialOrd + num::traits::Float + num::traits::NumRef,
>() -> T
where
    for<'r> &'r T: num::traits::RefNum<T>, {
    let mut lo = T::from(0.0030_f64).unwrap();
    let mut hi = T::from(0.0032_f64).unwrap();
    let mut mid = T::zero();
    for _ in 0..(std::mem::size_of::<T>() * 8) {
        mid = (&lo + &hi) / T::from(2_i16).unwrap();
        let lhs = gamma_compress_lin_part(&mid);
        let rhs = gamma_compress_exp_part(&mid);
        match lhs.partial_cmp(&rhs) {
            Some(std::cmp::Ordering::Less) => lo = mid.clone(),
            Some(std::cmp::Ordering::Greater) => hi = mid.clone(),
            _ => break,
        };
    }
    mid
}


fn write_to(
    dir: impl AsRef<std::path::Path>,
    file_name: impl AsRef<std::ffi::OsStr>,
    args: std::fmt::Arguments,
) -> std::io::Result<()> {
    let dest = dir.as_ref().join(file_name.as_ref());
    let mut dest = std::fs::File::create(dest)?;
    dest.write_fmt(args)
}

fn generate() -> std::io::Result<()> {
    let out_dir = if let Some(dir) = std::env::var_os("OUT_DIR") {
        dir
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "missing OUT_DIR environment variable",
        ));
    };

    let white_xy = chromacity((312713, 1000000), (329016, 1000000));
    let primaries_xy = [
        chromacity((64, 100), (33, 100)),
        chromacity((30, 100), (60, 100)),
        chromacity((15, 100), (6, 100)),
    ];

    let white_xyz = white_xy.to_xyz();
    let matrix =
        rgb_derivation::matrix::calculate(&white_xyz, &primaries_xy).unwrap();
    let inverse = rgb_derivation::matrix::inversed_copy(&matrix).unwrap();
    let primaries_xyz = rgb_derivation::matrix::transposed_copy(&matrix);

    write_to(
        &out_dir,
        "xyz_constants.rs",
        format_args!(
            r"// Generated by build.rs

/// xyY coordinates of the D65 reference white-point used in sRGB colour space.
#[allow(non_upper_case_globals)]
pub const D65_xyY: [f32; 3] = {white_xyY};

/// XYZ coordinates of the D65 reference white-point used in sRGB colour space.
pub const D65_XYZ: [f32; 3] = {white_XYZ};

/// xyY coordinates of red, green and blue primaries defining the sRGB space.
#[allow(non_upper_case_globals)]
pub const PRIMARIES_xyY: [[f32; 3]; 3] = {primaries_xyY};

/// XYZ coordinates of red, green and blue primaries defining the sRGB space.
pub const PRIMARIES_XYZ: [[f32; 3]; 3] = {primaries_XYZ};

/// The basis conversion matrix for moving from linear sRGB space to XYZ colour
/// space.
///
/// To perform the conversion it’s typically more convenient to use the
/// xyz_from_linear() function instead of accessing this constant.
///
/// The matrix is built with the assumption that colours are represented as
/// one-column matrices.  With that, converting from sRGB to XYZ is done by the
/// following formula: `XYZ = XYZ_FROM_SRGB_MATRIX ✕ RGB`.
pub const XYZ_FROM_SRGB_MATRIX: [[f32; 3]; 3] = {matrix};

/// The basis conversion matrix for moving from XYZ to linear sRGB colour
/// space.
///
/// To perform the conversion it’s typically more convenient to use the
/// linear_from_xyz() function instead of accessing this constant.
///
/// The matrix is built with the assumption that colours are represented as
/// one-column matrices.  With that, converting from XYZ to sRGB is done by the
/// following formula: `RGB = SRGB_FROM_XYZ_MATRIX ✕ XYZ`.
pub const SRGB_FROM_XYZ_MATRIX: [[f32; 3]; 3] = {inverse};
",
            white_xyY = fmt_chromacity(&white_xy),
            white_XYZ = fmt_vector(&white_xyz),
            primaries_xyY = fmt_matrix(&primaries_xy, fmt_chromacity),
            primaries_XYZ = fmt_matrix(&primaries_xyz, fmt_vector),
            matrix = fmt_matrix(&matrix, fmt_vector),
            inverse = fmt_matrix(&inverse, fmt_vector)
        ),
    )?;

    let s0 = calc_gamma_threshold::<f64>();
    let e0 = gamma_compress_lin_part(&s0);

    write_to(
        &out_dir,
        "gamma_constants.rs",
        format_args!(
            r"// Generated by build.rs

const S_0: f32 = {:.};
const E_0: f32 = {:.};
",
            s0, e0
        ),
    )
}


fn main() -> std::io::Result<()> {
    generate()?;
    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}
