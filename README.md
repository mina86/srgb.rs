# sRGB primitives and constants

The crate provides primitives for manipulating colours in sRGB colour
space.

Specifically, it provides functions for converting between sRGB,
linear sRGB and XYZ colour spaces; exposes definition of the D65
reference white point together with XYZ conversion matrices; and
finally provides functions for handling
[Rec.709](https://www.itu.int/rec/R-REC-BT.709-6-201506-I/en)
components encoding.

It intents to offer low-level primitives needed to work with sRGB
colour space.  Those primitives can be used by other libraries which
need to convert between sRGB and other colour spaces (if the
conversion requires going through XYZ colour space) or blend colours
together (which requires performing gamma correction).

Functions provided in the main module implement conversions between
sRGB and XYZ colour spaces.  Functions in [`gamma`] submodule provide
functions for doing gamma compression and expansion; they operate on
a single colour component.  Lastly, [`xyz`] submodule provides
functions for converting between linear sRGB and XYZ colour spaces as
well as constants exposing the matrices used by those functions.

The crate includes highly-optimised 8-bit gamma functions both when
converting from an 8-bit compressed value to a floating point linear
value as well as conversion in the opposite direction.  The latter is
over two and a half times faster than naïve implementation of the
gamma compression formula.

## Usage

Using this package with Cargo projects requires adding a single
dependency:

```toml
[dependencies]
srgb = "0.3"
```

With it in place, it’s now possible to write an application which
converts an sRGB colour into other colour spaces:

```rust
#[derive(Debug)]
struct RGBColour(u8, u8, u8);

impl RGBColour {
    fn parse(value: &str) -> Option<Self> {
        value.strip_prefix('#')
            .and_then(|v| (v.len() == 6 && !v.starts_with('+')).then(|| v))
            .and_then(|v| u32::from_str_radix(v, 16).ok())
            .map(|v| Self((v >> 16) as u8, (v >> 8) as u8, v as u8))
    }

    fn normalise(&self) -> (f32, f32, f32) {
        let [r, g, b] = srgb::normalised_from_u8([self.0, self.1, self.2]);
        (r, g, b)
        // Alternatively divide each component by 255 manually
    }

    fn expand_gamma(&self) -> (f32, f32, f32) {
        (
            srgb::gamma::expand_u8(self.0),
            srgb::gamma::expand_u8(self.1),
            srgb::gamma::expand_u8(self.2),
        )
        // Alternatively a convenience function is provided as well:
        // let [r, g, b] = srgb::gamma::linear_from_u8([self.0, self.1, self.2]);
        // (r, g, b)
    }

    fn to_xyz(&self) -> (f32, f32, f32) {
        let linear = srgb::gamma::linear_from_u8([self.0, self.1, self.2]);
        let [r, g, b] = srgb::xyz::xyz_from_linear(linear);
        (r, g, b)
        // Alternatively, if a custom matrix multiplication is available:
        // let [r, g, b] = matrix_product(
        //     srgb::xyz::XYZ_FROM_SRGB_MATRIX, linear);
    }
}

fn main() {
    for arg in std::env::args().into_iter().skip(1) {
        if let Some(rgb) = RGBColour::parse(&arg[..]) {
            println!("sRGB:       {:?}", rgb);
            println!("Normalised: {:?}", rgb.normalise());
            println!("Linear:     {:?}", rgb.expand_gamma());
            println!("XYZ:        {:?}", rgb.to_xyz());
        } else {
            eprintln!("expected ‘#rrggbb’ but got {}", arg);
        }
    }
}
```

## `rgb` crate support

This crate crate does not have an explicit [`rgb`
crate](https://crates.io/crates/rgb) support.  However, since all
functions taking an (s)RGB colour as argument accept `impl Into<[f32;
3]>` or `impl Into<[u8; 3]>` it is possible to pass `RGB` structure to
them.  Similarly, such functions return `[f32; 3]` or `[u8; 3]`which
can be converted into an `RGB` structure.

```rust
extern crate rgb;
use rgb::ComponentMap;

fn parse(value: &str) -> Option<rgb::RGB8> {
    value.strip_prefix('#')
        .and_then(|v| (v.len() == 6 && !v.starts_with('+')).then(|| v))
        .and_then(|v| u32::from_str_radix(v, 16).ok())
        .map(|v| (rgb::RGB::new((v >> 16) as u8, (v >> 8) as u8, v as u8)))
}

fn normalise(colour: rgb::RGB8) -> rgb::RGB<f32> {
    srgb::normalised_from_u8(colour).into()
}

fn expand_gamma(colour: rgb::RGB8) -> rgb::RGB<f32> {
    colour.map(srgb::gamma::expand_u8)
}

fn to_xyz(colour: rgb::RGB8) -> (f32, f32, f32) {
    let linear = srgb::gamma::linear_from_u8(colour);
    let [r, g, b] = srgb::xyz::xyz_from_linear(linear);
    (r, g, b)
}

fn main() {
    for arg in std::env::args().into_iter().skip(1) {
        if let Some(colour) = parse(&arg[..]) {
            println!("sRGB:       {:?}", colour);
            println!("Normalised: {:?}", normalise(colour));
            println!("Linear:     {:?}", expand_gamma(colour));
            println!("XYZ:        {:?}", to_xyz(colour));
        } else {
            eprintln!("expected ‘#rrggbb’ but got {}", arg);
        }
    }
}
```
