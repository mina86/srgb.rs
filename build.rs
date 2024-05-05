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

use std::io::Write;

use num::{One, Zero};

type Scalar = num::BigRational;
type Chromaticity = rgb_derivation::Chromaticity<Scalar>;


fn scalar(numer: i64, denom: i64) -> num::BigRational {
    num::BigRational::new(numer.into(), denom.into())
}

fn chromaticity(x: (i64, i64), y: (i64, i64)) -> Chromaticity {
    Chromaticity::new(scalar(x.0, x.1), scalar(y.0, y.1)).unwrap()
}

/// Formats scalar as a floating point number.  If denominator isn’t one, the
/// number is formatted as `n / d` string (where `n` and `d` are integers
/// written as floating point numbers); otherwise just the numerator is written.
fn fmt_scalar(scalar: &Scalar) -> String {
    let (numer, denom) = (scalar.numer(), scalar.denom());
    if numer.is_zero() || denom.is_one() {
        format!("{}.0", numer.to_str_radix(10))
    } else {
        format!("{}.0 / {}.0", numer.to_str_radix(10), denom.to_str_radix(10))
    }
}

fn fmt_vector(vec: &[Scalar; 3]) -> String {
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
) -> String {
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

fn fmt_chromaticity(ch: &Chromaticity) -> String {
    fmt_vector(&[ch.x().clone(), ch.y().clone(), One::one()])
}


fn gamma_compress_lin_part<T: num::traits::Float + num::traits::NumRef>(
    x: &T,
) -> T {
    T::from(12.92_f64).unwrap() * x
}

fn gamma_compress_pow_part<T: num::traits::Float + num::traits::NumRef>(
    x: &T,
) -> T {
    let exponent = T::from(5.0_f64 / 12.0_f64).unwrap();
    x.powf(exponent) * T::from(1.055).unwrap() - T::from(0.055).unwrap()
}

#[allow(clippy::op_ref)]
pub fn calc_gamma_threshold<
    T: PartialOrd + num::traits::Float + num::traits::NumRef,
>() -> T
where
    for<'r> &'r T: num::traits::RefNum<T>, {
    let mut lo = T::from(0.0030_f64).unwrap();
    let mut hi = T::from(0.0032_f64).unwrap();
    let mut mid = T::zero();
    for _ in 0..(std::mem::size_of::<T>() * 8) {
        mid = (lo + hi) / T::from(2_i16).unwrap();
        let lhs = gamma_compress_lin_part(&mid);
        let rhs = gamma_compress_pow_part(&mid);
        match lhs.partial_cmp(&rhs) {
            Some(std::cmp::Ordering::Less) => lo = mid,
            Some(std::cmp::Ordering::Greater) => hi = mid,
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

    let white_xy = chromaticity((312713, 1000000), (329016, 1000000));
    let primaries_xy = [
        chromaticity((64, 100), (33, 100)),
        chromaticity((30, 100), (60, 100)),
        chromaticity((15, 100), (6, 100)),
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
            white_xyY = fmt_chromaticity(&white_xy),
            white_XYZ = fmt_vector(&white_xyz),
            primaries_xyY = fmt_matrix(&primaries_xy, fmt_chromaticity),
            primaries_XYZ = fmt_matrix(&primaries_xyz, fmt_vector),
            matrix = fmt_matrix(&matrix, fmt_vector),
            inverse = fmt_matrix(&inverse, fmt_vector)
        ),
    )?;

    let s0 = calc_gamma_threshold::<f64>();
    let e0 = gamma_compress_lin_part(&s0);

    /* 512 bits of precision is a massive overkill but whatever, we don’t care
     * about speed and having too much precision won’t hurt. */
    let fl = |v| rug::Float::with_val(512, v);
    let u8_to_linear = (0..=255)
        .map(|v| {
            if v <= (e0 * 255.0) as u8 {
                fl(v as u32 * 10) / fl(32946)
            } else {
                let v = fl(v as u32 * 1_000 + 55 * 255) / fl(1055u32 * 255);
                let e = fl(24) / fl(10);
                rug::ops::Pow::pow(v, e)
            }
        })
        .map(|v| {
            /* Make sure zero is encoded as `0.0` so it’s parsed as a floating
             * point number and not integer.  Normally, to_str_radix() does not
             * include the decimal separator when formatting zero. */
            let v = v.to_string_radix(10, Some(24));
            format!("    {},\n", if v == "0" { &"0.0" } else { &v[..] })
        })
        .collect::<Vec<_>>()
        .join("");

    write_to(
        &out_dir,
        "gamma_constants.rs",
        format_args!(
            r"// Generated by build.rs

/// The threshold at which sRGB gamma compression switches from linear to power
/// function.
///
/// While many RGB colour models use a simple power function as their gamma
/// correction step, sRGB standard uses a function which consists of two parts:
/// linear at the beginning and a power function afterwards.  This constant is
/// the value at which the gamma compression switches between the two regimes.
/// In theory it’s also an argument at which both parts produce the same result
/// though that’s subject to floating-point rounding.
pub const S_0: f32 = {:.};

/// The threshold at which sRGB gamma expansion switches from linear to power
/// function.
///
/// While many RGB colour models use a simple power function as their gamma
/// correction step, sRGB standard uses a function which consists of two parts:
/// linear at the beginning and a power function afterwards.  This constant is
/// the value at which the gamma expansion switches between the two regimes.
/// In theory it’s also an argument at which both parts produce the same result
/// though that’s subject to floating-point rounding.
pub const E_0: f32 = {:.};

const U8_TO_LINEAR_LUT: [f32; 256] = [
{}
];
",
            s0, e0, u8_to_linear
        ),
    )
}


fn main() -> std::io::Result<()> {
    generate()?;
    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}
