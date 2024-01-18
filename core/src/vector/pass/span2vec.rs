use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use once_cell::sync::OnceCell;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use typst::syntax::Span;

use crate::debug_loc::{FileLocation, FlatSourceLocation, SourceSpanOffset};
use crate::error::prelude::ZResult;
use crate::error::prelude::*;
use crate::hash::Fingerprint;

/// Represents a node in the source mapping tree.
pub struct Node {
    pub kind: u8,
    pub source_span: Fingerprint,
    pub children: Vec<Node>,
}

type SrcVec = Vec<(usize, SourceNodeKind, Fingerprint)>;

#[derive(Debug)]
struct LazyVec {
    is_sorted: bool,
    val: SrcVec,
}

impl LazyVec {
    fn get(&mut self, idx: usize) -> Option<&(usize, SourceNodeKind, Fingerprint)> {
        if !self.is_sorted {
            self.val.sort_by_key(|x| x.0);
            self.is_sorted = true;
        }
        self.val.get(idx)
    }
}

const SPAN_ROUTING: usize = 63;

struct LazySpanCollector {
    val: [crossbeam_queue::SegQueue<SourceRegion>; SPAN_ROUTING + 1],
}

impl Default for LazySpanCollector {
    fn default() -> Self {
        Self {
            val: std::array::from_fn(|_| crossbeam_queue::SegQueue::new()),
        }
    }
}

