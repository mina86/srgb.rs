/* This file is part of srgb crate.
 * Copyright 2022 by Michał Nazarewicz <mina86@mina86.com>
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
#![allow(clippy::neg_cmp_op_on_partial_ord)]

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
/// the 0–255 range.
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
///
/// # Approximation
///
/// Since version 0.3, this function uses an approximated formula.  Importantly,
/// it has over 14 bits of precision which means that it’s sufficient for all
/// practical applications.  Furthermore, it is an inverse of [`expand_u8()`] so
/// for any integer `n` the comparison `n == compress_u8(expand_u8(n))` holds.
///
/// Since the function returns an 8-bit integer talking about precision or error
/// of the output doesn’t make much sense.  Instead, precision of the
/// approximation was measured comparing functions which return the highest
/// value which maps to given integer.
///
/// That is, two functions were constructed `f(n)` and `g(n)` such that:
///
/// 1. `compress_u8_precise(f(n)) == n`,
/// 2. if `x > f(n)` than `compress_u8_precise(x) > n`,
/// 3. `compress_u8(g(n)) == n` and
/// 4. if `x > g(n)` than `compress_u8(x) > n`.
///
/// With those, a regular statistics were used to test how well `g(n)`
/// approximates `f(n)`.  The results were as follows:
///
/// * Maximum absolute error: 0.85 * 2<sup>-14</sup> <small>(14.24 bits of
///   precision)</small>
/// * Average absolute error: 0.27 * 2<sup>-14</sup>
/// * Root mean squared error: 0.35 * 2<sup>-14</sup>
///
/// See [`compress_u8_precise()`] function for version of the function which
/// uses exact sRGB gamma formula (but is over 2.5 slower).
///
/// PS. This function’s performance is similar to that of `f32_to_srgb8` in the
/// `fast-srgb8` crate while at the same time it offers around 1.8 bits more
/// precision as measured with the above method.
#[inline]
pub fn compress_u8(s: f32) -> u8 {
    // Note: Using negated comparison to also catch NaNs.
    if !(s > FAST_START_AT) {
        const D: f32 = 12.92 * 255.0;
        D.mul_add(s.max(0.0), 0.5) as u8
    } else if s < FAST_START_255_AT {
        /* Would like to do those asserts but f32::to_bits is not a const fn.

        // Make sure x.to_bits() - FAST_BITS_OFFSET is not negative.
        const _COND1: bool = FAST_START_AT.to_bits() >= FAST_BITS_OFFSET;
        const _: [(); 1] = [(); _COND1 as usize];

        // Make sure that LUT contains enough entries.
        const _START: u32 = FAST_BITS_OFFSET;
        const _END: u32 = FAST_START_255_AT.to_bits();
        const _LUT_LEN: u32 = ((_END - _START) >> FAST_SHIFT) + 1;
        const _COND2: bool = (_LUT_LEN as usize) == FAST_LUT.len();
        const _: [(); 1] = [(); _COND2 as usize];

        */

        let bits = s.to_bits() - FAST_BITS_OFFSET;
        let lft_x = (bits >> FAST_SHIFT) as usize;
        let rht_x = lft_x + 1;

        debug_assert!(rht_x < FAST_LUT.len());
        let lft = unsafe { FAST_LUT.get_unchecked(lft_x) };
        let rht = unsafe { FAST_LUT.get_unchecked(rht_x) };

        let lft_x =
            f32::from_bits(FAST_BITS_OFFSET + ((lft_x as u32) << FAST_SHIFT));
        let rht_x =
            f32::from_bits(FAST_BITS_OFFSET + ((rht_x as u32) << FAST_SHIFT));

        let dx = rht_x - lft_x;
        let ox = s - lft_x;

        (lft + (rht - lft) * ox / dx) as u8
    } else {
        255
    }
}

/// Performs an sRGB gamma compression on specified linear component value.
///
/// In other words, converts a linear sRGB component into an 8-bit sRGB value.
/// The argument must be in the range from zero to one.  The result will be in
/// the range 0–255 range.
///
/// Unlike [`compress_u8()`] function, this function uses exact sRGB gamma
/// formula and as a result is over 2.5 times slower.
///
/// # Example
///
/// ```
/// assert_eq!(  0, srgb::gamma::compress_u8_precise(0.0));
/// assert_eq!(  5, srgb::gamma::compress_u8_precise(0.0015176348));
/// assert_eq!( 61, srgb::gamma::compress_u8_precise(0.046665084));
/// assert_eq!(233, srgb::gamma::compress_u8_precise(0.8148465));
/// assert_eq!(255, srgb::gamma::compress_u8_precise(1.0));
/// ```
#[inline]
pub fn compress_u8_precise(s: f32) -> u8 {
    // Adding 0.5 is for rounding.  Negated comparison is to catch NaNs.
    (if !(s > S_0) {
        const D: f32 = 12.92 * 255.0;
        crate::maths::mul_add(s.max(0.0), D, 0.5)
    } else {
        const A: f32 = 0.055 * 255.0;
        const D: f32 = 1.055 * 255.0;
        crate::maths::mul_add(D, s.min(1.0).powf(5.0 / 12.0), -A + 0.5)
    }) as u8
}

