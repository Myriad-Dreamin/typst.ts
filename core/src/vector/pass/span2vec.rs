use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use typst::syntax::Span;

use crate::debug_loc::{FileLocation, FlatSourceLocation};
use crate::hash::Fingerprint;

/// Represents a node in the source mapping tree.
pub struct Node {
    pub kind: u8,
    pub source_span: Fingerprint,
    pub children: Vec<Node>,
}

#[derive(Default)]
pub struct Span2VecPass {
    pub should_attach_debug_info: bool,

    region_cnt: AtomicUsize,
    span_to_collect: crossbeam_queue::SegQueue<SourceRegion>,
    // spans: Vec<(usize, Fingerprint, SpanId)>,
}

pub enum SourceNodeKind {
    Page,
    Group,
    GroupRef,
    Char(Span, u16),
    Text(Arc<[(Span, u16)]>),
    Image(Span),
    Shape(Span),
}

pub struct SourceRegion {
    pub region: usize,
    pub idx: u32,
    pub kind: SourceNodeKind,
    pub item: Fingerprint,
}

impl Span2VecPass {
    pub fn set_should_attach_debug_info(&mut self, should_attach_debug_info: bool) {
        self.should_attach_debug_info = should_attach_debug_info;
    }

    pub fn reset(&mut self) {
        // self.region_cnt = 0;
        self.region_cnt
            .store(1, std::sync::atomic::Ordering::SeqCst);
        self.span_to_collect = crossbeam_queue::SegQueue::new();
    }

    pub fn start(&self) -> usize {
        self.region_cnt
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    pub fn push_span(&self, region: SourceRegion) {
        self.span_to_collect.push(region);
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
