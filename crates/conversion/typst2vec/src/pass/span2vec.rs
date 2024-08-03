use core::fmt;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use crossbeam_queue::SegQueue;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reflexo::error::prelude::*;
use reflexo::hash::Fingerprint;
use std::sync::OnceLock;

use crate::debug_loc::{
    ElementPoint, FileLocation, FlatSourceLocation, SourceSpan, SourceSpanOffset,
};

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

impl Default for LazyVec {
    fn default() -> Self {
        Self::new()
    }
}

/// A Enum representing [`SourceNodeKind::Text`] or [`SourceNodeKind::Char`].
const SOURCE_MAPPING_TYPE_TEXT: u32 = 0;
/// A Enum representing [`SourceNodeKind::Group`].
const SOURCE_MAPPING_TYPE_GROUP: u32 = 1;
/// A Enum representing [`SourceNodeKind::Image`].
const SOURCE_MAPPING_TYPE_IMAGE: u32 = 2;
/// A Enum representing [`SourceNodeKind::Shape`].
const SOURCE_MAPPING_TYPE_SHAPE: u32 = 3;
/// A Enum representing [`SourceNodeKind::Page`].
const SOURCE_MAPPING_TYPE_PAGE: u32 = 4;
/// A Enum representing internal glyph offset of [`SOURCE_MAPPING_TYPE_TEXT`].
const SOURCE_MAPPING_TYPE_CHAR_INDEX: u32 = 5;

impl LazyVec {
    fn new() -> Self {
        Self {
            is_sorted: false,
            val: Vec::new(),
        }
    }

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

/// The unevaluated span info.
enum RawSpanInfo {
    /// A region to be inserted into the tree.
    Region(SourceRegion),
    /// A belonging relation to children queue to calculate the parents.
    XContainsY { x: usize, y: usize },
}

type RawSpanInfoQueue = crossbeam_queue::SegQueue<RawSpanInfo>;

struct LazySpanCollector {
    val: [RawSpanInfoQueue; SPAN_ROUTING + 1],
}

impl Default for LazySpanCollector {
    fn default() -> Self {
        Self {
            val: std::array::from_fn(|_| crossbeam_queue::SegQueue::new()),
        }
    }
}

impl LazySpanCollector {
    fn reset(&mut self) {
        *self = Self::default();
    }

    fn shard(&self, region: usize) -> &RawSpanInfoQueue {
        // lower bits of region.idx is the index of the queue
        let idx = region & SPAN_ROUTING;
        &self.val[idx]
    }

    fn push(&self, region: SourceRegion) {
        // Inserts XContainsY relation into Y's queue to calculate the parents.
        match &region.kind {
            SourceNodeKind::Page { region: ch } | SourceNodeKind::Group { region: ch } => {
                self.shard(*ch).push(RawSpanInfo::XContainsY {
                    x: region.region,
                    y: *ch,
                });
            }
            SourceNodeKind::Char(..)
            | SourceNodeKind::Text(..)
            | SourceNodeKind::Image(..)
            | SourceNodeKind::Shape(..)
            | SourceNodeKind::Doc => {}
        }

        // Inserts the region into its own queue.
        self.shard(region.region).push(RawSpanInfo::Region(region));
    }
}

#[derive(Default)]
struct LazyRegionInfo {
    /// A map from parent region id to a list of children.
    children: HashMap<usize, LazyVec>,
    /// A map from child region id to its parent.
    parents: HashMap<usize, usize>,
    /// A map from span to belonging region ids.
    span_indice: HashMap<SourceSpan, Vec<usize>>,
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

impl From<SegQueue<RawSpanInfo>> for LazyRegionInfo {
    fn from(value: SegQueue<RawSpanInfo>) -> Self {
        let mut children = HashMap::new();
        let mut parents = HashMap::new();

        let mut span_indice = HashMap::new();
        let mut insert_span = |span: SourceSpan, region: usize| {
            span_indice
                .entry(span)
                .or_insert_with(Vec::new)
                .push(region);
        };

        for i in value.into_iter() {
            match i {
                RawSpanInfo::XContainsY { x, y } => {
                    parents.insert(y, x);
                }
                RawSpanInfo::Region(region) => {
                    match &region.kind {
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
                        SourceNodeKind::Page { .. }
                        | SourceNodeKind::Group { .. }
                        | SourceNodeKind::Doc => {}
                    }

                    children
                        .entry(region.region)
                        .or_insert_with(LazyVec::new)
                        .val
                        .push((region.idx as usize, region.kind, region.item));
                }
            }
        }

        Self {
            children,
            parents,
            span_indice,
        }
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
            .map(LazyRegionInfo::from)
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

    span_tree: OnceLock<LazySpanInfo>,
    collector: LazySpanCollector,
}

#[derive(Debug, Clone)]
pub enum SourceNodeKind {
    Doc,
    Page { region: usize },
    Group { region: usize },
    Char((SourceSpan, u16)),
    Text(Arc<[(SourceSpan, u16)]>),
    Image(SourceSpan),
    Shape(SourceSpan),
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
        self.span_tree = OnceLock::new();
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

