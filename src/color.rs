// use css_colors::{self, CssColor};
use cssparser_color::{parse_color_keyword, Color as CssParserColor};
use palette::{FromColor, Hsl, Lab, Lch, Oklab, Oklch, Srgb};
use std::fmt::Debug;

#[derive(Debug, Clone)]
enum Space {
    Rgb,
    Hsl,
    Lab,
    Lch,
    Oklch,
    Oklab,
    Hex,
    CssName,
}

#[derive(Debug, Clone)]
pub(crate) struct Color {
    space: Space,
    pub(crate) color_value: Srgb<f32>,
}

impl Color {
    fn new(space: Space, color_value: Srgb<f32>) -> Self {
        Self { space, color_value }
    }

    pub(crate) fn from_repr(s: &str) -> Result<Self, &'static str> {
        Self::from_css(&convert_to_css_string(s))
    }

    fn from_css(css_string: &str) -> Result<Self, &'static str> {
        let mut input = cssparser::ParserInput::new(css_string);
        let mut parser = cssparser::Parser::new(&mut input);

        let css_color = CssParserColor::parse(&mut parser).map_err(|_| "Invalid color format")?;

        let srgb = match css_color {
            CssParserColor::CurrentColor => return Err("Unsupported: currentColor"),
            CssParserColor::Rgba(rgba) => {
                let red = rgba.red as f32 / 255.0;
                let green = rgba.green as f32 / 255.0;
                let blue = rgba.blue as f32 / 255.0;
                Srgb::new(red, green, blue)
            }
            CssParserColor::Hsl(hsla) => {
                let hue = hsla.hue.ok_or("Missing hue value")?;
                let saturation = hsla.saturation.ok_or("Missing saturation value")?;
                let lightness = hsla.lightness.ok_or("Missing lightness value")?;
                let hsl = Hsl::new(hue, saturation, lightness);
                Srgb::from_color(hsl)
            }
            CssParserColor::Lab(lab) => {
                let lightness = lab.lightness.ok_or("Missing lightness value")?;
                let a = lab.a.ok_or("Missing 'a' value")?;
                let b = lab.b.ok_or("Missing 'b' value")?;
                let lab = Lab::new(lightness, a, b);
                Srgb::from_color(lab)
            }
            CssParserColor::Lch(lch) => {
                let lightness = lch.lightness.ok_or("Missing lightness value")?;
                let chroma = lch.chroma.ok_or("Missing chroma value")?;
                let hue = lch.hue.ok_or("Missing hue value")?.to_degrees();
                let lch = Lch::new(lightness, chroma, hue);
                Srgb::from_color(lch)
            }
            CssParserColor::Oklab(oklab) => {
                let lightness = oklab.lightness.ok_or("Missing lightness value")?;
                let a = oklab.a.ok_or("Missing 'a' value")?;
                let b = oklab.b.ok_or("Missing 'b' value")?;
                let oklab = Oklab::new(lightness, a, b);
                Srgb::from_color(oklab)
            }
            CssParserColor::Oklch(oklch) => {
                let lightness = oklch.lightness.ok_or("Missing lightness value")?;
                let chroma = oklch.chroma.ok_or("Missing chroma value")?;
                let hue = oklch.hue.ok_or("Missing hue value")?.to_degrees();
                let oklch = Oklch::new(lightness, chroma, hue);
                Srgb::from_color(oklch)
            }
            _ => return Err("Unsupported color format"),
        };

        // BAKERT this is a rpeat of the enum, combine them somehow
        let space = if css_string.starts_with("rgb") {
            Space::Rgb
        } else if css_string.starts_with("hsl") {
            Space::Hsl
        } else if css_string.starts_with("lab") {
            Space::Lab
        } else if css_string.starts_with("lch") {
            Space::Lch
        } else if css_string.starts_with("oklab") {
            Space::Oklab
        } else if css_string.starts_with("oklch") {
            Space::Oklch
        } else if css_string.starts_with("#") {
            Space::Hex
        } else if parse_color_keyword::<CssParserColor>(css_string).is_ok() {
            Space::CssName
        } else {
            return Err("Unknown color space");
        };

        Ok(Color::new(space, srgb))
    }
}

// BAKERT IsWithinBounds

// fn convert_to_css_string(input: &str) -> Result<String, &'static str> {
//     // BAKERT we need to get more sewious here … "blah/fuck/pope" is not legal
//     // OR we have to make this return just the str not the Result because it can't fail
//     // BAKERT make sure if you pass in a nromal css defitnion with a forward slash we don't mangle it here, or at least we Err so you don't use the mangle
//     let components: Vec<&str> = input.split('/').collect();
//     if components.len() < 2 {
//         return Err("Invalid format: Must be in the form 'space/component/component/...'");
//     }
//     let space = components[0];
//     let values = components[1..].join(" ");
//     let result = format!("{}({})", space, values);
//     Ok(result)
// }

// BAKERT
fn convert_to_css_string(input: &str) -> String {
    if input.contains('(') || input.contains(')') {
        return input.to_string();
    }
    if !input.contains('/') {
        return input.to_string();
    }

    let components: Vec<&str> = input.split('/').collect();
    if components.is_empty() {
        return input.to_string();
    }

    let space = components[0];
    // BAKERT should be a constant
    if space == "css" {
        return components[1].to_string(); // BAKERT maybe invalid if more than 2 parts?
    }
    let components: Vec<String> = components.iter().map(|s| s.replace("pct", "%")).collect();

    let values = if components.len() == 5 {
        let main_values = components[1..4].join(" ");
        let alpha = &components[4];
        format!("{} / {}", main_values, alpha)
    } else {
        components[1..].join(" ")
    };

    format!("{}({})", space, values)
}

