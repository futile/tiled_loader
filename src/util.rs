// from https://serde.rs/enum-str.html
macro_rules! enum_str {
    ($name:ident { $($variant:ident($str:expr), )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {
            $($variant,)*
        }

        // We don't need serialization
        // impl ::serde::Serialize for $name {
        //     fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        //         where S: ::serde::Serializer,
        //     {
        //         // Serialize the enum as a string.
        //         serializer.serialize_str(match *self {
        //             $( $name::$variant => $str, )*
        //         })
        //     }
        // }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: ::serde::Deserializer<'de>,
            {
                struct Visitor;

                impl<'dev> ::serde::de::Visitor<'dev> for Visitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("expected a valid variant")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<$name, E>
                        where E: ::serde::de::Error,
                    {
                        match value {
                            $( $str => Ok($name::$variant), )*
                                _ => Err(E::invalid_value(serde::de::Unexpected::Other(value),
                                                          &"expected a valid variant"))
                        }
                    }
                }

                // Deserialize the enum from a string.
                deserializer.deserialize_str(Visitor)
            }
        }
    }
}
