//! Functions implementing sRGB gamma compression and expansion formulæ.

// Defines S_0 and E_0 constants
include!(concat!(env!("OUT_DIR"), "/gamma_constants.rs"));

macro_rules! expand_impl {
    ($e:ident, $t:ty, $low:expr, $high:expr) => {{
        const FULL_RANGE: bool = $low == <$t>::MIN && $high == <$t>::MAX;
        const RANGE: f32 = ($high - $low) as f32;
        const THRESHOLD: $t = (E_0 * RANGE) as $t + $low;
        if !FULL_RANGE && $e <= $low {
            0.0
        } else if $e <= THRESHOLD {
            const D: f32 = 12.92 * RANGE;
            ($e - $low) as f32 / D
        } else if FULL_RANGE || $e < $high {
            const A: f32 = 0.055 * RANGE;
            const D: f32 = 1.055 * RANGE;
            ((($e - $low) as f32 + A) / D).powf(2.4)
        } else {
            1.0
        }
    }}
}

macro_rules! compress_impl {
    ($s:ident, $t:ty, $low:expr, $high:expr) => {{
        const RANGE: f32 = ($high - $low) as f32;
        // Adding 0.5 is for rounding.
        (if $s <= S_0 {
            const D: f32 = 12.92 * RANGE;
            crate::maths::mul_add($s.max(0.0), D, 0.5)
        } else {
            const A: f32 = 0.055 * RANGE;
            const D: f32 = 1.055 * RANGE;
            crate::maths::mul_add(D, $s.powf(1.0 / 2.4), -A + 0.5)
        }) as $t + $low
    }}
}

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
/// assert_eq!(0.0015176348, srgb::gamma::expand_u8(  5));
/// assert_eq!(0.046665084,  srgb::gamma::expand_u8( 61));
/// assert_eq!(0.8148465,    srgb::gamma::expand_u8(233));
/// assert_eq!(1.0,          srgb::gamma::expand_u8(255));
/// ```
#[inline]
pub fn expand_u8(e: u8) -> f32 {
    expand_impl!(e, u8, u8::MIN, u8::MAX)
}

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
    compress_impl!(s, u8, u8::MIN, u8::MAX)
}


/// Performs an sRGB gamma expansion on specified component value whose range is
/// [16, 235].
///
/// The value is clamped to the expected range.  The range corresponds to 8-bit
/// coding in Rec.709 standard.
///
/// # Example
///
/// ```
/// assert_eq!(0.0,          srgb::gamma::expand_rec709_8bit(  0));
/// assert_eq!(0.0,          srgb::gamma::expand_rec709_8bit( 16));
/// assert_eq!(0.0007068437, srgb::gamma::expand_rec709_8bit( 18));
/// assert_eq!(0.94884676,   srgb::gamma::expand_rec709_8bit(230));
/// assert_eq!(1.0,          srgb::gamma::expand_rec709_8bit(235));
/// assert_eq!(1.0,          srgb::gamma::expand_rec709_8bit(255));
///
/// let norm = srgb::rec709::decode_rec709_8bit(126);
/// assert_eq!(0.21616048, srgb::gamma::expand_rec709_8bit(126));
/// assert_eq!(0.21616042, srgb::gamma::expand_normalised(norm));
/// ```
#[inline]
pub fn expand_rec709_8bit(e: u8) -> f32 {
    expand_impl!(e, u8, 16, 235)
}

/// Performs an sRGB gamma compression on specified linear component and encodes
/// result as an integer in the [16, 235] range.
///
/// The value is clamped to the [0.0, 1.0] range.  The range of the result
/// corresponds to 8-bit coding in Rec.709 standard.
///
/// # Example
///
/// ```
/// assert_eq!( 16, srgb::gamma::compress_rec709_8bit(0.0));
/// assert_eq!( 18, srgb::gamma::compress_rec709_8bit(0.0007));
/// assert_eq!(230, srgb::gamma::compress_rec709_8bit(0.9488));
/// assert_eq!(235, srgb::gamma::compress_rec709_8bit(1.0));
///
/// assert_eq!(126, srgb::gamma::compress_rec709_8bit(0.21616048));
/// let norm = srgb::rec709::decode_rec709_8bit(126);
/// assert_eq!(0.21616042, srgb::gamma::expand_normalised(norm));
/// ```
#[inline]
pub fn compress_rec709_8bit(s: f32) -> u8 {
    compress_impl!(s, u8, 16, 235)
}

/// Performs an sRGB gamma expansion on specified component value whose range is
/// [64, 940].
///
/// The value is clamped to the expected range.  The range corresponds to 10-bit
/// coding in Rec.709 standard.
///
/// # Example
///
/// ```
/// assert_eq!(0.0,           srgb::gamma::expand_rec709_10bit(   0));
/// assert_eq!(0.0,           srgb::gamma::expand_rec709_10bit(  64));
/// assert_eq!(0.00053013273, srgb::gamma::expand_rec709_10bit(  70));
/// assert_eq!(0.67418975,    srgb::gamma::expand_rec709_10bit( 800));
/// assert_eq!(1.0,           srgb::gamma::expand_rec709_10bit( 940));
/// assert_eq!(1.0,           srgb::gamma::expand_rec709_10bit(1023));
///
/// let norm = srgb::rec709::decode_rec709_10bit(507);
/// assert_eq!(0.21936223, srgb::gamma::expand_rec709_10bit(507));
/// assert_eq!(0.21936223, srgb::gamma::expand_normalised(norm));
/// ```
#[inline]
pub fn expand_rec709_10bit(e: u16) -> f32 {
    expand_impl!(e, u16, 64, 940)
}

/// Performs an sRGB gamma compression on specified linear component and encodes
/// result as an integer in the [64, 940] range.
///
/// The value is clamped to the [0.0, 1.0] range.  The range of the result
/// corresponds to 10-bit coding in Rec.709 standard.
///
/// # Example
///
/// ```
/// assert_eq!(  64, srgb::gamma::compress_rec709_10bit(0.0));
/// assert_eq!(  70, srgb::gamma::compress_rec709_10bit(0.000530));
/// assert_eq!( 800, srgb::gamma::compress_rec709_10bit(0.674189));
/// assert_eq!( 940, srgb::gamma::compress_rec709_10bit(1.0));
///
/// assert_eq!(504, srgb::gamma::compress_rec709_10bit(0.21616048));
/// let norm = srgb::rec709::decode_rec709_10bit(504);
/// assert_eq!(0.21616042, srgb::gamma::expand_normalised(norm));
/// ```
#[inline]
pub fn compress_rec709_10bit(s: f32) -> u16 {
    compress_impl!(s, u16, 64, 940)
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
///     [0.8148465, 0.80695224, 0.7991027],
///     srgb::gamma::linear_from_u8([233, 232, 231])
/// );
/// assert_eq!(
///     [0.6583748, 0.015208514, 0.046665084],
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
