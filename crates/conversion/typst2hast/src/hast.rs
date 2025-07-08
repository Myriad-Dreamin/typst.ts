//! See [@types/hast.](https://github.com/DefinitelyTyped/DefinitelyTyped/blob/70305194cccca1d648922c0130eb62740b301fd9/types/hast/index.d.ts)

use ecow::EcoString;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum HastElementContent {
    Root(HastRoot),
    Text(HastText),
    Comment(HastText),
    Element(Box<HastElement>),
}

impl Default for HastElementContent {
    fn default() -> Self {
        HastElementContent::Root(HastRoot::default())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct HastRoot {
    pub children: Vec<HastElementContent>,
    // data
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HastText {
    pub value: EcoString,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HastElement {
    pub tag_name: EcoString,
    pub properties: HastElementProperties,
    pub children: Vec<HastElementContent>,
    // todo: content
    // todo: data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HastElementData>,
}

pub type HastElementProperties = std::collections::BTreeMap<EcoString, EcoString>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HastElementData {
    pub hash: Option<EcoString>,
}
