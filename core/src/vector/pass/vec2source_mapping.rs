use std::collections::HashMap;

use crate::debug_loc::{FileLocation, FlatSourceLocation};
use crate::{hash::Fingerprint, vector::vm::RenderVm};

/// Represents a node in the source mapping tree.
pub struct Node {
    pub kind: u8,
    pub source_span: Fingerprint,
    pub children: Vec<Node>,
}

pub struct Vec2SourceMapping {}

impl Vec2SourceMapping {
    pub fn item<'a>(ctx: &impl RenderVm<'a>, i: &Fingerprint) -> Node {
        let _ = ctx;
        let _ = ctx.get_item(i).unwrap();

        Node {
            kind: 0,
            source_span: Fingerprint::from_u128(0),
            children: vec![],
        }
    }
}

pub struct SourceInfo {
    pub files: Vec<FileLocation>,
    pub spans: HashMap<Fingerprint, FlatSourceLocation>,
}

pub struct SourceInfoCollector {}

impl SourceInfoCollector {
    pub fn finallize(self) -> SourceInfo {
        todo!()
    }
    pub fn intern(_i: &Node) {
        todo!()
    }
}
