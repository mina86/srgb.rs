//! Functions implementing sRGB gamma compression and expansion formulæ.

/// Performs an sRGB gamma expansion on specified 8-bit component value.  In
/// other words, converts an 8-bit sRGB component value into a linear sRGB
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
/// use srgb::gamma::expand_u8;
///
/// let white = [expand_u8(233), expand_u8(232), expand_u8(231)];
/// assert_eq!([0.8148465, 0.80695224, 0.7991027], white);
///
/// let red = [expand_u8(212), expand_u8(33), expand_u8(61)];
/// assert_eq!([0.6583748, 0.015208514, 0.046665084], red);
/// ```
#[inline]
pub fn expand_u8(e: u8) -> f32 {
    if e > 10 {
        const A: f32 = 0.055 * 255.0;
        const D: f32 = 1.055 * 255.0;
        ((e as f32 + A) / D).powf(2.4)
    } else {
        const D: f32 = 12.92 * 255.0;
        e as f32 / D
    }
}

/// Performs an sRGB gamma compression on specified linear component value.  In
/// other words, converts a linear sRGB component into an 8-bit sRGB value.  The
/// argument must be in the range from zero to one.  The result will be in the
/// range 0–255 range.
///
/// This function is faster (and slightly more accurate as it performs fewer
/// floating point operations) than first compressing into a normalised value
/// and then converting that to an 8-bit value.
///
/// # Example
///
/// ```
/// use srgb::gamma::compress_u8;
///
/// let white = [0.8148465, 0.80695224, 0.7991027];
/// let white =
///     [compress_u8(white[0]), compress_u8(white[1]), compress_u8(white[2])];
/// assert_eq!([233, 232, 231], white);
///
/// let red = [0.6583748, 0.015208514, 0.046665084];
/// let red = [compress_u8(red[0]), compress_u8(red[1]), compress_u8(red[2])];
/// assert_eq!([212, 33, 61], red);
/// ```
#[inline]
pub fn compress_u8(s: f32) -> u8 {
    let e = if s > 0.0031308 {
        const A: f32 = 0.055 * 255.0;
        const D: f32 = 1.055 * 255.0;
        crate::maths::mul_add(D, s.powf(1.0 / 2.4), -A)
    } else {
        const D: f32 = 12.92 * 255.0;
        D * s
    };
    // Adding 0.5 is for rounding.
    (e.clamp(0.0, 255.0) + 0.5) as u8
}


/// Performs an sRGB gamma expansion on specified normalised component value.
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
/// use srgb::gamma::expand_normalised;
///
/// let mut white = [0.91372544, 0.90980387, 0.9058823];
/// white.iter_mut().for_each(|c| *c = expand_normalised(*c));
/// assert_eq!([0.8148465, 0.80695224, 0.7991027], white);
///
/// let mut red = [0.8313725, 0.12941176, 0.23921567];
/// red.iter_mut().for_each(|c| *c = expand_normalised(*c));
/// assert_eq!([0.6583748, 0.015208514, 0.046665084], red);
/// ```
#[inline]
pub fn expand_normalised(e: f32) -> f32 {
    const E_0: f32 = 12.92 * 0.00313066844250060782371;
    if e > E_0 {
        ((e as f32 + 0.055) / 1.055).powf(2.4)
    } else {
        e / 12.92
    }
}

/// Performs an sRGB gamma compression on specified linear component value.  In
/// other words, converts a linear sRGB component into a normalised sRGB value.
/// The argument must be in the range from zero to one.  The result will be in
/// the same range.
///
/// Prefer [`compress_u8()`] if you’re intending to end up with an 8-bit colour
/// in 0–255 range.
///
/// # Example
///
/// ```
/// use srgb::gamma::compress_normalised;
///
/// let mut white = [0.8148465, 0.80695224, 0.7991027];
/// white.iter_mut().for_each(|c: &mut f32| *c = compress_normalised(*c));
/// assert_eq!([0.91372544, 0.90980387, 0.9058823], white);
///
/// let mut red = [0.6583748, 0.015208514, 0.046665084];
/// red.iter_mut().for_each(|c| *c = compress_normalised(*c));
/// assert_eq!([0.8313725, 0.12941176, 0.23921567], red);
/// ```
#[inline]
pub fn compress_normalised(s: f32) -> f32 {
    if s > 0.0031308 {
        crate::maths::mul_add(1.055, s.powf(1.0 / 2.4), -0.055)
    } else {
        12.92 * s
    }
}


#[cfg(test)]
mod test {
    use approx::assert_ulps_eq;

    const CASES: [(f32, u8); 8] = [
        (0.0, 0),
        (0.051269453, 64),
        (0.18116423, 118),
        (0.21586047, 128),
        (0.5028864, 188),
        (0.52711517, 192),
        (0.74540424, 224),
        (1.0, 255),
    ];

    #[test]
    fn test_expand_u8() {
        for (s, e) in CASES.iter().copied() {
            assert_eq!(s, super::expand_u8(e));
        }
    }

    #[test]
    fn test_compress_u8() {
        for (s, e) in CASES.iter().copied() {
            assert_eq!(e, super::compress_u8(s));
        }
    }

    #[test]
    fn test_expand_normalised() {
        for (s, e) in CASES.iter().copied() {
            assert_ulps_eq!(
                s,
                super::expand_normalised(e as f32 / 255.0),
                max_ulps = 3
            );
        }
    }

    #[test]
    fn test_compress_normalised() {
        for (s, e) in CASES.iter().copied() {
            assert_ulps_eq!(e as f32 / 255.0, super::compress_normalised(s));
        }
    }
}
