//! Functions implementing sRGB gamma compression and expansion formulæ.

// Defines S_0 and E_0 constants
include!(concat!(env!("OUT_DIR"), "/gamma_constants.rs"));

/// Performs an sRGB gamma expansion on specified 8-bit component value.
///
/// In other words, converts an 8-bit sRGB component value into a linear sRGB
/// value.  The argument must be in the range 0–255.  The result will be in the
/// range from zero to one.
///
/// This function is faster (and slightly more accurate as it performs fewer
/// floating point operations) than first normalising the 8-bit value and then
/// expanding that normalised value.
///
/// # Example
///
/// ```
/// assert_eq!(0.0,          srgb::gamma::expand_u8(  0));
/// assert_eq!(0.001517635,  srgb::gamma::expand_u8(  5));
/// assert_eq!(0.046665087,  srgb::gamma::expand_u8( 61));
/// assert_eq!(0.8148466,    srgb::gamma::expand_u8(233));
/// assert_eq!(1.0,          srgb::gamma::expand_u8(255));
/// ```
#[inline]
pub fn expand_u8(e: u8) -> f32 { U8_TO_LINEAR_LUT[e as usize] }

/// Performs an sRGB gamma compression on specified linear component value.
///
/// In other words, converts a linear sRGB component into an 8-bit sRGB value.
/// The argument must be in the range from zero to one.  The result will be in
/// the range 0–255 range.
///
/// This function is faster (and slightly more accurate as it performs fewer
/// floating point operations) than first compressing into a normalised value
/// and then converting that to an 8-bit value.
///
/// # Example
///
/// ```
/// assert_eq!(  0, srgb::gamma::compress_u8(0.0));
/// assert_eq!(  5, srgb::gamma::compress_u8(0.0015176348));
/// assert_eq!( 61, srgb::gamma::compress_u8(0.046665084));
/// assert_eq!(233, srgb::gamma::compress_u8(0.8148465));
/// assert_eq!(255, srgb::gamma::compress_u8(1.0));
/// ```
#[inline]
pub fn compress_u8(s: f32) -> u8 {
    // Adding 0.5 is for rounding.
    (if s <= S_0 {
        const D: f32 = 12.92 * 255.0;
        crate::maths::mul_add(s.max(0.0), D, 0.5)
    } else {
        const A: f32 = 0.055 * 255.0;
        const D: f32 = 1.055 * 255.0;
        crate::maths::mul_add(D, s.powf(5.0 / 12.0), -A + 0.5)
    }) as u8
}

macro_rules! compress_rec709_impl {
    ($s:ident, $t:ty, $low:expr, $high:expr) => {{
        const RANGE: f32 = ($high - $low) as f32;
        // Adding 0.5 is for rounding.
        (if $s <= 0.018 {
            const D: f32 = 4.5 * RANGE;
            crate::maths::mul_add($s.max(0.0), D, 0.5)
        } else {
            const A: f32 = 0.099 * RANGE;
            const D: f32 = 1.099 * RANGE;
            crate::maths::mul_add(D, $s.powf(1.0 / 2.2), -A + 0.5)
        }) as $t +
            $low
    }};
}

macro_rules! expand_rec709_impl {
    ($e:ident, $t:ty, $low:expr, $high:expr) => {{
        const RANGE: f32 = ($high - $low) as f32;
        const THRESHOLD: $t = (4.5 * 0.018 * RANGE) as $t + $low;
        if $e <= $low {
            0.0
        } else if $e <= THRESHOLD {
            const D: f32 = 4.5 * RANGE;
            ($e - $low) as f32 / D
        } else if $e < $high {
            const A: f32 = 0.099 * RANGE;
            const D: f32 = 1.099 * RANGE;
            ((($e - $low) as f32 + A) / D).powf(2.2)
        } else {
            1.0
        }
    }};
}