    /// Queries the element paths from the given span offset.
    ///
    /// Returns a list of paths, each path is a list of (kind, offset,
    /// fingerprint)
    /// + kind: the type of the element
    /// + offset: the index of the element in its parent
    /// + fingerprint: the fingerprint of the element
    pub fn query_element_paths(
        &mut self,
        span_offset: SourceSpanOffset,
    ) -> ZResult<Vec<Vec<ElementPoint>>> {
        self.span_tree.get_or_init(|| {
            log::info!("lazy spans are initializing");
            std::mem::take(&mut self.collector).into()
        });

        let span_info = self
            .span_tree
            .get_mut()
            .ok_or_else(|| error_once!("span info not initialized"))?;

        let span = span_offset.span;

        // Finds all the regions that contains the span.
        let mut related_regions: Vec<usize> = span_info
            .elem_tree
            .iter_mut()
            .flat_map(|s| s.span_indice.get(&span))
            .flatten()
            .copied()
            .collect();
        related_regions.sort();
        related_regions.dedup();

        // log::info!("pass check related_regions({related_regions:?}");

        let doc_region = *self.doc_region.get_mut();
        if doc_region == 0 {
            return Err(error_once!("doc not initialized"));
        }

        let mut res = vec![];
        for reg in related_regions {
            let ch = span_info
                .get_mut(&reg)
                .ok_or_else(|| error_once!("related region not found", reg: reg))?;
            ch.ensure_sorted();

            for (idx, ch) in ch.val.iter().enumerate() {
                match &ch.1 {
                    SourceNodeKind::Char((s, _)) => {
                        // todo: check upper bound
                        if *s == span {
                            log::info!("pass cursor char({s:?})");
                            res.push(vec![(
                                reg as u32,
                                ElementPoint {
                                    kind: SOURCE_MAPPING_TYPE_TEXT,
                                    index: idx as u32,
                                    fingerprint: "".to_owned(),
                                },
                            )]);
                        }
                    }
                    SourceNodeKind::Text(chars) => {
                        // log::info!("pass cursor check text({chars:?})");
                        for (ch_idx, (s, byte_offset)) in chars.iter().enumerate() {
                            // todo: it may not be monotonic
                            let next = chars.get(ch_idx + 1);
                            let byte_range = if matches!(next, Some((next, _)) if next == s) {
                                (*byte_offset as usize)..(next.unwrap().1 as usize)
                            } else {
                                (*byte_offset as usize)..(usize::MAX)
                            };
                            if *s == span && byte_range.contains(&span_offset.offset) {
                                log::info!("pass cursor text({s:?})");
                                res.push(vec![
                                    (
                                        0u32,
                                        ElementPoint {
                                            kind: SOURCE_MAPPING_TYPE_CHAR_INDEX,
                                            index: ch_idx as u32,
                                            fingerprint: "".to_owned(),
                                        },
                                    ),
                                    (
                                        reg as u32,
                                        ElementPoint {
                                            kind: SOURCE_MAPPING_TYPE_TEXT,
                                            index: idx as u32,
                                            fingerprint: "".to_owned(),
                                        },
                                    ),
                                ]);
                            }
                        }
                    }
                    SourceNodeKind::Image(s) => {
                        if *s == span {
                            log::info!("pass cursor image({s:?})");
                            res.push(vec![(
                                reg as u32,
                                ElementPoint {
                                    kind: SOURCE_MAPPING_TYPE_IMAGE,
                                    index: idx as u32,
                                    fingerprint: "".to_owned(),
                                },
                            )]);
                        }
                    }
                    SourceNodeKind::Shape(s) => {
                        if *s == span {
                            log::info!("pass cursor shape({s:?})");
                            res.push(vec![(
                                reg as u32,
                                ElementPoint {
                                    kind: SOURCE_MAPPING_TYPE_SHAPE,
                                    index: idx as u32,
                                    fingerprint: "".to_owned(),
                                },
                            )]);
                        }
                    }
                    SourceNodeKind::Page { .. }
                    | SourceNodeKind::Group { .. }
                    | SourceNodeKind::Doc => {}
                }
            }
        }

        log::info!("pass found candidates({res:?}), with root: {doc_region}");
        for r in res.iter_mut() {
            let reg = r.last().unwrap().0 as usize;
            let mut cur = reg;
            while cur != doc_region {
                let par = span_info
                    .get_parent(&cur)
                    .ok_or_else(|| error_once!("parent not found", cur: cur))?;

                let ch = span_info
                    .get_mut(&par)
                    .ok_or_else(|| error_once!("region children not found", reg: par))?;
                ch.ensure_sorted();

                // log::info!("found parent({cur:?}) -> ({par:?})");

                let mut found = false;
                for (idx, ch) in ch.val.iter().enumerate() {
                    match &ch.1 {
                        SourceNodeKind::Page { region } => {
                            // log::info!("pass find check page({region:?})");
                            if *region == cur {
                                r.push((
                                    par as u32,
                                    ElementPoint {
                                        kind: SOURCE_MAPPING_TYPE_PAGE,
                                        index: idx as u32,
                                        fingerprint: "".to_owned(),
                                    },
                                ));
                                found = true;
                                break;
                            }
                        }
                        SourceNodeKind::Group { region } => {
                            // log::info!("pass find check group({region:?})");
                            if *region == cur {
                                r.push((
                                    par as u32,
                                    ElementPoint {
                                        kind: SOURCE_MAPPING_TYPE_GROUP,
                                        index: idx as u32,
                                        fingerprint: "".to_owned(),
                                    },
                                ));
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
                log::info!("drop candidate({reg:?})");
                r.clear();
            } else {
                r.reverse();
            }
        }

        res.retain(|x| !x.is_empty());
        Ok(res
            .into_iter()
            .map(|x| x.into_iter().map(|y| y.1).collect())
            .collect())
    }

    pub fn query(
        &mut self,
        path: &[ElementPoint],
    ) -> ZResult<Option<(SourceSpanOffset, SourceSpanOffset)>> {
        self.span_tree.get_or_init(|| {
            log::info!("lazy spans are initializing");
            std::mem::take(&mut self.collector).into()
        });

        let doc_region = *self.doc_region.get_mut();
        if doc_region == 0 {
            return Err(error_once!("doc not initialized"));
        }

        let span_info = self
            .span_tree
            .get_mut()
            .ok_or_else(|| error_once!("span info not initialized"))?;

        let mut d = span_info
            .get_mut(&doc_region)
            .ok_or_else(|| error_once!("not found"))?;

        log::info!("pass check remote path({path:?})");

        let mut candidate: Option<(SourceSpanOffset, SourceSpanOffset)> = None;
        let mut in_text_indice: Option<Arc<[(SourceSpan, u16)]>> = None;
        for ElementPoint {
            kind: remote_kind,
            index: idx,
            fingerprint: fg,
        } in path
        {
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
                    let is_attached = |x: (SourceSpan, u16)| x.0 != SourceSpan::detached();
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
                    let is_attached = |x: &&(SourceSpan, u16)| x.0 != SourceSpan::detached();
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
