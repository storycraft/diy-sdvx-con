use std::{collections::HashMap, sync::LazyLock};

use regex::{Regex, RegexBuilder};
use serde::de::Unexpected;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Spec {
    #[serde(default)]
    pub keycodes: HashMap<String, KeycodeIns>,

    #[serde(default)]
    pub ranges: HashMap<KeyRange, KeyRangeIns>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum KeycodeIns {
    Delete(Delete),
    Reset(u8),
    Def(Keycode),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Keycode {
    pub group: Option<String>,
    pub key: String,
    pub label: Option<String>,
    #[serde(default)]
    pub aliases: Vec<AliasIns>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum AliasIns {
    Reset(Reset),
    Def(String),
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct KeyRange {
    pub start: u16,
    pub end: u16,
}

impl serde::Serialize for KeyRange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:#04X}/{:#04X}", self.start, self.end))
    }
}

impl<'de> serde::Deserialize<'de> for KeyRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'a> serde::de::Visitor<'a> for Visitor {
            type Value = KeyRange;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("expecting range string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                    RegexBuilder::new(r"0x([0-9a-f]{4})\/0x([0-9a-f]{4})")
                        .case_insensitive(true)
                        .build()
                        .unwrap()
                });

                let Some(captures) = REGEX.captures(v) else {
                    return Err(E::invalid_value(Unexpected::Str(v), &"range format string"));
                };

                let (_, [start, end]) = captures.extract();
                Ok(KeyRange {
                    start: u16::from_str_radix(start, 16).unwrap(),
                    end: u16::from_str_radix(end, 16).unwrap(),
                })
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum KeyRangeIns {
    Delete(Delete),
    Def { define: String },
}

macro_rules! impl_constants {
    ($ty:tt = $lit:literal) => {
        const _: () = {
            impl $ty {
                pub const VALUE: &str = $lit;
            }

            impl ::serde::Serialize for $ty {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: ::serde::Serializer,
                {
                    serializer.serialize_str($lit)
                }
            }

            impl<'de> ::serde::Deserialize<'de> for $ty {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: ::serde::Deserializer<'de>,
                {
                    deserializer.deserialize_str(ConstantVisitor)
                }
            }

            struct ConstantVisitor;
            impl<'a> ::serde::de::Visitor<'a> for ConstantVisitor {
                type Value = $ty;

                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.write_str("expected string")
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: ::serde::de::Error,
                {
                    match v {
                        $lit => Ok($ty),
                        _ => Err(E::custom(format!("unexpected string value: {}", v))),
                    }
                }
            }
        };
    };
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Delete;
impl_constants!(Delete = "!delete!");

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Reset;
impl_constants!(Reset = "!reset!");