/// Performs an Rec.709 gamma expansion on specified component value whose range
/// is [16, 235].
///
/// The value is clamped to the expected range.  The range corresponds to 8-bit
/// coding in Rec.709 standard.  Note that Rec.709 transfer function is
/// different from sRGB transfer function (even though both standards use the
/// same primaries and white point).
///
/// # Example
///
/// ```
/// assert_eq!(0.0,          srgb::gamma::expand_rec709_8bit(  0));
/// assert_eq!(0.0,          srgb::gamma::expand_rec709_8bit( 16));
/// assert_eq!(0.0020294266, srgb::gamma::expand_rec709_8bit( 18));
/// assert_eq!(0.9548653,    srgb::gamma::expand_rec709_8bit(230));
/// assert_eq!(1.0,          srgb::gamma::expand_rec709_8bit(235));
/// assert_eq!(1.0,          srgb::gamma::expand_rec709_8bit(255));
/// ```
#[inline]
pub fn expand_rec709_8bit(e: u8) -> f32 { expand_rec709_impl!(e, u8, 16, 235) }

/// Performs an sRGB gamma compression on specified linear component and encodes
/// result as an integer in the [16, 235] range.
///
/// The value is clamped to the [0.0, 1.0] range.  The range of the result
/// corresponds to 8-bit coding in Rec.709 standard.  Note that Rec.709 transfer
/// function is different from sRGB transfer function (even though both
/// standards use the same primaries and white point).
///
/// # Example
///
/// ```
/// assert_eq!( 16, srgb::gamma::compress_rec709_8bit(0.0));
/// assert_eq!( 18, srgb::gamma::compress_rec709_8bit(0.002));
/// assert_eq!(230, srgb::gamma::compress_rec709_8bit(0.954));
/// assert_eq!(235, srgb::gamma::compress_rec709_8bit(1.0));
/// ```
#[inline]
pub fn compress_rec709_8bit(s: f32) -> u8 {
    compress_rec709_impl!(s, u8, 16, 235)
}

/// Performs an Rec.709 gamma expansion on specified component value whose range
/// is [64, 940].
///
/// The value is clamped to the expected range.  The range corresponds to 10-bit
/// coding in Rec.709 standard.  Note that Rec.709 transfer function is
/// different from sRGB transfer function (even though both standards use the
/// same primaries and white point).
///
/// # Example
///
/// ```
/// assert_eq!(0.0,           srgb::gamma::expand_rec709_10bit(   0));
/// assert_eq!(0.0,           srgb::gamma::expand_rec709_10bit(  64));
/// assert_eq!(0.00152207,    srgb::gamma::expand_rec709_10bit(  70));
/// assert_eq!(0.7077097,     srgb::gamma::expand_rec709_10bit( 800));
/// assert_eq!(1.0,           srgb::gamma::expand_rec709_10bit( 940));
/// assert_eq!(1.0,           srgb::gamma::expand_rec709_10bit(1023));
/// ```
#[inline]
pub fn expand_rec709_10bit(e: u16) -> f32 {
    expand_rec709_impl!(e, u16, 64, 940)
}

/// Performs an Rec.709 gamma compression on specified linear component and
/// encodes result as an integer in the [64, 940] range.
///
/// The value is clamped to the [0.0, 1.0] range.  The range of the result
/// corresponds to 10-bit coding in Rec.709 standard.  Note that Rec.709
/// transfer function is different from sRGB transfer function (even though both
/// standards use the same primaries and white point).
///
/// # Example
///
/// ```
/// assert_eq!(  64, srgb::gamma::compress_rec709_10bit(0.0));
/// assert_eq!(  70, srgb::gamma::compress_rec709_10bit(0.0015));
/// assert_eq!( 800, srgb::gamma::compress_rec709_10bit(0.7077));
/// assert_eq!( 940, srgb::gamma::compress_rec709_10bit(1.0));
/// ```
#[inline]
pub fn compress_rec709_10bit(s: f32) -> u16 {
    compress_rec709_impl!(s, u16, 64, 940)
}


