use std::{
    borrow::Cow,
    ops::{Deref, Index},
    sync::Arc,
};

#[cfg(feature = "rkyv")]
use rkyv::{Archive, Deserialize as rDeser, Serialize as rSer};

use crate::vector::ir::{ImmutStr, Scalar};

use super::{Module, ModuleView, Page, PageMetadata, SourceMappingNode};

/// Describing
#[derive(Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
#[repr(C)]
pub enum LayoutRegionNode {
    // next indirection
    Indirect(usize),
    // flat page layout
    Pages(Arc<(Vec<PageMetadata>, Vec<Page>)>),
    // source mapping node per page
    SourceMapping(Arc<(Vec<PageMetadata>, Vec<SourceMappingNode>)>),
}

impl LayoutRegionNode {
    pub fn new_pages(pages: Vec<Page>) -> Self {
        Self::Pages(Arc::new((Default::default(), pages)))
    }

    pub fn new_source_mapping(source_mapping: Vec<SourceMappingNode>) -> Self {
        Self::SourceMapping(Arc::new((Default::default(), source_mapping)))
    }

    pub fn pages<'a>(&'a self, module: &'a Module) -> Option<LayoutRegionPagesRAII<'a>> {
        let v = if let Self::Pages(v) = self {
            v
        } else {
            return None;
        };

        if v.0.is_empty() {
            return Some(LayoutRegionPagesRAII {
                module: Cow::Borrowed(ModuleView::new(module)),
                pages: &v.1,
            });
        }

        None
    }

    pub fn source_mapping<'a>(
        &'a self,
        module: &'a Module,
    ) -> Option<LayoutRegionSourceMappingRAII<'a>> {
        let v = if let Self::SourceMapping(v) = self {
            v
        } else {
            return None;
        };

        if v.0.is_empty() {
            return Some(LayoutRegionSourceMappingRAII {
                module: Cow::Borrowed(ModuleView::new(module)),
                source_mapping: &v.1,
            });
        }

        None
    }
}

pub struct LayoutRegionPagesRAII<'a> {
    module: Cow<'a, ModuleView>,
    pages: &'a [Page],
    // todo: chaining module
}

impl<'a> LayoutRegionPagesRAII<'a> {
    pub fn module(&self) -> &Module {
        self.module.as_ref().as_ref()
    }

    pub fn pages(&self) -> &'a [Page] {
        self.pages
    }
}

pub struct LayoutRegionSourceMappingRAII<'a> {
    module: Cow<'a, ModuleView>,
    source_mapping: &'a [SourceMappingNode],
    // todo: chaining module
}

impl<'a> LayoutRegionSourceMappingRAII<'a> {
    pub fn module(&self) -> &Module {
        self.module.as_ref().as_ref()
    }

    pub fn source_mapping(&self) -> &'a [SourceMappingNode] {
        self.source_mapping
    }
}

/// Describing
#[derive(Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct LayoutRegionRepr<T> {
    pub kind: ImmutStr,
    pub layouts: Vec<(T, LayoutRegionNode)>,
}

/// Describing
#[derive(Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum LayoutRegion {
    ByScalar(LayoutRegionRepr<Scalar>),
    ByStr(LayoutRegionRepr<ImmutStr>),
}

impl LayoutRegion {
    pub fn new_single(layout: LayoutRegionNode) -> Self {
        Self::ByScalar(LayoutRegionRepr {
            kind: "_".into(),
            layouts: vec![(Default::default(), layout)],
        })
    }

    pub fn new_by_scalar(kind: ImmutStr, layouts: Vec<(Scalar, LayoutRegionNode)>) -> Self {
        Self::ByScalar(LayoutRegionRepr { kind, layouts })
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::ByScalar(v) => v.layouts.is_empty(),
            Self::ByStr(v) => v.layouts.is_empty(),
        }
    }

    pub fn unwrap_single(&self) -> LayoutRegionNode {
        match self {
            Self::ByScalar(v) => v.layouts.first().unwrap().1.clone(),
            Self::ByStr(v) => v.layouts.first().unwrap().1.clone(),
        }
    }

    pub fn by_scalar(&self) -> Option<&[(Scalar, LayoutRegionNode)]> {
        if let Self::ByScalar(v) = self {
            Some(&v.layouts)
        } else {
            None
        }
    }
}

impl Index<usize> for LayoutRegion {
    type Output = LayoutRegionNode;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Self::ByScalar(v) => &v.layouts[index].1,
            Self::ByStr(v) => &v.layouts[index].1,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct LayoutSourceMapping(pub LayoutRegion);

impl Default for LayoutSourceMapping {
    fn default() -> Self {
        Self::new_single(Default::default())
    }
}

impl LayoutSourceMapping {
    pub fn new_single(source_mapping: Vec<SourceMappingNode>) -> Self {
        Self(LayoutRegion::new_single(
            LayoutRegionNode::new_source_mapping(source_mapping),
        ))
    }
}

impl Deref for LayoutSourceMapping {
    type Target = LayoutRegion;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
