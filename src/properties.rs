use std::collections::HashMap;
use serde::de;

use super::Property;

pub type Properties = HashMap<String, Property>;

pub fn deserialize_properties<'de, D: de::Deserializer<'de>>(deserializer: D)
                                                   -> Result<Option<Properties>, D::Error> {
    #[derive(Debug, Deserialize)]
    pub struct RawProperty {
        pub name: String,
        #[serde(rename(deserialize="type"))]
        #[serde(default)]
        pub type_: String,
        pub value: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct RawProperties {
        #[serde(rename(deserialize="property"))]
        properties: Vec<RawProperty>,
    }

    let raw_props: RawProperties = de::Deserialize::deserialize(deserializer)?;
    let mut props = Properties::new();

    use std::num::ParseFloatError;
    use std::num::ParseIntError;
    use std::str::ParseBoolError;
    use std::error::Error;

    for raw_prop in raw_props.properties {
        if props.contains_key(&raw_prop.name) {
            return Err(de::Error::custom(format!("property '{}' was found twice", raw_prop.name)));
        }

        let val = match &raw_prop.type_[..] {
            "" | "string" => Property::String(raw_prop.value),
            "float" => {
                Property::Float(raw_prop.value
                    .parse()
                    .map_err(|e: ParseFloatError| de::Error::custom(e.description()))?)
            }
            "bool" => {
                Property::Bool(raw_prop.value
                    .parse()
                    .map_err(|e: ParseBoolError| de::Error::custom(e.description()))?)
            }
            "int" => {
                Property::Int(raw_prop.value
                    .parse()
                    .map_err(|e: ParseIntError| de::Error::custom(e.description()))?)
            }
            s => return Err(de::Error::custom(format!("unexpected property type: '{}'", s))),
        };

        props.insert(raw_prop.name, val);
    }

    Ok(Some(props))
}