/// Value at which [`compress_u8`] will start using the approximation.
/// Below that value the linear piece of sRGB gamma compression formula is used.
const FAST_START_AT: f32 = 0.0031919535067975154;

/// Value at which [`compress_u8`] will start returning 255.
const FAST_START_255_AT: f32 = 0.9954979522975671;

/// Value to subtracted from [`compress_u8`] argument when calculating
/// LUT index.
const FAST_BITS_OFFSET: u32 = 994926221;

/// Shift used for [`compress_u8`] argument when calculating LUT index.
const FAST_SHIFT: usize = 19;

/// LUT used by [`compress_u8`].
const FAST_LUT: [f32; 136] = [
    10.842953690763022,
    11.247256975805167,
    11.596920589218305,
    11.974676118377491,
    12.347806123119943,
    12.72714106745028,
    13.094275703436033,
    13.63805819726478,
    14.292937686709234,
    14.923680771165712,
    15.570126974440122,
    16.209820658051132,
    16.76447039487231,
    17.337253305518896,
    17.896354278599194,
    18.475192108100988,
    19.002640297704545,
    19.522935134467726,
    20.041242037921712,
    20.44552429210586,
    21.00679130121049,
    21.47038533959708,
    21.95723744906331,
    22.740790057148146,
    23.629638990247546,
    24.528352342327914,
    25.344989332552313,
    26.150728850828973,
    26.931743687393734,
    27.69293355977654,
    28.45446820605869,
    29.163328918736447,
    29.88016655621456,
    30.541704620580504,
    31.283955072170404,
    31.909339744941736,
    32.57861591749353,
    33.22128809353077,
    33.860388301927244,
    34.86071045271506,
    36.05023219119529,
    37.19627012057453,
    38.33638181546821,
    39.40630933386438,
    40.480161375261915,
    41.50097043065091,
    42.48497905447285,
    43.44071136587259,
    44.416934554423975,
    45.3332498678882,
    46.24404859335657,
    47.115083409448665,
    48.000760775816545,
    48.86182846076076,
    49.68975696485138,
    51.05894889111046,
    52.65192977911797,
    54.18507435324061,
    55.67581775800756,
    57.1224420380186,
    58.526096385269604,
    59.882603744104806,
    61.22115631317263,
    62.52368891793067,
    63.785800901727924,
    65.03864110345859,
    66.23138631030096,
    67.43407436283562,
    68.58192908335137,
    69.73662278145312,
    70.85981071029647,
    72.691567255648,
    74.80948140598991,
    76.85585905387728,
    78.8411048501123,
    80.76906363534704,
    82.64367220796105,
    84.4680915856258,
    86.24182355126462,
    87.98035912297826,
    89.67241895226687,
    91.32473005724265,
    92.94833612001699,
    94.53132666465619,
    96.0886285884802,
    97.59136088906574,
    99.10827936108777,
    101.55122267350802,
    104.37534049860218,
    107.10956846611478,
    109.76245842045016,
    112.33487273840012,
    114.83812019182777,
    117.27118243546161,
    119.6465610745734,
    121.96191517878458,
    124.22149067968998,
    126.43203722300153,
    128.59477860091027,
    130.71537331816762,
    132.79010699255343,
    134.8231304635986,
    136.81839992088112,
    140.07622800979328,
    143.85020337902137,
    147.50467279470942,
    151.03984651898168,
    154.4785449067233,
    157.8141534496148,
    161.06490567147762,
    164.23388376212907,
    167.32565362583685,
    170.34319248466915,
    173.29325733995148,
    176.1804936543506,
    179.00691559527124,
    181.77602379414793,
    184.4917506799657,
    187.15869953564714,
    191.50921533898074,
    196.54895886439573,
    201.41755025831992,
    206.1404862741362,
    210.72419738558636,
    215.18208554701212,
    219.52146542097762,
    223.75061196813897,
    227.87642551572122,
    231.90572013604708,
    235.8446551234182,
    239.69838617873333,
    243.4705437011741,
    247.16989886510626,
    250.79526125436902,
    254.35313489067892,
    260.01981786784313,
];


