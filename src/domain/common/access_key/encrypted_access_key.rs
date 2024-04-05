use super::{access_key_prefix::AccessKeyPrefix, encrypted_data::EncryptedData};
use crate::{IntegrationOSError, InternalError};
use base64ct::{Base64UrlUnpadded, Encoding};
use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
    str,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EncryptedAccessKey<'a> {
    pub prefix: AccessKeyPrefix,
    data: Cow<'a, str>,
}

impl<'a> EncryptedAccessKey<'a> {
    pub fn new(prefix: AccessKeyPrefix, data: String) -> Self {
        Self {
            prefix,
            data: Cow::Owned(data),
        }
    }

    pub fn to_static(self) -> EncryptedAccessKey<'static> {
        match self.data {
            Cow::Borrowed(data) => EncryptedAccessKey {
                prefix: self.prefix,
                data: Cow::Owned(data.to_owned()),
            },
            Cow::Owned(data) => EncryptedAccessKey {
                prefix: self.prefix,
                data: Cow::Owned(data),
            },
        }
    }

    pub fn parse(access_key: &'a str) -> Result<Self, IntegrationOSError> {
        // Parse out the prefix, separated by '_'
        let mut parts = access_key.splitn(4, '_');
        let event_type = parts
            .next()
            .ok_or(InternalError::configuration_error(
                "No event type in access key",
                None,
            ))?
            .try_into()?;
        let environment = parts
            .next()
            .ok_or(InternalError::configuration_error(
                "No environment in access key",
                None,
            ))?
            .try_into()?;
        let version = parts
            .next()
            .ok_or(InternalError::configuration_error(
                "No version in access key",
                None,
            ))?
            .parse()
            .map_err(|e| {
                InternalError::configuration_error(&format!("Invalid version: {}", e), None)
            })?;

        let remainder = parts.next().ok_or(InternalError::configuration_error(
            "No remainder in access key",
            None,
        ))?;

        Ok(Self {
            prefix: AccessKeyPrefix::new(environment, event_type, version),
            data: Cow::Borrowed(remainder),
        })
    }

    pub fn get_encrypted_data(&self) -> Result<EncryptedData, IntegrationOSError> {
        // Take the rest of the key and decode it from base64url
        let remainder = Base64UrlUnpadded::decode_vec(&self.data).map_err(|e| {
            InternalError::configuration_error(&format!("Invalid base64url: {}", e), None)
        })?;
        if remainder.len() < 48 {
            return Err(InternalError::configuration_error(
                "Encrypted data (remainder) is too short",
                None,
            ));
        }
        Ok(EncryptedData::new(remainder))
    }
}