// fn find_named_color(color: Srgb<u8>) -> Option<&'static str> {
//     // Iterate over all named CSS colors and check if any match the given Srgb value
//     for named_color in css_colors::NAMED_COLORS.iter() {
//         let named_srgb = Srgb::new(
//             (named_color.color.r * 255.0) as u8,
//             (named_color.color.g * 255.0) as u8,
//             (named_color.color.b * 255.0) as u8,
//         );
//         if named_srgb == color {
//             return Some(named_color.name);
//         }
//     }
//     None
// }

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_convert_to_css_string() {
        let test_cases = vec![
            ("rgb/255/127/0", "rgb(255 127 0)"),
            ("rgb/100pct/50pct/0pct", "rgb(100% 50% 0%)"),
            ("rgb/0/206/209/1.0", "rgb(0 206 209 / 1.0)"),
            ("hsl/34.99/0.4/0.1", "hsl(34.99 0.4 0.1)"),
            ("rgba/255/127/0/0.5", "rgba(255 127 0 / 0.5)"),
            ("lch/50/100/120", "lch(50 100 120)"),
            ("oklch/0.8/0.2/120", "oklch(0.8 0.2 120)"),
            ("oklch/0.8/0.2/120/0.5", "oklch(0.8 0.2 120 / 0.5)"),
            ("css/darkturquoise", "darkturquoise"),
            ("darkturquoise", "darkturquoise"),
        ];
        for (input, expected_output) in &test_cases {
            assert_eq!(convert_to_css_string(input), expected_output.to_string());
        }
        let ignore_cases = vec![
            "oklab(0.77 -0.16 -0.02 / 0.5)",
            "oklch(0.77 0.16 187.69 / 0.5)",
        ];
        for bad_input in &ignore_cases {
            assert_eq!(convert_to_css_string(bad_input), bad_input.to_string());
        }
    }

    #[test]
    fn test_parse() {
        let dark_turquoise_representations = vec![
            "rgb/0/206/209",
            "rgb/0/206/209/1.0",
            "rgb(0 206 209)",
            "rgb(0, 206, 209)",
            "rgb(0,206,209)",
            "rgb(0 206 209 / 1.0)",
            // BAKERT need to do an rgb that uses %, if thta's legal in css
            "hsl(181, 100%, 41%)",
            "hsla/181/100.0pct/41pct/1.0",
            "hsla(181, 100%, 41%, 1.0)",
            // "hsl(181deg 30% 60%)",
            "hsl(0.50277778turn 100% 41% / 1.0)",
            "hsl(181 100% 41% / 100%)",
            "lab/75.29/-40.04/-13.52",
            "lab(75.29 -40.04 -13.52)",
            "lab(75.29 -40.05 -13.52 / 1.0)",
            // "lch(75.29 42.26 198.66)",
            // "lch/75.29/42.26/198.66/0.5",
            // "lch(75.29 42.26 198.66 / 0.5)",
            // "oklab(0.77 -0.16 -0.02)",
            // "oklab(0.77 -0.16 -0.02 / 0.5)",
            // "oklch(77.19% 0.131 196.64)",
            // "oklch(77.19% 0.131 196.64 / 1.0)",
            // "lab(50% 0 0 / 0.5)",
            "darkturquoise",
            "css/darkturquoise",
            // BAKERT we're going to have to cope with these as # won't work in canoncial repr of hex even though it would be way better "00ced1",
            "#00ced1",
        ];

        const expected: Srgb = Srgb::new(0.0, 206.0 / 255.0, 209.0 / 255.0);

        for representation in &dark_turquoise_representations {
            let actual = match Color::from_repr(representation) {
                Ok(color) => color,
                Err(e) => panic!("Failed to parse color {}: {}", representation, e),
            };
            println!("{}", representation);
            compare_colors(expected, actual.color_value);
            assert_eq!(
                1.0, actual.alpha,
                "Alpha values do not match: {} != {}",
                1.0, actual.alpha
            );
        }

        let bad_representations = vec![
            "rgb/1/2",
            // "rgb/-1/255/255",
            // "rgb/-1/255/255/0.1",
            // "rgb/-1/255/255/100",
            "gibberish",
            "lch/(76.45/33.69/196.47/0.5)",
            "very/much/nonsense",
            "very/much/nonsense/1.0",
            "rgb/very/much/nonsense",
            "rgb/very/much/nonsense/1.0",
        ];
        for representation in &bad_representations {
            match Color::from_repr(representation) {
                Ok(_) => panic!("Expected error for color {}", representation),
                Err(_) => (),
            }
        }
    }

    // #[test]
    // fn test_find_named_color() {
    //     let dark_turquoise = Srgb::new(0, 206, 209);
    //     assert_eq!(find_named_color(dark_turquoise), Some("darkturquoise"));
    //     let unknown_color = Srgb::new(0, 0, 0);
    //     assert_eq!(find_named_color(unknown_color), Some("black"));
    // }

    fn compare_colors(color1: Srgb, color2: Srgb) {
        const ALLOWED_DIFF: f32 = 1.0 / 255.0;
        assert_relative_eq!(color1.red, color2.red, max_relative = ALLOWED_DIFF);
        assert_relative_eq!(color1.green, color2.green, max_relative = ALLOWED_DIFF);
        assert_relative_eq!(color1.blue, color2.blue, max_relative = ALLOWED_DIFF);
    }
}