macro_rules! compress_rec709_impl {
    ($s:ident, $t:ty, $low:expr, $high:expr) => {{
        const RANGE: f32 = ($high - $low) as f32;
        // Adding 0.5 is for rounding.  Negated comparison is to catch NaNs.
        (if !($s > 0.018) {
            const D: f32 = 4.5 * RANGE;
            crate::maths::mul_add($s.max(0.0), D, 0.5)
        } else {
            const A: f32 = 0.099 * RANGE;
            const D: f32 = 1.099 * RANGE;
            crate::maths::mul_add(D, $s.min(1.0).powf(1.0 / 2.2), -A + 0.5)
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
    // Note: Using negated comparison to also catch NaNs.
    if !(e > E_0) {
        e / 12.92
    } else {
        ((e + 0.055) / 1.055).powf(2.4)
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
/// assert_eq!(0.23921567, srgb::gamma::compress_normalised(0.046665084));
/// assert_eq!(0.91372544, srgb::gamma::compress_normalised(0.8148465));
/// // Unfortunately, imprecision of floating point numbers may be an issue:
/// assert_eq!(0.99999994, srgb::gamma::compress_normalised(1.0));
/// ```
#[inline]
pub fn compress_normalised(s: f32) -> f32 {
    // Note: Using negated comparison to also catch NaNs.
    if !(s > S_0) {
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
pub fn linear_from_u8(encoded: impl std::convert::Into<[u8; 3]>) -> [f32; 3] {
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
pub fn u8_from_linear(linear: impl std::convert::Into<[f32; 3]>) -> [u8; 3] {
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
pub fn linear_from_normalised(
    normalised: impl std::convert::Into<[f32; 3]>,
) -> [f32; 3] {
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
///     [0.83137256, 0.1294117, 0.23921564],
///     srgb::gamma::normalised_from_linear([0.65837485, 0.0152085, 0.04666508])
/// );
/// ```
#[inline]
pub fn normalised_from_linear(
    linear: impl std::convert::Into<[f32; 3]>,
) -> [f32; 3] {
    super::arr_map(linear, compress_normalised)
}


#[cfg(test)]
mod test {
    use approx::assert_ulps_eq;
    use float_next_after::NextAfter;

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
    fn test_compress_u8_precise() {
        for (s, e) in CASES.iter().copied() {
            assert_eq!(e, compress_u8_precise(s));
        }
    }

    #[test]
    fn test_compress_u8() {
        for e in 0..=255 {
            assert_eq!(e, compress_u8(expand_u8(e)));
        }
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
    fn test_round_trip_u8_precise() {
        run_round_trip_test(
            0,
            255,
            |v| expand_u8(v as u8),
            |v| compress_u8_precise(v) as u16,
        );
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
    fn test_rec709_scaling() {
        for v in 16..=235 {
            let expanded = expand_rec709_8bit(v);
            assert_eq!(expanded, expand_rec709_10bit(v as u16 * 4));
            assert_eq!(
                compress_rec709_8bit(expanded) as u16 * 4,
                compress_rec709_10bit(expanded)
            );
        }
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
            (0.24183433033897472, 2.3217916846930695),
            (error_ec.sum() * 1e12, error_ce.sum() * 1e12)
        );
    }

    #[test]
    fn test_compress_u8_increases() {
        // Starting at 0.0 makes this test dramatically slower so skip the first
        // few values.
        let mut value = 0.0001;
        let mut prev = compress_u8(value);
        assert_eq!(0, prev, "Didn’t start at zero");
        while value < 1.0 {
            let next = value.next_after(std::f32::INFINITY);
            let res = compress_u8(next);
            assert!(
                prev <= res,
                "{} = f({}) > f({}) = {}",
                prev,
                value,
                next,
                res
            );
            assert!(
                res - prev <= 1,
                "f({}) - f({}) = {} - {} > 1",
                next,
                value,
                res,
                prev
            );
            value = next;
            prev = res;
        }
        assert_eq!(255, prev, "Didn’t reach 255");
    }

    #[test]
    fn test_compress_u8_statistics() {
        fn edges(compress: fn(f32) -> u8) -> [f32; 255] {
            let mut edges = [0.0; 255];
            let mut x = 0.0001;
            while compress(x) != 0 {
                x *= 0.5;
                assert_ne!(x, 0.0);
            }
            edges[0] = x;
            loop {
                x = x.next_after(std::f32::INFINITY);
                assert!(x < 1.0);
                let y = compress(x);
                if y == 255 {
                    break edges;
                }
                edges[y as usize] = x;
            }
        }

        let want = edges(compress_u8_precise);
        let got = edges(compress_u8);

        let mut max_abs_error = 0.0;
        let mut abs_error = kahan::KahanSum::new();
        let mut squared_error = kahan::KahanSum::new();
        for (a, b) in want.iter().zip(got.iter()) {
            let err = (a - b).abs();
            abs_error += err;
            squared_error += err * err;
            if err > max_abs_error {
                max_abs_error = err;
            }
        }

        let scale = (1 << 14) as f32;
        let count = want.len() as f32;
        let aad = abs_error.sum() / count * scale;
        let rmse = (squared_error.sum() / count).sqrt() * scale;
        max_abs_error *= scale;

        assert_eq!(
            (0.8496094, 0.27195325, 0.34617355),
            (max_abs_error, aad, rmse)
        );
    }
}