/// Performs an sRGB gamma expansion on specified normalised component value.
///
/// In other words, converts a normalised sRGB component value into a linear
/// sRGB value.  The argument must be in the range from zero to one.  The result
/// will be in the same range.
///
/// Prefer [`expand_u8()`] if you’re starting with an 8-bit colour in 0–255
/// range.
///
/// # Example
///
/// ```
/// assert_eq!(0.0,         srgb::gamma::expand_normalised(0.0));
/// assert_eq!(0.046665084, srgb::gamma::expand_normalised(0.23921567));
/// assert_eq!(0.8148465,   srgb::gamma::expand_normalised(0.91372544));
/// assert_eq!(1.0,         srgb::gamma::expand_normalised(1.0));
/// ```
#[inline]
pub fn expand_normalised(e: f32) -> f32 {
    if e <= E_0 {
        e / 12.92
    } else {
        ((e as f32 + 0.055) / 1.055).powf(2.4)
    }
}

/// Performs an sRGB gamma compression on specified linear component value.
///
/// In other words, converts a linear sRGB component into a normalised sRGB
/// value.  The argument must be in the range from zero to one.  The result will
/// be in the same range.
///
/// Prefer [`compress_u8()`] if you’re intending to end up with an 8-bit colour
/// in 0–255 range.
///
/// # Example
///
/// ```
/// assert_eq!(0.0,        srgb::gamma::compress_normalised(0.0));
/// assert_eq!(0.23921569, srgb::gamma::compress_normalised(0.046665084));
/// assert_eq!(0.91372544, srgb::gamma::compress_normalised(0.8148465));
/// // Unfortunately, imprecision of floating point numbers may be an issue:
/// assert_eq!(0.99999994, srgb::gamma::compress_normalised(1.0));
/// ```
#[inline]
pub fn compress_normalised(s: f32) -> f32 {
    if s <= S_0 {
        12.92 * s
    } else {
        crate::maths::mul_add(1.055, s.powf(1.0 / 2.4), -0.055)
    }
}


/// Converts a 24-bit sRGB colour (also known as true colour) into linear space.
///
/// That is, performs gamma expansion on each component and returns the colour
/// in linear sRGB space with each component normalised to the range 0–1.
///
/// This is just a convenience wrapper around [`expand_u8()`] function.
///
/// # Example
/// ```
/// assert_eq!(
///     [0.8148466, 0.80695224, 0.7991027],
///     srgb::gamma::linear_from_u8([233, 232, 231])
/// );
/// assert_eq!(
///     [0.65837485, 0.015208514, 0.046665087],
///     srgb::gamma::linear_from_u8([212, 33, 61])
/// );
/// ```
#[inline]
pub fn linear_from_u8(encoded: [u8; 3]) -> [f32; 3] {
    super::arr_map(encoded, expand_u8)
}

/// Converts an sRGB colour in linear space to a 24-bit sRGB colour (also known
/// as true colour).
///
/// That is, performs gamma compression on each component and encodes each
/// component as an 8-bit integer.
///
/// This is just a convenience wrapper around [`compress_u8()`] function.
///
/// # Example
/// ```
/// assert_eq!(
///     [233, 232, 231],
///     srgb::gamma::u8_from_linear([0.8148465, 0.80695224, 0.7991027])
/// );
/// assert_eq!(
///     [212, 33, 61],
///     srgb::gamma::u8_from_linear([0.6583748, 0.015208514, 0.046665084])
/// );
/// ```
#[inline]
pub fn u8_from_linear(linear: [f32; 3]) -> [u8; 3] {
    super::arr_map(linear, compress_u8)
}


/// Converts an sRGB colour in normalised representation into linear space.
///
/// That is, performs gamma expansion on each component (which should be in 0–1
/// range) and returns the colour in linear space.
///
/// This is just a convenience wrapper around [`expand_normalised()`]
/// function.
///
/// # Example
/// ```
/// assert_eq!(
///     [0.8148467, 0.80695236, 0.79910284],
///     srgb::gamma::linear_from_normalised([0.9137255, 0.9098039, 0.90588236])
/// );
/// assert_eq!(
///     [0.65837485, 0.015208514, 0.046665095],
///     srgb::gamma::linear_from_normalised([0.83137256, 0.12941177, 0.2392157])
/// );
/// ```
#[inline]
pub fn linear_from_normalised(normalised: [f32; 3]) -> [f32; 3] {
    super::arr_map(normalised, expand_normalised)
}

