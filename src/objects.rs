use serde::de;
use regex::Regex;

use ::{Properties, Color};

#[derive(Debug, Deserialize)]
pub struct Ellipse;

#[derive(Debug, Deserialize)]
pub struct Polyline {
    #[serde(deserialize_with="::objects::deserialize_points")]
    #[serde(default)]
    pub points: Vec<(f32, f32)>,
}

#[derive(Debug, Deserialize)]
pub struct Polygon {
    #[serde(deserialize_with="::objects::deserialize_points")]
    #[serde(default)]
    pub points: Vec<(f32, f32)>,
}

fn deserialize_points<'de, D: de::Deserializer<'de>> (deserializer: D)
                                            -> Result<Vec<(f32, f32)>, D::Error> {
    let points_str: String = de::Deserialize::deserialize(deserializer)?;

    lazy_static! {
        static ref POINTS_REGEX: Regex = Regex::new(r"((-?\d+),(-?\d+))").unwrap();
    }

    POINTS_REGEX.captures_iter(&points_str)
        .map(|cap| {
            let first = cap.get(2)
                .ok_or(de::Error::custom(format!("could not match from regex (points)")))?
                .as_str()
                .parse()
                .map_err(|e| de::Error::custom(format!("could not decode points: {}", e)))?;
            let second = cap.get(3)
                .ok_or(de::Error::custom(format!("could not match from regex (points)")))?
                .as_str()
                .parse()
                .map_err(|e| de::Error::custom(format!("could not decode points: {}", e)))?;

            Ok((first, second))
        })
        .collect()
}

#[derive(Debug, Deserialize)]
pub struct Object {
    pub id: u32,
    pub name: Option<String>,
    #[serde(rename(deserialize="type"))]
    pub type_: Option<String>,
    pub gid: Option<u32>,
    pub x: f32,
    pub y: f32,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub rotation: Option<f32>,

    #[serde(deserialize_with="::properties::deserialize_properties")]
    #[serde(default)]
    pub properties: Option<Properties>,

    pub ellipse: Option<()>,
    pub polyline: Option<Polyline>,
    pub polygon: Option<Polygon>,
}

#[derive(Debug, Deserialize)]
pub struct Objectgroup {
    pub name: String,
    pub draworder: Option<String>,
    pub visible: Option<u8>,
    pub opacity: Option<f32>,
    pub offsetx: Option<f32>,
    pub offsety: Option<f32>,
    pub color: Option<Color>,

    #[serde(deserialize_with="::properties::deserialize_properties")]
    #[serde(default)]
    pub properties: Option<Properties>,

    #[serde(rename(deserialize="object"), default)]
    pub objects: Vec<Object>,
}

