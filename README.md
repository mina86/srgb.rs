# sRGB primitives and constants

A lightweight crate providing functions and constants used in sRGB
colour space.  Specifically gamma correction, D65 definition and XYZ
conversion functions and constants.

The crate intents to provide low-level primitives needed to work with sRGB
colour space.  Those primitives can be used by other libraries which need to
convert between sRGB and other colour spaces (if the conversion requires
going through XYZ colour space) or blend colours together (which requires
performing gamma correction).

## Usage

Using this package with Cargo projects is as simple as adding a single
dependency:

```toml
[dependencies]
srgb = "0.1"
```

With that dependency in place, it’s now simple to write an application
which converts an sRGB colour into other colour spaces:


```
#[derive(Debug)]
struct RGB(u8, u8, u8);

impl RGB {
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
        // let [r, g, b] = srgb::linear_from_u8([self.0, self.1, self.2]);
        // (r, g, b)
    }

    fn to_xyz(&self) -> (f32, f32, f32) {
        let linear = srgb::linear_from_u8([self.0, self.1, self.2]);
        let [r, g, b] = srgb::xyz_from_linear(linear);
        (r, g, b)
        // Alternatively, if a custom matrix multiplication is available:
        // let [r, g, b] = matrix_product(
        //     srgb::xyz::XYZ_FROM_SRGB_MATRIX, linear);
    }
}

fn main() {
    for arg in std::env::args().into_iter().skip(1) {
        if let Some(rgb) = RGB::parse(&arg[..]) {
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
