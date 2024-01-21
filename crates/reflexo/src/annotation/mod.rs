use serde::{Deserialize, Serialize};

pub mod link;
pub use self::link::LinkAnnotation;

/// annotation content definition
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AnnotationList {
    pub links: Vec<LinkAnnotation>,
}
