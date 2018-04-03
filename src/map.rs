use serde::de;

use super::{Orientation, TileRenderOrder, StaggerAxis, StaggerIndex, Color, MapLayer, Layer, Objectgroup, ImageLayer, Tileset, Properties};

#[derive(Debug, Deserialize)]
enum AnyMapLayer {
    #[serde(rename="layer")]
    Layer(Layer),
    #[serde(rename="objectgroup")]
    ObjectGroup(Objectgroup),
    #[serde(rename="imagelayer")]
    ImageLayer(ImageLayer),
    #[serde(rename="properties", deserialize_with="::properties::deserialize_properties")]
    Properties(Option<Properties>),
    #[serde(rename="tileset")]
    Tileset(Tileset),
}

#[derive(Debug, Deserialize)]
struct MapImpl {
    version: String,

    orientation: Orientation,
    renderorder: TileRenderOrder,
    hexsidelength: Option<i32>,
    staggeraxis: Option<StaggerAxis>,
    staggerindex: Option<StaggerIndex>,

    width: u32,
    height: u32,
    tilewidth: u32,
    tileheight: u32,

    nextobjectid: u32,
    backgroundcolor: Option<Color>,

    #[serde(rename="$value")]
    layers: Vec<AnyMapLayer>
}

impl<'de> de::Deserialize<'de> for super::Map {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<super::Map, D::Error> {
        let mapi: MapImpl = de::Deserialize::deserialize(deserializer)?;

        let mut layers = Vec::new();
        let mut properties = None;
        let mut tilesets = Vec::new();

        for layer in mapi.layers {
            match layer {
                AnyMapLayer::Properties(p) => {
                    if properties.is_none() {
                        properties = p;
                    } else {
                        return Err(de::Error::custom("multiple properties encountered"));
                    }
                },
                AnyMapLayer::Tileset(t) => tilesets.push(t),
                AnyMapLayer::Layer(l) => layers.push(MapLayer::Layer(l)),
                AnyMapLayer::ImageLayer(il) => layers.push(MapLayer::ImageLayer(il)),
                AnyMapLayer::ObjectGroup(o) => layers.push(MapLayer::ObjectGroup(o)),
            }
        }

        Ok(super::Map {
            version: mapi.version,

            orientation: mapi.orientation,
            renderorder: mapi.renderorder,
            hexsidelength: mapi.hexsidelength,
            staggeraxis: mapi.staggeraxis,
            staggerindex: mapi.staggerindex,

            width: mapi.width,
            height: mapi.height,
            tilewidth: mapi.tilewidth,
            tileheight: mapi.tileheight,

            nextobjectid: mapi.nextobjectid,
            backgroundcolor: mapi.backgroundcolor,

            layers: layers,
            properties: properties,
            tilesets: tilesets,
        })
    }
}
