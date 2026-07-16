//! Bidirectional synchronization between Typst source files and paged output.
//!
//! The lookup strategy follows Tinymist's source/document jump implementation,
//! but keeps the mapping on the incremental compiler session so callers can
//! associate it with the exact successfully compiled document revision.

use std::path::Path;
use std::sync::Arc;

use reflexo_typst::compat::syntax::LinkedNodeExt;
use reflexo_typst::{
    debug_loc::SourceSpanOffset, vfs::WorkspaceResolver, BrowserCompilerFeat, TypstPagedDocument,
    WorldComputeGraph,
};
use serde::{Deserialize, Serialize};
use typst::layout::{Abs, Frame, FrameItem, Point, Size};
use typst::syntax::{LinkedNode, Source, Span, SyntaxKind, VirtualRoot};
use typst::visualize::Geometry;
use typst::World;

/// A point in a compiled paged document.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentPosition {
    /// Zero-based page offset.
    pub page_offset: usize,
    /// Horizontal page coordinate in Typst points.
    pub x: f32,
    /// Vertical page coordinate in Typst points.
    pub y: f32,
}

/// A source location resolved from a compiled document.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLocation {
    /// Virtual source path, including the leading slash.
    pub path: String,
    /// Package specification when the source belongs to a package.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// UTF-8 byte offset in the source file.
    pub byte_offset: usize,
}

/// The mapping state associated with one successful incremental compilation.
pub struct SourceMappingState {
    graph: Arc<WorldComputeGraph<BrowserCompilerFeat>>,
    document: Arc<TypstPagedDocument>,
}

impl SourceMappingState {
    pub fn new(
        graph: Arc<WorldComputeGraph<BrowserCompilerFeat>>,
        document: Arc<TypstPagedDocument>,
    ) -> Self {
        Self { graph, document }
    }

    /// Find rendered positions corresponding to a UTF-8 source byte offset.
    pub fn source_to_document(&self, path: &str, byte_offset: usize) -> Vec<DocumentPosition> {
        let world = &self.graph.snap.world;
        let Some(source_id) = world.id_for_path(Path::new(path)) else {
            return Vec::new();
        };
        let Ok(source) = world.source(source_id) else {
            return Vec::new();
        };
        if byte_offset > source.text().len() || !source.text().is_char_boundary(byte_offset) {
            return Vec::new();
        }

        jump_from_cursor(&self.document, &source, byte_offset)
    }

    /// Find the source location under a rendered page coordinate.
    pub fn document_to_source(&self, page_offset: usize, x: f32, y: f32) -> Option<SourceLocation> {
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        let page = self.document.pages().get(page_offset)?;
        let world = &self.graph.snap.world;
        let click = Point::new(Abs::pt(f64::from(x)), Abs::pt(f64::from(y)));
        let span = jump_from_click(world, &page.frame, click)?;
        resolve_source_location(world, span)
    }
}

fn resolve_source_location(
    world: &impl World,
    location: SourceSpanOffset,
) -> Option<SourceLocation> {
    let id = location.span.id()?;
    let source = world.source(id).ok()?;
    let range = source.find(location.span)?.range();
    let byte_offset = range.start.saturating_add(location.offset).min(range.end);
    let package = match id.root() {
        VirtualRoot::Package(spec) if WorkspaceResolver::is_package_file(id) => {
            Some(spec.to_string())
        }
        VirtualRoot::Project | VirtualRoot::Package(_) => None,
    };

    Some(SourceLocation {
        path: id.vpath().get_with_slash().to_string(),
        package,
        byte_offset,
    })
}

