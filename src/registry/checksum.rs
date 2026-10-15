use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};

const SHA1_LENGTH: usize = 20;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Checksum {
    Sha512(Vec<u8>),
    Sha256(Vec<u8>),
    Sha1(Vec<u8>),
}

impl Checksum {
    pub fn from_integrity(text: &str) -> Option<Self> {
        let mut strongest: Option<Self> = None;
        for token in text.split_whitespace() {
            let (algorithm, digest) = token.split_once('-')?;
            let bytes = STANDARD.decode(digest).ok()?;
            let candidate = match algorithm {
                "sha512" => Self::Sha512(bytes),
                "sha256" => Self::Sha256(bytes),
                "sha1" => Self::Sha1(bytes),
                _ => continue,
            };
            if strongest
                .as_ref()
                .is_none_or(|current| candidate.strength() > current.strength())
            {
                strongest = Some(candidate);
            }
        }
        strongest
    }

    pub fn sha256_hex(text: &str) -> Option<Self> {
        hex_bytes(text).map(Self::Sha256)
    }

    pub fn sha1_hex(text: &str) -> Option<Self> {
        let digest = text.split_whitespace().next()?;
        hex_bytes(digest)
            .filter(|bytes| bytes.len() == SHA1_LENGTH)
            .map(Self::Sha1)
    }

    pub fn matches(&self, bytes: &[u8]) -> bool {
        match self {
            Self::Sha512(expected) => Sha512::digest(bytes).as_slice() == expected.as_slice(),
            Self::Sha256(expected) => Sha256::digest(bytes).as_slice() == expected.as_slice(),
            Self::Sha1(expected) => Sha1::digest(bytes).as_slice() == expected.as_slice(),
        }
    }

    fn strength(&self) -> u8 {
        match self {
            Self::Sha512(_) => 3,
            Self::Sha256(_) => 2,
            Self::Sha1(_) => 1,
        }
    }
}

fn hex_bytes(text: &str) -> Option<Vec<u8>> {
    let text = text.trim();
    if !text.len().is_multiple_of(2) {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(text.get(index..index + 2)?, 16).ok())
        .collect()
}