impl Hash for EncryptedAccessKey<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl Display for EncryptedAccessKey<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}", self.prefix, self.data)
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::{
        access_key::event_type::EventType, configuration::environment::Environment,
    };

    use super::*;

    #[test]
    fn test_parse_encrypted_access_key() {
        let data = EncryptedAccessKey {
            prefix: AccessKeyPrefix {
                environment: Environment::Live,
                event_type: EventType::SecretKey,
                version: 1,
            },
            data: Cow::Borrowed("aDcyQWdjcGVhbnFkWmdYa2VibHg0ekF5Z3BuMXI5S3g1d1pEdkg2N0s1UUM4cEdZQ3d4VXZTc0dpSVdrQUlzd1BVQTZPUE9FdEdOUEI0SmU5enBwcG5uWVg4WW1iaGdoc0J0N0NsSUNuYU9jczJhNWkwSUF6YVVwaGhZa296MXI4QldMU0NpOW0wZ3BFdzd0SDFKaHhLLWs1TXktN1RnamZBJXphcnc1dER6dXQwMzdOb2NjT3RjTUElZ3NkV1JhYjRHQjVTSURYV0I1U2dOUWs0bGp6MXdOYkNBSmtTSEZRMW0tZw"),
        };
        let key = "sk_live_1_aDcyQWdjcGVhbnFkWmdYa2VibHg0ekF5Z3BuMXI5S3g1d1pEdkg2N0s1UUM4cEdZQ3d4VXZTc0dpSVdrQUlzd1BVQTZPUE9FdEdOUEI0SmU5enBwcG5uWVg4WW1iaGdoc0J0N0NsSUNuYU9jczJhNWkwSUF6YVVwaGhZa296MXI4QldMU0NpOW0wZ3BFdzd0SDFKaHhLLWs1TXktN1RnamZBJXphcnc1dER6dXQwMzdOb2NjT3RjTUElZ3NkV1JhYjRHQjVTSURYV0I1U2dOUWs0bGp6MXdOYkNBSmtTSEZRMW0tZw";
        let encrypted = EncryptedAccessKey::parse(key).unwrap();
        assert_eq!(data, encrypted);
    }

    #[test]
    fn test_get_encrypted_data() {
        let data = EncryptedData::new(
            [
                67, 189, 88, 80, 134, 114, 117, 200, 18, 192, 148, 13, 57, 64, 135, 133, 164, 204,
                170, 98, 47, 178, 82, 26, 124, 94, 75, 150, 227, 145, 37, 242, 74, 201, 211, 6, 16,
                177, 237, 97, 239, 242, 118, 208, 72, 173, 91, 168, 152, 75, 200, 50, 234, 202,
                104, 5, 42, 150, 232, 208, 243, 28, 236, 224, 176, 78, 199, 39, 255, 148, 157, 174,
                159, 10, 178, 23, 57, 53, 78, 112, 138, 4, 195, 114, 68, 156, 155, 200, 191, 81,
                76, 26, 125, 208, 17, 49, 19, 7, 246, 91, 3, 45, 83, 129, 67, 5, 72, 79, 217, 184,
                86, 155, 25, 187, 121, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 207, 44,
                160, 245, 243, 192, 153, 115, 52, 43, 237, 229, 134, 183, 40, 169, 184, 38, 75,
                100, 164, 54, 148, 18, 139, 221, 47, 209, 250, 113, 32, 4,
            ]
            .to_vec(),
        );
        let key = "sk_live_1_Q71YUIZydcgSwJQNOUCHhaTMqmIvslIafF5LluORJfJKydMGELHtYe_ydtBIrVuomEvIMurKaAUqlujQ8xzs4LBOxyf_lJ2unwqyFzk1TnCKBMNyRJybyL9RTBp90BExEwf2WwMtU4FDBUhP2bhWmxm7eQAAAAAAAAAAAAAAAAAAAADPLKD188CZczQr7eWGtyipuCZLZKQ2lBKL3S_R-nEgBA";
        let encrypted_access_key = EncryptedAccessKey::parse(key).unwrap();
        let encrypted = encrypted_access_key.get_encrypted_data().unwrap();
        assert_eq!(data, encrypted);
    }

    #[test]
    fn test_display() {
        let key = "sk_live_1_aDcyQWdjcGVhbnFkWmdYa2VibHg0ekF5Z3BuMXI5S3g1d1pEdkg2N0s1UUM4cEdZQ3d4VXZTc0dpSVdrQUlzd1BVQTZPUE9FdEdOUEI0SmU5enBwcG5uWVg4WW1iaGdoc0J0N0NsSUNuYU9jczJhNWkwSUF6YVVwaGhZa296MXI4QldMU0NpOW0wZ3BFdzd0SDFKaHhLLWs1TXktN1RnamZBJXphcnc1dER6dXQwMzdOb2NjT3RjTUElZ3NkV1JhYjRHQjVTSURYV0I1U2dOUWs0bGp6MXdOYkNBSmtTSEZRMW0tZw";
        let encrypted = EncryptedAccessKey::parse(key).unwrap();
        assert_eq!(key, encrypted.to_string());
    }
}
