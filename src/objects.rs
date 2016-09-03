use serde::de;
use regex::Regex;

#[derive(Debug, Deserialize)]
pub struct Ellipse;

#[derive(Debug, Deserialize)]
pub struct Polyline {
    #[serde(deserialize_with="::objects::deserialize_points")]
    #[serde(default)]
    pub points: Vec<(i32, i32)>,
}

#[derive(Debug, Deserialize)]
pub struct Polygon {
    #[serde(deserialize_with="::objects::deserialize_points")]
    #[serde(default)]
    pub points: Vec<(i32, i32)>,
}

fn deserialize_points<D: de::Deserializer> (deserializer: &mut D)
                                            -> Result<Vec<(i32, i32)>, D::Error> {
    let points_str: String = de::Deserialize::deserialize(deserializer)?;

    lazy_static! {
        static ref POINTS_REGEX: Regex = Regex::new(r"((-?\d+),(-?\d+))").unwrap();
    }

    POINTS_REGEX.captures_iter(&points_str)
        .map(|cap| {
            let first = cap.at(2)
                .ok_or(de::Error::custom(format!("could not match from regex (points)")))?
                .parse()
                .map_err(|e| de::Error::custom(format!("could not decode points: {}", e)))?;
            let second = cap.at(3)
                .ok_or(de::Error::custom(format!("could not match from regex (points)")))?
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
    pub x: i32,
    pub y: i32,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub rotation: Option<f32>,

    pub is_ellipse: Option<Ellipse>,
    pub polyline: Option<Polyline>,
    pub polygon: Option<Polygon>,
}

#[derive(Debug, Deserialize)]
pub struct Objectgroup {
    pub name: String,
    pub draworder: Option<String>,
    pub visible: Option<u8>,
    pub opacity: Option<f32>,

    #[serde(rename(deserialize="object"))]
    pub objects: Vec<Object>,
}
