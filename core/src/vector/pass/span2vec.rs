use core::fmt;
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
    fn ensure_sorted(&mut self) {
        if !self.is_sorted {
            self.val.sort_by_key(|x| x.0);
            self.is_sorted = true;
        }
    }

    fn get(&mut self, idx: usize) -> Option<&(usize, SourceNodeKind, Fingerprint)> {
        self.ensure_sorted();
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

#[derive(Default)]
struct LazyRegionInfo {
    /// A map from parent region id to a list of children.
    children: HashMap<usize, LazyVec>,
    /// A map from child region id to its parent.
    parents: HashMap<usize, usize>,
    /// A map from span to belonging region ids.
    span_indice: HashMap<Span, Vec<usize>>,
}

impl fmt::Debug for LazyRegionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LazyRegionInfo")
            .field("children_cnt", &self.children.len())
            .field("parents_cnt", &self.parents.len())
            .field("span_indice_cnt", &self.span_indice.len())
            .finish()
    }
}

struct LazySpanInfo {
    /// A lazy ordered tree indexed by a region id.
    ///
    /// When one accesses it with a region id, it will
    /// lazily sort the elements in the region.
    elem_tree: [LazyRegionInfo; SPAN_ROUTING + 1],
}
impl LazySpanInfo {
    fn get_mut(&mut self, doc_region: &usize) -> Option<&mut LazyVec> {
        let idx = doc_region & SPAN_ROUTING;
        self.elem_tree[idx].children.get_mut(doc_region)
    }

    fn get_parent(&self, doc_region: &usize) -> Option<usize> {
        let idx = doc_region & SPAN_ROUTING;
        self.elem_tree[idx].parents.get(doc_region).copied()
    }
}

impl Default for LazySpanInfo {
    fn default() -> Self {
        Self {
            elem_tree: std::array::from_fn(|_| LazyRegionInfo::default()),
        }
    }
}

impl From<LazySpanCollector> for LazySpanInfo {
    fn from(collector: LazySpanCollector) -> Self {
        let val = collector
            .val
            .into_par_iter()
            .map(|e| {
                let mut children = HashMap::new();
                let mut parents = HashMap::new();
                let mut span_indice = HashMap::new();
                let mut insert_span = |span: Span, region: usize| {
                    span_indice
                        .entry(span)
                        .or_insert_with(Vec::new)
                        .push(region);
                };
                for region in e.into_iter() {
                    match &region.kind {
                        SourceNodeKind::Page { region: ch } => {
                            parents.insert(*ch, region.region);
                        }
                        SourceNodeKind::Group { region: ch } => {
                            parents.insert(*ch, region.region);
                        }
                        SourceNodeKind::Char((s, _)) => {
                            insert_span(*s, region.region);
                        }
                        SourceNodeKind::Text(chars) => {
                            for s in chars.iter() {
                                insert_span(s.0, region.region);
                            }
                        }
                        SourceNodeKind::Image(s) => {
                            insert_span(*s, region.region);
                        }
                        SourceNodeKind::Shape(s) => {
                            insert_span(*s, region.region);
                        }
                        SourceNodeKind::Doc => {}
                    }

                    children
                        .entry(region.region)
                        .or_insert_with(|| LazyVec {
                            is_sorted: false,
                            val: Vec::new(),
                        })
                        .val
                        .push((region.idx as usize, region.kind, region.item));
                }

                LazyRegionInfo {
                    children,
                    parents,
                    span_indice,
                }
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        LazySpanInfo { elem_tree: val }
    }
}

#[derive(Default)]
pub struct Span2VecPass {
    pub should_attach_debug_info: bool,

    region_cnt: AtomicUsize,
    pub doc_region: AtomicUsize,

    span_tree: OnceCell<LazySpanInfo>,
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

    pub fn query_cursors(
        &mut self,
        span_offset: SourceSpanOffset,
    ) -> ZResult<Vec<Vec<(u32, u32, String)>>> {
        self.span_tree.get_or_init(|| {
            log::info!("lazy spans are initializing");
            std::mem::take(&mut self.collector).into()
        });

        let span_info = self
            .span_tree
            .get_mut()
            .ok_or_else(|| error_once!("span info not initialized"))?;

        let span = span_offset.span;

        let related_regions: Vec<usize> = span_info
            .elem_tree
            .iter_mut()
            .flat_map(|s| s.span_indice.get(&span))
            .flatten()
            .copied()
            .collect();

        let doc_region = self.doc_region.load(std::sync::atomic::Ordering::SeqCst);

        let mut res = vec![];
        for reg in related_regions {
            let ch = span_info
                .get_mut(&reg)
                .ok_or_else(|| error_once!("not found"))?;
            ch.ensure_sorted();

            for (idx, ch) in ch.val.iter().enumerate() {
                match &ch.1 {
                    SourceNodeKind::Char((s, _)) => {
                        if *s == span {
                            res.push(vec![(reg as u32, idx as u32, "".to_owned())]);
                        }
                    }
                    SourceNodeKind::Text(chars) => {
                        for (s, _) in chars.iter() {
                            if *s == span {
                                res.push(vec![(reg as u32, idx as u32, "".to_owned())]);
                            }
                        }
                    }
                    SourceNodeKind::Image(s) => {
                        if *s == span {
                            res.push(vec![(reg as u32, idx as u32, "".to_owned())]);
                        }
                    }
                    SourceNodeKind::Shape(s) => {
                        if *s == span {
                            res.push(vec![(reg as u32, idx as u32, "".to_owned())]);
                        }
                    }
                    SourceNodeKind::Page { .. }
                    | SourceNodeKind::Group { .. }
                    | SourceNodeKind::Doc => {}
                }
            }
        }

        for r in res.iter_mut() {
            let reg = r.last().unwrap().0 as usize;
            let mut cur = reg;
            while cur != doc_region {
                let par = span_info
                    .get_parent(&doc_region)
                    .ok_or_else(|| error_once!("not found"))?;

                let ch = span_info
                    .get_mut(&reg)
                    .ok_or_else(|| error_once!("not found"))?;
                ch.ensure_sorted();

                let mut found = false;
                for (idx, ch) in ch.val.iter().enumerate() {
                    match &ch.1 {
                        SourceNodeKind::Page { region } | SourceNodeKind::Group { region } => {
                            if *region == cur {
                                r.push((par as u32, idx as u32, ch.2.as_svg_id("")));
                                found = true;
                                break;
                            }
                        }
                        SourceNodeKind::Char(..)
                        | SourceNodeKind::Text(..)
                        | SourceNodeKind::Image(..)
                        | SourceNodeKind::Shape(..)
                        | SourceNodeKind::Doc => {}
                    }
                }

                if !found {
                    break;
                }
                cur = par;
            }

            if cur != doc_region {
                r.clear();
            } else {
                r.reverse();
            }
        }

        res.retain(|x| !x.is_empty());
        Ok(res)
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
