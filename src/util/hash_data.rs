use crate::{IntegrationOSError, InternalError};
use serde_json::Value;
use sha3::{Digest, Keccak256};

pub struct HashData;

impl HashData {
    pub fn create_from_value(value: Value) -> Result<String, IntegrationOSError> {
        let value_str = serde_json::to_string(&value).map_err(|err| {
            InternalError::invalid_argument(&format!("Failed to serialize value: {err}"), None)
        })?;

        Ok(Self::create(&value_str))
    }

    pub fn create(value: &str) -> String {
        let mut hasher = Keccak256::new();
        hasher.update(value);
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create() {
        let value = json!({
            "response": {
                "user": {
                    "name": "Alice",
                    "age": 3
                }
            }
        });

        let hash = HashData::create_from_value(value).unwrap();

        assert_eq!(
            hash,
            "eb42c0c05a0ac6cd15e4cf907a6aa913ebfe6aea79ee7edd054b519435f827cb"
        );
    }
}
