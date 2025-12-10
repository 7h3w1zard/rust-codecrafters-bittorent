pub use hashes::Hashes;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

/// A torrent file (metainfo file) contains a bencoded dictionary.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Torrent {
    /// URL to a "tracker", that keeps track of peers participating in the sharing of a torrent.
    pub announce: String,

    /// A dictionary with keys:
    pub info: Info,
}

impl Torrent {
    pub fn info_hash(&self) -> [u8; 20] {
        let info_encoded = serde_bencode::to_bytes(&self.info).expect("re-encode info section should be fine)");
        let mut hasher = Sha1::new();
        hasher.update(&info_encoded);
        hasher.finalize().try_into().expect("GenericArray<_, 20>")
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Info {
    /// suggested name to save the file / directory as
    pub name: String,

    /// number of bytes in each piece
    #[serde(rename = "piece length")]
    pub piece_length: usize,

    /// concatenated SHA-1 hashes of each piece
    pub pieces: Hashes,

    #[serde(flatten)]
    pub keys: Keys,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Keys {
    SingleFile {
        /// size of the file in bytes, for single-file torrents
        length: usize,
    },
    MultiFile {
        files: Vec<File>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File {
    length: usize,
    path: Vec<String>,
}

mod hashes {
    use serde::ser::{Serialize, Serializer};
    use serde::{
        Deserialize, Deserializer,
        de::{self, Visitor},
    };
    use std::fmt;

    #[derive(Debug, Clone)]
    pub struct Hashes(pub Vec<[u8; 20]>);
    struct HashesVisitor;

    impl<'de> Visitor<'de> for HashesVisitor {
        type Value = Hashes;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a byte string whose length is a multiple of 20")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v.len() % 20 != 0 {
                return Err(E::custom(format!("length is {}", v.len())));
            }

            Ok(Hashes(
                v.chunks_exact(20)
                    .map(|slice_20| slice_20.try_into().expect("guaranteed to be length 20"))
                    .collect(),
            ))
        }
    }

    impl<'de> Deserialize<'de> for Hashes {
        fn deserialize<D>(deserializer: D) -> Result<Hashes, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_bytes(HashesVisitor)
        }
    }

    impl Serialize for Hashes {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let single_slice = self.0.concat();
            serializer.serialize_bytes(&single_slice)
        }
    }
}
