use serde::{Deserialize, Serialize};
use typst::{
    doc::{Document, Position},
    model::{Introspector, Location},
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AnnotationBox {
    /// Which page this item is on.
    pub page_ref: u32,
    /// Item width in pt
    pub width: f32,
    /// Item height in pt
    pub height: f32,
    /// Item transform matrix
    pub transform: [f32; 6],
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UrlOpenAction {
    /// The URL to open.
    pub url: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GoToAction {
    /// Which page this destination is on.
    pub page_ref: u32,
    /// x coordinate in pt
    pub x: f32,
    /// y coordinate in pt
    pub y: f32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t", content = "v")]
pub enum LinkAction {
    Url(UrlOpenAction),
    GoTo(GoToAction),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LinkAnnotation {
    /// The annotation box.
    pub annotation_box: AnnotationBox,
    /// The action to perform when the annotation is activated.
    pub action: LinkAction,
}

pub struct AnnotationProcessor {
    introspector: Introspector,
}

impl AnnotationProcessor {
    pub fn new(doc: &Document) -> Self {
        Self {
            introspector: Introspector::new(&doc.pages),
        }
    }

    pub fn process_location(&self, location: Location) -> Position {
        self.introspector.position(location)
    }
}