/// Find the source span under a physical position in a page frame.
fn jump_from_click(world: &impl World, frame: &Frame, click: Point) -> Option<SourceSpanOffset> {
    // Preserve link activation semantics instead of turning links into source
    // navigation targets.
    for (pos, item) in frame.items() {
        if let FrameItem::Link(_, size) = item {
            if is_in_rect(*pos, *size, click) {
                return None;
            }
        }
    }

    for &(mut pos, ref item) in frame.items().rev() {
        match item {
            FrameItem::Group(group) => {
                // Typst's current preview jump implementation handles group
                // translations. General transformed groups remain follow-up
                // work in Tinymist as well.
                if let Some(span) = jump_from_click(world, &group.frame, click - pos) {
                    return Some(span);
                }
            }
            FrameItem::Text(text) => {
                for glyph in &text.glyphs {
                    let width = glyph.x_advance.at(text.size);
                    if is_in_rect(
                        Point::new(pos.x, pos.y - text.size),
                        Size::new(width, text.size),
                        click,
                    ) {
                        let (span, span_offset) = glyph.span;
                        let mut offset = usize::from(span_offset);
                        let source = world.source(span.id()?).ok()?;
                        let node = source.find(span)?;
                        if matches!(node.kind(), SyntaxKind::Text | SyntaxKind::MathText)
                            && click.x - pos.x > width / 2.0
                        {
                            offset = offset.saturating_add(glyph.range().len());
                        }
                        return Some(SourceSpanOffset { span, offset });
                    }
                    pos.x += width;
                }
            }
            FrameItem::Shape(shape, span) => {
                let Geometry::Rect(size) = shape.geometry else {
                    continue;
                };
                if is_in_rect(pos, size, click) {
                    return Some((*span).into());
                }
            }
            FrameItem::Image(_, size, span) if is_in_rect(pos, *size, click) => {
                return Some((*span).into());
            }
            FrameItem::Link(..) | FrameItem::Tag(..) | FrameItem::Image(..) => {}
        }
    }

    None
}

/// Find rendered positions for a source cursor.
fn jump_from_cursor(
    document: &TypstPagedDocument,
    source: &Source,
    cursor: usize,
) -> Vec<DocumentPosition> {
    let Some(node) = LinkedNode::new(source.root()).leaf_at_compat(cursor) else {
        return Vec::new();
    };
    if !matches!(node.kind(), SyntaxKind::Text | SyntaxKind::MathText) {
        return Vec::new();
    }
    let span = node.span();

    let mut positions = Vec::new();
    let mut closest_page = 0;
    let mut closest_point = Point::default();
    let mut closest_distance = u64::MAX;

    for (page_offset, page) in document.pages().iter().enumerate() {
        let mut page_distance = closest_distance;
        if let Some(point) =
            find_in_frame(&page.frame, span, &mut page_distance, &mut closest_point)
        {
            positions.push(DocumentPosition {
                page_offset,
                x: point.x.to_pt() as f32,
                y: point.y.to_pt() as f32,
            });
        }
        if page_distance != closest_distance {
            closest_page = page_offset;
            closest_distance = page_distance;
        }
    }

    if positions.is_empty() && closest_distance != u64::MAX {
        positions.push(DocumentPosition {
            page_offset: closest_page,
            x: closest_point.x.to_pt() as f32,
            y: closest_point.y.to_pt() as f32,
        });
    }

    positions
}

fn find_in_frame(
    frame: &Frame,
    span: Span,
    min_distance: &mut u64,
    result: &mut Point,
) -> Option<Point> {
    for &(mut pos, ref item) in frame.items() {
        if let FrameItem::Group(group) = item {
            if let Some(point) = find_in_frame(&group.frame, span, min_distance, result) {
                return Some(point + pos);
            }
        }

        if let FrameItem::Text(text) = item {
            for glyph in &text.glyphs {
                if glyph.span.0 == span {
                    return Some(pos);
                }

                if glyph.span.0.id() == span.id() {
                    let distance = glyph
                        .span
                        .0
                        .into_raw()
                        .get()
                        .abs_diff(span.into_raw().get());
                    if distance < *min_distance {
                        *min_distance = distance;
                        *result = pos;
                    }
                }
                pos.x += glyph.x_advance.at(text.size);
            }
        }
    }

    None
}

fn is_in_rect(pos: Point, size: Size, click: Point) -> bool {
    pos.x <= click.x && pos.x + size.x >= click.x && pos.y <= click.y && pos.y + size.y >= click.y
}
