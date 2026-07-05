use serde::{de::DeserializeOwned, Serialize};

use crate::error::SerializeError;

/// Serializer boundary. Codecs choose serializers but do not own their format.
pub trait Serializer: Send + Sync {
    fn name(&self) -> &'static str;

    fn content_type(&self) -> &'static str;

    fn serialize<T>(&self, value: &T) -> Result<Vec<u8>, SerializeError>
    where
        T: Serialize + ?Sized;

    fn deserialize<T>(&self, bytes: &[u8]) -> Result<T, SerializeError>
    where
        T: DeserializeOwned;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct JsonSerializer;

impl Serializer for JsonSerializer {
    fn name(&self) -> &'static str {
        "json"
    }

    fn content_type(&self) -> &'static str {
        "application/json"
    }

    fn serialize<T>(&self, value: &T) -> Result<Vec<u8>, SerializeError>
    where
        T: Serialize + ?Sized,
    {
        serde_json::to_vec(value).map_err(|error| SerializeError::Serialize(error.to_string()))
    }

    fn deserialize<T>(&self, bytes: &[u8]) -> Result<T, SerializeError>
    where
        T: DeserializeOwned,
    {
        serde_json::from_slice(bytes)
            .map_err(|error| SerializeError::Deserialize(error.to_string()))
    }
}
