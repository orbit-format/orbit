use std::fmt::{self, Display};

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrbitNumber {
    Integer(i64),
    Float(f64),
}

impl OrbitNumber {
    pub fn as_f64(self) -> f64 {
        match self {
            OrbitNumber::Integer(value) => value as f64,
            OrbitNumber::Float(value) => value,
        }
    }

    pub fn as_i64(self) -> Option<i64> {
        match self {
            OrbitNumber::Integer(value) => Some(value),
            OrbitNumber::Float(value) => {
                if value.fract() == 0.0 {
                    Some(value as i64)
                } else {
                    None
                }
            }
        }
    }
}

impl From<i64> for OrbitNumber {
    fn from(value: i64) -> Self {
        OrbitNumber::Integer(value)
    }
}

impl From<f64> for OrbitNumber {
    fn from(value: f64) -> Self {
        OrbitNumber::Float(value)
    }
}

impl Display for OrbitNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrbitNumber::Integer(value) => write!(f, "{}", value),
            OrbitNumber::Float(value) => write!(f, "{}", value),
        }
    }
}

impl Serialize for OrbitNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            OrbitNumber::Integer(value) => serializer.serialize_i64(*value),
            OrbitNumber::Float(value) => serializer.serialize_f64(*value),
        }
    }
}

impl<'de> Deserialize<'de> for OrbitNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OrbitNumberVisitor;

        impl<'de> Visitor<'de> for OrbitNumberVisitor {
            type Value = OrbitNumber;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a numeric literal")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(OrbitNumber::Integer(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value <= i64::MAX as u64 {
                    Ok(OrbitNumber::Integer(value as i64))
                } else {
                    Ok(OrbitNumber::Float(value as f64))
                }
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(OrbitNumber::Float(value))
            }
        }

        deserializer.deserialize_any(OrbitNumberVisitor)
    }
}
