use std::collections::HashMap;
use std::sync::Arc;

use ttf_parser::GlyphId;
use typst::font::Font;
use typst::geom::{Abs, Dir, Point, Scalar, Size, Transform};
use typst::image::Image;

pub type ImmutStr = Arc<str>;

#[derive(Debug, Clone, Hash)]
pub struct ImageItem {
    pub image: Image,
    pub size: Size,
}

#[derive(Debug, Clone)]
pub enum PathStyle {
    Fill(ImmutStr),
    Stroke(ImmutStr),
    StrokeLineCap(ImmutStr),
    StrokeLineJoin(ImmutStr),
    StrokeMitterLimit(Scalar),
    StrokeDashOffset(Abs),
    StrokeDashArray(Arc<[Abs]>),
    StrokeWidth(Abs),
}

#[derive(Debug, Clone)]
pub struct PathItem {
    pub d: String,
    pub styles: Vec<PathStyle>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum GlyphItem {
    // Failed,
    Raw(Font, GlyphId),
}

#[derive(Debug, Clone)]
pub struct TextShape {
    pub dir: Dir,
    pub ppem: f32,
    pub fill: ImmutStr,
}

#[derive(Debug, Clone)]
pub struct TextItem {
    pub content: ImmutStr,
    pub glyphs: Vec<GlyphItem>,
    pub step: Arc<[(Abs, Abs)]>,
    pub shape: Arc<TextShape>,
}

#[derive(Debug, Clone)]
pub enum TransformItem {
    Matrix(Arc<Transform>),
    Clip(Arc<PathItem>),
}

#[derive(Debug, Clone)]
pub struct TransformedItem(pub TransformItem, pub SvgItem);

#[derive(Debug, Clone)]
pub struct GroupItem(pub Arc<[(Point, SvgItem)]>);

#[derive(Debug, Clone, Copy)]
pub struct DefId(pub u64);

impl DefId {
    pub fn make_relative(&self, id: DefId) -> RelativeDefId {
        RelativeDefId(id.0 as i64 - self.0 as i64)
    }

    pub fn make_absolute(&self, id: RelativeDefId) -> DefId {
        DefId((id.0 + self.0 as i64) as u64)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RelativeDefId(pub i64);

#[derive(Debug, Clone)]
pub enum SvgItem {
    Image(Arc<ImageItem>),
    Path(Arc<PathItem>),
    Text(Arc<TextItem>),
    Transformed(Arc<TransformedItem>),
    Group(Arc<GroupItem>),
}

#[derive(Debug, Clone)]
pub struct FlatTextItem {
    pub content: ImmutStr,
    pub glyphs: Vec<DefId>,
    pub step: Arc<[(Abs, Abs)]>,
    pub shape: Arc<TextShape>,
}

#[derive(Debug, Clone)]
pub struct TransformedRef(pub TransformItem, pub RelativeDefId);

#[derive(Debug, Clone)]
pub struct GroupRef(pub Arc<[(Point, RelativeDefId)]>);

#[derive(Debug, Clone)]
pub enum FlatSvgItem {
    None,
    Glyph(Arc<GlyphItem>),
    Image(Arc<ImageItem>),
    Path(Arc<PathItem>),
    Text(Arc<FlatTextItem>),
    Item(Arc<TransformedRef>),
    Group(Arc<GroupRef>),
}

#[derive(Debug, Default)]
pub struct Module {
    pub glyphs: Vec<GlyphItem>,
    pub items: Vec<FlatSvgItem>,
}

impl Module {
    pub fn get_glyph(&self, id: DefId) -> Option<&GlyphItem> {
        self.glyphs.get(id.0 as usize)
    }

    pub fn get_item(&self, id: DefId) -> Option<&FlatSvgItem> {
        self.items.get(id.0 as usize)
    }
}

#[derive(Default)]
pub struct ModuleBuilder {
    pub glyph_ids: u64,
    pub glyph_uniquer: HashMap<GlyphItem, DefId>,
    pub items: Vec<FlatSvgItem>,
}

impl ModuleBuilder {
    pub fn build_glyph(&mut self, glyph: GlyphItem) -> DefId {
        if let Some(id) = self.glyph_uniquer.get(&glyph) {
            return *id;
        }

        let id = DefId(self.glyph_ids);
        self.glyph_ids += 1;
        self.glyph_uniquer.insert(glyph, id);
        id
    }

    pub fn build(&mut self, item: SvgItem) -> DefId {
        let id = DefId(self.items.len() as u64);
        self.items.push(FlatSvgItem::None);

        let resolved_item = match item {
            SvgItem::Image(image) => FlatSvgItem::Image(image),
            SvgItem::Path(path) => FlatSvgItem::Path(path),
            SvgItem::Text(text) => {
                let glyphs = text
                    .glyphs
                    .iter()
                    .cloned()
                    .map(|glyph| self.build_glyph(glyph))
                    .collect::<Vec<_>>();
                let shape = text.shape.clone();
                let content = text.content.clone();
                let step = text.step.clone();
                FlatSvgItem::Text(Arc::new(FlatTextItem {
                    glyphs,
                    shape,
                    content,
                    step,
                }))
            }
            SvgItem::Transformed(transformed) => {
                let item = &transformed.1;
                let item_id = self.build(item.clone());
                let transform = transformed.0.clone();

                FlatSvgItem::Item(Arc::new(TransformedRef(
                    transform,
                    id.make_relative(item_id),
                )))
            }
            SvgItem::Group(group) => {
                let items = group
                    .0
                    .iter()
                    .map(|(point, item)| (*point, id.make_relative(self.build(item.clone()))))
                    .collect::<Vec<_>>();
                FlatSvgItem::Group(Arc::new(GroupRef(items.into())))
            }
        };

        self.items[id.0 as usize] = resolved_item;
        id
    }

    pub fn finalize(self) -> Module {
        let mut glyphs = self.glyph_uniquer.into_iter().collect::<Vec<_>>();
        glyphs.sort_by(|(_, a), (_, b)| a.0.cmp(&b.0));
        Module {
            items: self.items,
            glyphs: glyphs.into_iter().map(|(a, _)| a).collect(),
        }
    }
}

impl SvgItem {
    pub fn flatten(self) -> (DefId, Module) {
        let mut builder = ModuleBuilder::default();

        let entry_id = builder.build(self);
        (entry_id, builder.finalize())
    }
}