impl LazySpanCollector {
    fn push(&self, region: SourceRegion) {
        // lower bits of region.idx is the index of the queue
        let idx = region.region & SPAN_ROUTING;
        self.val[idx].push(region);
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}

struct LazySpanTree {
    val: [HashMap<usize, LazyVec>; SPAN_ROUTING + 1],
}
impl LazySpanTree {
    fn get_mut(&mut self, doc_region: &usize) -> Option<&mut LazyVec> {
        let idx = doc_region & SPAN_ROUTING;
        self.val[idx].get_mut(doc_region)
    }
}

impl Default for LazySpanTree {
    fn default() -> Self {
        Self {
            val: std::array::from_fn(|_| HashMap::new()),
        }
    }
}

impl From<LazySpanCollector> for LazySpanTree {
    fn from(collector: LazySpanCollector) -> Self {
        let val = collector
            .val
            .into_par_iter()
            .map(|e| {
                let mut res = HashMap::new();
                for region in e.into_iter() {
                    res.entry(region.region)
                        .or_insert_with(|| LazyVec {
                            is_sorted: false,
                            val: Vec::new(),
                        })
                        .val
                        .push((region.idx as usize, region.kind, region.item));
                }
                res
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        LazySpanTree { val }
    }
}

#[derive(Default)]
pub struct Span2VecPass {
    pub should_attach_debug_info: bool,

    region_cnt: AtomicUsize,
    pub doc_region: AtomicUsize,

    span_tree: OnceCell<LazySpanTree>,
    collector: LazySpanCollector,
}

#[derive(Debug, Clone)]
pub enum SourceNodeKind {
    Doc,
    Page { region: usize },
    Group { region: usize },
    Char((Span, u16)),
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
        assert!(
            (SPAN_ROUTING + 1).is_power_of_two(),
            "SPAN_ROUTING + 1 must be power of 2"
        );
        self.should_attach_debug_info = should_attach_debug_info;
    }

    pub fn reset(&mut self) {
        // self.region_cnt = 0;
        self.region_cnt
            .store(1, std::sync::atomic::Ordering::SeqCst);
        self.collector.reset();
        self.span_tree = once_cell::sync::OnceCell::new();
        self.doc_region
            .store(0, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn start(&self) -> usize {
        self.region_cnt
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    pub fn push_span(&self, region: SourceRegion) {
        self.collector.push(region);
    }

    pub fn query(
        &mut self,
        path: &[(u32, u32, String)],
    ) -> ZResult<Option<(SourceSpanOffset, SourceSpanOffset)>> {
        self.span_tree.get_or_init(|| {
            log::info!("lazy spans are initializing");
            std::mem::take(&mut self.collector).into()
        });

        let doc_region = self.doc_region.load(std::sync::atomic::Ordering::SeqCst);

        if doc_region == 0 {
            return Err(error_once!("doc not initialized"));
        }

        let span_info = self
            .span_tree
            .get_mut()
            .ok_or_else(|| error_once!("span info not initialized"))?;

        const SOURCE_MAPPING_TYPE_TEXT: u32 = 0;
        const SOURCE_MAPPING_TYPE_GROUP: u32 = 1;
        const SOURCE_MAPPING_TYPE_IMAGE: u32 = 2;
        const SOURCE_MAPPING_TYPE_SHAPE: u32 = 3;
        const SOURCE_MAPPING_TYPE_PAGE: u32 = 4;
        const SOURCE_MAPPING_TYPE_CHAR_INDEX: u32 = 5;

        let mut d = span_info
            .get_mut(&doc_region)
            .ok_or_else(|| error_once!("not found"))?;

        log::info!("pass check remote path({path:?})");

        let mut candidate: Option<(SourceSpanOffset, SourceSpanOffset)> = None;
        let mut in_text_indice: Option<Arc<[(Span, u16)]>> = None;
        for (remote_kind, idx, fg) in path {
            // Special case for char index
            if SOURCE_MAPPING_TYPE_CHAR_INDEX == *remote_kind {
                log::info!(
                    "pass check remote_char_index({remote_kind}, {idx}) => {:?}",
                    in_text_indice
                );
                let char_idx = *idx as usize;
                if let Some(chars) = in_text_indice.as_ref() {
                    let ch = chars.as_ref().get(char_idx);
                    let Some(ch) = ch else {
                        continue;
                    };
                    if !ch.0.is_detached() {
                        candidate = Some(((*ch).into(), (*ch).into()));
                    }
                }
                continue;
            }

            // Overwrite the previous text indice
            in_text_indice = None;
            // Find the child node
            let ch = d
                .get(*idx as usize)
                .ok_or_else(|| error_once!("not found"))?;
            if !fg.is_empty() && !fg.as_str().contains(&ch.2.as_svg_id("")) {
                return Err(error_once!("fg not match", fg: fg, ch: ch.2.as_svg_id("")));
            }

            log::info!("pass check remote({remote_kind}, {idx}) => {:?}", ch.1);

            match (*remote_kind, ch.1.clone()) {
                (SOURCE_MAPPING_TYPE_PAGE, SourceNodeKind::Page { region }) => {
                    d = span_info.get_mut(&region).ok_or_else(
                        || error_once!("region not found", at: remote_kind, at_idx: idx),
                    )?;
                }
                (SOURCE_MAPPING_TYPE_GROUP, SourceNodeKind::Group { region }) => {
                    d = span_info.get_mut(&region).ok_or_else(
                        || error_once!("region not found", at: remote_kind, at_idx: idx),
                    )?;
                }
                (SOURCE_MAPPING_TYPE_TEXT, SourceNodeKind::Char(ch)) => {
                    let is_attached = |x: (Span, u16)| x.0 != Span::detached();
                    let st = is_attached(ch).then_some(ch);
                    candidate = st.map(From::from).zip(st.map(From::from));

                    // Generates a dynamic text indice here.
                    //
                    // We don't wrap it as creating `SourceNodeKind::Char`, because
                    // it will be used rarely.
                    //
                    // This strategy would help us to reduce the time and memory
                    // usage.
                    in_text_indice = Some(Arc::new([ch]));
                }
                (SOURCE_MAPPING_TYPE_TEXT, SourceNodeKind::Text(chars)) => {
                    let is_attached = |x: &&(Span, u16)| x.0 != Span::detached();
                    let st = chars.iter().find(is_attached).copied();
                    let ed = chars.iter().rev().find(is_attached).copied();
                    candidate = st.map(From::from).zip(ed.map(From::from));
                    in_text_indice = Some(chars.clone());
                }
                (SOURCE_MAPPING_TYPE_IMAGE, SourceNodeKind::Image(s)) => {
                    candidate = Some((s.into(), s.into()));
                }
                (SOURCE_MAPPING_TYPE_SHAPE, SourceNodeKind::Shape(s)) => {
                    candidate = Some((s.into(), s.into()));
                }
                _ => {
                    return Err(error_once!(
                        "invalid/mismatch node type",
                        ty: remote_kind,
                        actual: format!("{:?}", ch.1),
                        parent: format!("{:?}", candidate),
                        child_idx_in_parent: idx,
                    ))
                }
            }
        }

        Ok(candidate)
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