/// Converts an sRGB colour in linear space to normalised space.
///
/// That is, performs gamma compression on each component (which should be in
/// 0–1 range) and encodes each component as an 8-bit integer.
///
/// This is just a convenience wrapper around [`compress_normalised()`]
/// function.
///
/// # Example
/// ```
/// assert_eq!(
///     [0.9137255, 0.9098039, 0.90588236],
///     srgb::gamma::normalised_from_linear([0.8148467, 0.80695236, 0.79910284])
/// );
/// assert_eq!(
///     [0.83137256, 0.12941168, 0.23921566],
///     srgb::gamma::normalised_from_linear([0.65837485, 0.0152085, 0.04666508])
/// );
/// ```
#[inline]
pub fn normalised_from_linear(linear: [f32; 3]) -> [f32; 3] {
    super::arr_map(linear, compress_normalised)
}


#[cfg(test)]
mod test {
    use approx::assert_ulps_eq;

    use super::*;

    const CASES: [(f32, u8); 12] = [
        (0.0, 0),
        (0.001517635, 5),
        (0.003035270, 10),
        (0.014443844, 32),
        (0.051269458, 64),
        (0.181164244, 118),
        (0.215860500, 128),
        (0.351532599, 160),
        (0.502886458, 188),
        (0.527115125, 192),
        (0.745404209, 224),
        (1.0, 255),
    ];

    #[test]
    fn test_expand_u8() {
        for (s, e) in CASES.iter().copied() {
            assert_eq!(s, expand_u8(e));
        }
    }

    #[test]
    fn test_compress_u8() {
        for (s, e) in CASES.iter().copied() {
            assert_eq!(e, compress_u8(s));
        }
    }

    #[test]
    fn test_expand_normalised() {
        for (s, e) in CASES.iter().copied() {
            assert_ulps_eq!(
                s,
                expand_normalised(e as f32 / 255.0),
                max_ulps = 5
            );
        }
    }

    #[test]
    fn test_compress_normalised() {
        for (s, e) in CASES.iter().copied() {
            assert_ulps_eq!(e as f32 / 255.0, compress_normalised(s));
        }
    }

    fn run_round_trip_test(
        min: u16,
        max: u16,
        to_lin: impl Fn(u16) -> f32,
        from_lin: impl Fn(f32) -> u16,
    ) {
        for v in min..=max {
            let lin = to_lin(v);
            let got = from_lin(lin);
            assert_eq!((v, lin), (got, lin));
        }
    }

    #[test]
    fn test_round_trip_u8() {
        run_round_trip_test(
            0,
            255,
            |v| expand_u8(v as u8),
            |v| compress_u8(v) as u16,
        );
    }

    #[test]
    fn test_round_trip_rec709_8bit() {
        run_round_trip_test(
            16,
            235,
            |v| expand_rec709_8bit(v as u8),
            |v| compress_rec709_8bit(v) as u16,
        );
    }

    #[test]
    fn test_round_trip_rec709_10bit() {
        run_round_trip_test(
            64,
            940,
            expand_rec709_10bit,
            compress_rec709_10bit,
        );
    }

    #[test]
    fn test_round_trip_normalised() {
        for i in 0..=1000 {
            let want = i as f32 / 1000.0;
            let got = compress_normalised(expand_normalised(want));
            assert_ulps_eq!(want, got);
        }
    }

    #[test]
    fn test_round_trip_error() {
        let mut error_ec = kahan::KahanSum::new();
        let mut error_ce = kahan::KahanSum::new();
        for i in 0..=1000 {
            let want = i as f32 / 1000.0;
            let diff = want as f64 -
                compress_normalised(expand_normalised(want)) as f64;
            error_ec += diff * diff;
            let diff = want as f64 -
                expand_normalised(compress_normalised(want)) as f64;
            error_ce += diff * diff;
        }

        assert_eq!(
            (0.43569314822633487, 3.0850596057820088),
            (error_ec.sum() * 1e12, error_ce.sum() * 1e12)
        );
    }
}
