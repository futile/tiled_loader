use serde::de;
use regex::Regex;

use base64;
use byteorder::{LittleEndian, ReadBytesExt};

use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq)]
pub enum DataEncoding {
    Base64,
    CSV,
    XML,
}

impl DataEncoding {
    fn decode<E: de::Error>(
        &self,
        data_text: &str,
        compression: &DataCompression,
    ) -> Result<Vec<u32>, E> {
        match *self {
            DataEncoding::CSV => {
                if compression != &DataCompression::None {
                    return Err(de::Error::custom(
                        "compression with csv-encoding not allowed",
                    ));
                }

                lazy_static! {
                    static ref CSV_REGEX: Regex = Regex::new(r"(\d+)").unwrap();
                }

                CSV_REGEX
                    .captures_iter(&data_text)
                    .map(|cap| {
                        cap.get(1).ok_or(de::Error::custom(format!(
                            "could not match from regex (csv)"
                        )))
                    })
                    .map(|s| {
                        s?.as_str()
                            .parse()
                            .map_err(|e| de::Error::custom(format!("could not decode CSV: {}", e)))
                    })
                    .collect()
            }
            DataEncoding::Base64 => {
                let decoded_raw: Vec<u8> = base64::decode(&data_text)
                    .map_err(|e| de::Error::custom(format!("could not decode base64: {}", e)))?;

                let decompressed = compression.decompress(&decoded_raw.as_slice())?;

                decompressed
                    .chunks(4)
                    .map(|mut bytes| {
                        bytes.read_u32::<LittleEndian>().map_err(|e| {
                            de::Error::custom(format!(
                                "could not decode from little \
                                 endian(base64): {}",
                                e
                            ))
                        })
                    })
                    .collect()
            }
            _ => panic!("not yet implemented"),
            // DataEncoding::XML => {
            //     use std::num::ParseIntError;
            //     use std::error::Error;

            //     let members = match data_content {
            //         &Content::Members(ref members) => members,
            //         _ => {
            //             return Err(de::Error::custom(format!("expected members, got {:?}",
            //                                                  data_content)))
            //         }
            //     };

            //     let tiles = match members.get("tile") {
            //         Some(tiles) => tiles,
            //         None => return Ok(Vec::new()),
            //     };

            //     tiles.iter()
            //         .map(|e| {
            //             match e.attributes.get("gid") {
            //                 Some(gids) => {
            //                     if gids.len() != 1 {
            //                         Err(de::Error::custom(format!("expected exactly one gid, \
            //                                                        got {:?}",
            //                                                       gids)))
            //                     } else {
            //                         gids[0].parse().map_err(|e: ParseIntError| {
            //                             de::Error::custom(e.description())
            //                         })
            //                     }
            //                 }
            //                 _ => unimplemented!(),
            //             }
            //         })
            //         .collect()
            // }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DataCompression {
    None,
    Zlib,
    Gzip,
}

impl DataCompression {
    fn decompress<'a, E: de::Error>(&self, compressed: &'a [u8]) -> Result<Cow<'a, [u8]>, E> {
        use std::io::Read;

        match *self {
            DataCompression::None => Ok(Cow::Borrowed(compressed)),
            DataCompression::Zlib => {
                use flate2::read::ZlibDecoder;

                let mut decoder = ZlibDecoder::new(compressed);
                let mut decoded = Vec::new();

                decoder.read_to_end(&mut decoded).map_err(|e| {
                    de::Error::custom(format!("could not decode zlib-compressed data: {}", e))
                })?;

                Ok(Cow::Owned(decoded))
            }
            DataCompression::Gzip => {
                use flate2::read::GzDecoder;

                let mut decoder = GzDecoder::new(compressed);
                let mut decoded = Vec::new();

                decoder.read_to_end(&mut decoded).map_err(|e| {
                    de::Error::custom(format!("could not decode gzip-compressed data: {}", e))
                })?;

                Ok(Cow::Owned(decoded))
            }
        }
    }
}

#[derive(Debug)]
pub struct Data {
    pub encoding: DataEncoding,
    pub compression: DataCompression,

    pub tile_gids: Vec<u32>,
}

impl<'de> de::Deserialize<'de> for Data {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Data, D::Error> {
        #[derive(Debug, Deserialize)]
        #[serde(rename = "tile")]
        struct SimpleTile {
            gid: u32,
        }

        #[derive(Debug, Deserialize)]
        #[serde(untagged)]
        enum DataContent {
            Str(String),
            Tiles(Vec<SimpleTile>),
        };

        fn string_or_struct<'de, D>(deserializer: D) -> Result<Option<DataContent>, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            struct StringOrStruct(::serde::export::PhantomData<fn() -> Option<DataContent>>);

            impl<'de> de::Visitor<'de> for StringOrStruct {
                type Value = Option<DataContent>;

                fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    formatter.write_str("string or map")
                }

                fn visit_str<E>(self, value: &str) -> Result<Option<DataContent>, E>
                where
                    E: de::Error,
                {
                    Ok(Some(DataContent::Str(value.into())))
                }

                fn visit_map<M>(self, _visitor: M) -> Result<Option<DataContent>, M::Error>
                where
                    M: de::MapAccess<'de>,
                {
                    Err(de::Error::custom(
                        "currently not possible to decode XML data",
                    ))
                }
            };

            deserializer.deserialize_any(StringOrStruct(::serde::export::PhantomData))
        }

        #[derive(Debug, Deserialize)]
        struct DataImpl {
            encoding: Option<String>,
            compression: Option<String>,
            #[serde(rename = "$value", deserialize_with = "string_or_struct", default)]
            value: Option<DataContent>,
        };

        let val: DataImpl = de::Deserialize::deserialize(deserializer)?;

        let enc = match val.encoding.as_ref().map(|s| s.as_str()) {
            Some("base64") => DataEncoding::Base64,
            Some("csv") => DataEncoding::CSV,
            Some(e) => return Err(de::Error::custom(format!("unexpected encoding: '{}'", e))),
            None => DataEncoding::XML,
        };

        let comp = match val.compression.as_ref().map(|s| s.as_str()) {
            Some("zlib") => DataCompression::Zlib,
            Some("gzip") => DataCompression::Gzip,
            Some(e) => {
                return Err(de::Error::custom(format!(
                    "unexpected compression: '{}'",
                    e
                )))
            }
            None => DataCompression::None,
        };

        let gids = match val.value {
            Some(DataContent::Str(s)) => enc.decode(&s, &comp)?,
            Some(DataContent::Tiles(ts)) => ts.iter().map(|t| t.gid).collect(),
            None => Vec::new(),
        };

        Ok(Data {
            encoding: enc,
            compression: comp,
            tile_gids: gids,
        })
    }
}
