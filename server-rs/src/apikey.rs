use crate::error::Error;
use itertools::Itertools;
use sha2::{Digest, Sha256};
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Write;
use uuid::Uuid;

const HEX_CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

/// A hashed api key (safe to store in db + otherwise expose)
#[derive(PartialEq, Eq, Debug)]
pub struct HashedApiKey([u8; 32]);

impl fmt::Display for HashedApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.0 {
            f.write_char(HEX_CHARS[(b & 0x0f) as usize])?;
            f.write_char(HEX_CHARS[((b >> 4) & 0x0f) as usize])?;
        }

        Ok(())
    }
}

impl TryFrom<String> for HashedApiKey {
    type Error = Error;

    fn try_from(key: String) -> Result<Self, Self::Error> {
        let mut res = [0u8; 32];

        if key.len() != 64 {
            Err(Error::MalformedApiKey)
        } else {
            for (i, (c1, c2)) in key.chars().tuples().enumerate() {
                let v1 = HEX_CHARS.iter().position(|c| *c == c1);
                let v2 = HEX_CHARS.iter().position(|c| *c == c2);
                match (v1, v2) {
                    (Some(i1), Some(i2)) => {
                        res[i] = ((i1 & 0xf) + ((i2 << 4) & 0xf0) & 0xff) as u8;
                    }
                    (_, _) => return Err(Error::MalformedApiKey),
                }
            }

            Ok(HashedApiKey(res))
        }
    }
}

/// A non hashed api key (not safe to expose in db)
#[derive(PartialEq, Eq, Debug)]
pub struct ApiKey(Uuid);

impl ApiKey {
    pub fn hash(&self) -> HashedApiKey {
        let key_hash = Sha256::digest(self.0.as_bytes());

        let mut hash = [0; 32];
        for (i, b) in key_hash.as_slice().iter().enumerate() {
            hash[i] = *b;
        }

        HashedApiKey(hash)
    }

    pub fn new() -> ApiKey {
        ApiKey(Uuid::new_v4())
    }
}

impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.simple())
    }
}

impl TryFrom<&str> for ApiKey {
    type Error = Error;
    fn try_from(str: &str) -> Result<ApiKey, Self::Error> {
        match Uuid::parse_str(str) {
            Ok(uuid) => Ok(ApiKey(uuid)),
            Err(_) => Err(Error::MalformedApiKey),
        }
    }
}
