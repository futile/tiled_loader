use serde::de;

use super::Color;

impl<'de> de::Deserialize<'de> for Color {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Color, D::Error> {
        use regex::Regex;

        let color_str: String = de::Deserialize::deserialize(deserializer)?;

        lazy_static! {
            static ref COLOR_REGEX: Regex = Regex::new(
                r"(?x)#?
(?P<alpha>[[:xdigit:]]{2})?
(?P<red>[[:xdigit:]]{2})
(?P<green>[[:xdigit:]]{2})
(?P<blue>[[:xdigit:]]{2})"
            ).unwrap();
        }

        use serde::de::Error;

        let caps = COLOR_REGEX
            .captures(&color_str)
            .ok_or(Error::custom(format!(
                "color did not match regex: {}",
                &color_str
            )))?;

        let red = caps.name("red")
            .ok_or(Error::custom("could not deserialize red"))?
            .into();
        let green = caps.name("green")
            .ok_or(Error::custom("could not deserialize green"))?
            .into();
        let blue = caps.name("blue")
            .ok_or(Error::custom("could not deserialize blue"))?
            .into();
        let alpha = caps.name("alpha");

        let red = u8::from_str_radix(red, 16)
            .map_err(|e| Error::custom(format!("could not parse red: {}", e)))?;
        let green = u8::from_str_radix(green, 16)
            .map_err(|e| Error::custom(format!("could not parse green: {}", e)))?;
        let blue = u8::from_str_radix(blue, 16)
            .map_err(|e| Error::custom(format!("could not parse blue: {}", e)))?;
        let alpha = alpha.map_or(Ok(255), |alph| {
            u8::from_str_radix(alph.into(), 16)
                .map_err(|e| Error::custom(format!("could not parse alpha: {}", e)))
        })?;

        Ok(Color {
            r: red,
            g: green,
            b: blue,
            a: alpha,
        })
    }
}
