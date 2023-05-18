use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use typst::font::FontInfo;

#[derive(Serialize, Deserialize)]
#[serde(tag = "t", content = "v")]
pub enum CacheCondition {
    Sha256(String),
}

#[derive(Serialize, Deserialize)]
pub struct FontInfoCache {
    pub info: FontInfo,
    pub conditions: Vec<CacheCondition>,
}

impl FontInfoCache {
    pub fn from_data(buffer: &[u8]) -> impl Iterator<Item = Self> + '_ {
        let hash = hex::encode(Sha256::digest(buffer));

        let make_cache = move |fi: FontInfo| -> FontInfoCache {
            FontInfoCache {
                info: fi,
                conditions: vec![CacheCondition::Sha256(hash.clone())],
            }
        };

        FontInfo::iter(buffer).map(make_cache)
    }
}
