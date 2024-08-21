use cssparser_color::{parse_color_keyword, Color as CssParserColor};
use palette::{Srgb, Hsl, Lab, Lch, Oklab, Oklch, FromColor};

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
    color_space: Space,
    pub(crate) color_value: Srgb<f32>,
    alpha: f32, // BAKERT the way we have alpha separately is nonsense
    original_representation: String,
}

impl Color {
    fn new(color_space: Space, color_value: Srgb<f32>, alpha: f32, original_representation: String) -> Self {
        Color {
            color_space,
            color_value,
            alpha,
            original_representation,
        }
    }

    pub(crate) fn from_repr(s: &str) -> Result<Self, &'static str> {
        Self::from_css(&convert_to_css_string(s))
    }

    fn from_css(css_string: &str) -> Result<Self, &'static str> {
        let mut input = cssparser::ParserInput::new(css_string);
        let mut parser = cssparser::Parser::new(&mut input);

        let css_color = CssParserColor::parse(&mut parser).map_err(|_| "Invalid color format")?;

        let (srgb, alpha) = match css_color {
            CssParserColor::CurrentColor => return Err("Unsupported: currentColor"),
            CssParserColor::Rgba(rgba) => {
                let red = rgba.red as f32;
                let green = rgba.green as f32;
                let blue = rgba.blue as f32;
                let alpha = rgba.alpha; // BAKERT why dont' thse need unrrap_or/ok_or like the others?
                (
                    Srgb::new(red, green, blue), // BAKERT need more u8 less f32? Need to be fleixble to all kinds? What does CssParserColor give us?
                    alpha,
                )
            }
            CssParserColor::Hsl(hsla) => {
                let hue = hsla.hue.ok_or("Missing hue value")?;
                let saturation = hsla.saturation.ok_or("Missing saturation value")?;
                let lightness = hsla.lightness.ok_or("Missing lightness value")?;
                let alpha = hsla.alpha.unwrap_or(1.0);
                // BAKERT there's a mismatch between css color's hsl and palette's hsl, I think
                let hsl = Hsl::new(hue, saturation, lightness);
                let srgb = Srgb::from_color(hsl);
                println!("hsl with {}, {}, {}", hue, saturation, lightness);
                (srgb, alpha)
            }
            CssParserColor::Lab(lab) => {
                let lightness = lab.lightness.ok_or("Missing lightness value")?;
                let a = lab.a.ok_or("Missing 'a' value")?;
                let b = lab.b.ok_or("Missing 'b' value")?;
                let lab = Lab::new(lightness, a, b);
                let srgb = Srgb::from_color(lab);
                (srgb, 1.0) // LAB does not have alpha, so default to 1.0
            }
            CssParserColor::Lch(lch) => {
                let lightness = lch.lightness.ok_or("Missing lightness value")?;
                let chroma = lch.chroma.ok_or("Missing chroma value")?;
                let hue = lch.hue.ok_or("Missing hue value")?.to_degrees();
                let lch = Lch::new(lightness, chroma, hue);
                let srgb = Srgb::from_color(lch);
                (srgb, 1.0) // Lch does not have alpha, so default to 1.0 BAKERT these comments are lies
            }
            CssParserColor::Oklab(oklab) => {
                let lightness = oklab.lightness.ok_or("Missing lightness value")?;
                let a = oklab.a.ok_or("Missing 'a' value")?;
                let b = oklab.b.ok_or("Missing 'b' value")?;
                let oklab = Oklab::new(lightness, a, b);
                let srgb = Srgb::from_color(oklab);
                (srgb, 1.0) // Oklab does not have alpha, so default to 1.0
            }
            CssParserColor::Oklch(oklch) => {
                let lightness = oklch.lightness.ok_or("Missing lightness value")?;
                let chroma = oklch.chroma.ok_or("Missing chroma value")?;
                let hue = oklch.hue.ok_or("Missing hue value")?.to_degrees();
                let oklch = Oklch::new(lightness, chroma, hue);
                let srgb = Srgb::from_color(oklch);
                (srgb, 1.0) // Oklch does not have alpha, so default to 1.0
            }
            _ => return Err("Unsupported color format"),
        };

        // BAKERT this is a rpeat of the enum, combine them somehow
        let color_space = if css_string.starts_with("rgb") {
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
        } else if let Ok(_) = parse_color_keyword::<CssParserColor>(css_string) {
            Space::CssName
        } else {
            return Err("Unknown color space");
        };

        Ok(Color::new(color_space, srgb, alpha, css_string.to_string()))
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
//     let color_space = components[0];
//     let values = components[1..].join(" ");
//     let result = format!("{}({})", color_space, values);
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

    let color_space = components[0];
    // BAKERT should be a constant
    if color_space == "css" {
        return components[1].to_string(); // BAKERT maybe invalid if more than 2 parts?
    }
    let components: Vec<String> = components
        .iter()
        .map(|s| s.replace("pct", "%"))
        .collect();

    let values = if components.len() == 5 {
        let main_values = components[1..4].join(" ");
        let alpha = &components[4];
        format!("{} / {}", main_values, alpha)
    } else {
        components[1..].join(" ")
    };

    format!("{}({})", color_space, values)
}

#[cfg(test)]
mod tests {
    use super::*;

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
            "hsl(181, 100%, 41%)",
            "hsla/180.3/100.0pct/40.9pct/0.5",
            "hsla(181, 100%, 41%, 0.5)",
            "lab/76.45/-32.33/-9.10",
            "lab(76.45 -32.33 -9.10)",
            "lab(76.45 -32.33 -9.10 / 0.5)",
            "lch(76.45 33.69 196.47)",
            "lch/76.45/33.69/196.47/0.5",
            "lch(76.45 33.69 196.47 / 0.5)",
            "oklab(0.77 -0.16 -0.02)",
            "oklab(0.77 -0.16 -0.02 / 0.5)",
            "oklch(0.77 0.16 187.69)",
            "oklch(0.77 0.16 187.69 / 0.5)",
            "lab(50% 0 0 / 0.5)",
            "darkturquoise",
            "css/darkturquoise",
            // BAKERT we're going to have to cope with these as # won't work in canoncial repr of hex even though it would be way better "00ced1",
            "#00ced1",
        ];

        const expected: Srgb = Srgb::new(0.0, 206.0, 209.0);

        for representation in &dark_turquoise_representations {
            let actual = match Color::from_repr(representation) {
                Ok(color) => color,
                Err(e) => panic!("Failed to parse color {}: {}", representation, e),
            };
            println!("{}", representation);
            assert_eq!(expected, actual.color_value, "Color values do not match: {:?} != {:?}", expected, actual.color_value);
            assert_eq!(1.0, actual.alpha, "Alpha values do not match: {} != {}", 1.0, actual.alpha);
        }

        // let bad_representations = vec![
        //     "rgb/1/2",
        //     "rgb/-1/255/255",
        //     "rgb/-1/255/255/0.1",
        //     "rgb/-1/255/255/100",
        //     "gibberish",
        //     "lch/(76.45/33.69/196.47/0.5)",
        // ];
    }
}