use std::collections::HashMap;
use std::sync::Arc;

use base64::Engine;
use ttf_parser::GlyphId;
use typst::font::Font;
use typst::geom::{Abs, Axes, Dir, Point, Ratio, Scalar, Size, Transform};
use typst::image::Image;
use typst_ts_core::typst_affinite_hash;

pub type ImmutStr = Arc<str>;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ImageItem {
    pub image: Image,
    pub size: Size,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StyleNs {
    Fill,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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
pub enum TextStyle {}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PathItem {
    pub d: ImmutStr,
    pub styles: Vec<PathStyle>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum GlyphItem {
    // Failed,
    Raw(Font, GlyphId),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TextShape {
    pub dir: Dir,
    pub ascender: Abs,
    pub upem: Scalar,
    pub ppem: Scalar,
    pub fill: ImmutStr,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TextItemContent {
    pub content: ImmutStr,
    pub glyphs: Vec<(Abs, Abs, GlyphItem)>,
}

#[derive(Debug, Clone)]
pub struct TextItem {
    pub content: Arc<TextItemContent>,
    pub shape: Arc<TextShape>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum TransformItem {
    Matrix(Arc<Transform>),
    Translate(Arc<Axes<Abs>>),
    Scale(Arc<(Ratio, Ratio)>),
    Rotate(Arc<Scalar>),
    Skew(Arc<(Ratio, Ratio)>),
    Clip(Arc<PathItem>),
}

#[derive(Debug, Clone)]
pub struct TransformedItem(pub TransformItem, pub Box<SvgItem>);

#[derive(Debug, Clone)]
pub struct GroupItem(pub Vec<(Point, SvgItem)>);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct DefId(pub u64);

impl DefId {
    pub fn make_relative(&self, id: DefId) -> RelativeDefId {
        RelativeDefId(id.0 as i64 - self.0 as i64)
    }

    pub fn make_absolute(&self, id: RelativeDefId) -> DefId {
        DefId((id.0 + self.0 as i64) as u64)
    }

    pub fn make_relative_ref(&self, abs_ref: AbsoulteRef) -> RelativeRef {
        RelativeRef {
            fingerprint: abs_ref.fingerprint,
            id: self.make_relative(abs_ref.id),
        }
    }

    pub fn make_absolute_ref(&self, rel_ref: RelativeRef) -> AbsoulteRef {
        AbsoulteRef {
            fingerprint: rel_ref.fingerprint,
            id: self.make_absolute(rel_ref.id),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct RelativeDefId(pub i64);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct LinkItem {
    pub href: ImmutStr,
    pub size: Size,
}

#[derive(Debug, Clone)]
pub enum SvgItem {
    Image(ImageItem),
    Link(LinkItem),
    Path(PathItem),
    Text(TextItem),
    Transformed(TransformedItem),
    Group(GroupItem),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FlatTextItemContent {
    pub content: ImmutStr,
    pub glyphs: Arc<[(Abs, Abs, AbsoulteRef)]>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FlatTextItem {
    pub content: Arc<FlatTextItemContent>,
    pub shape: Arc<TextShape>,
}

/// A stable relative reference.
/// See <https://github.com/rust-lang/rust/blob/master/compiler/rustc_hir/src/stable_hash_impls.rs#L22>
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct RelativeRef {
    pub fingerprint: u128,
    pub id: RelativeDefId,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AbsoulteRef {
    pub fingerprint: u128,
    pub id: DefId,
}

impl AbsoulteRef {
    #[comemo::memoize]
    pub fn as_svg_id(&self, prefix: &'static str) -> String {
        let fg =
            base64::engine::general_purpose::STANDARD_NO_PAD.encode(self.fingerprint.to_le_bytes());

        let id = {
            let id = self.id.0.to_le_bytes();
            // truncate zero
            let rev_zero = id.iter().rev().skip_while(|&&b| b == 0).count();
            let id = &id[..rev_zero];
            base64::engine::general_purpose::STANDARD_NO_PAD.encode(id)
        };
        [prefix, &fg, &id].join("")
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TransformedRef(pub TransformItem, pub RelativeRef);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GroupRef(pub Arc<[(Point, RelativeRef)]>);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum FlatSvgItem {
    None,
    Glyph(GlyphItem),
    Image(ImageItem),
    Link(LinkItem),
    Path(PathItem),
    Text(FlatTextItem),
    Item(TransformedRef),
    Group(GroupRef),
}

#[derive(Debug, Default)]
pub struct Module {
    pub glyphs: Vec<GlyphItem>,
    pub items: Vec<FlatSvgItem>,
}

impl Module {
    pub fn get_glyph(&self, id: &AbsoulteRef) -> Option<&GlyphItem> {
        self.glyphs.get(id.id.0 as usize)
    }

    pub fn get_item(&self, id: &AbsoulteRef) -> Option<&FlatSvgItem> {
        self.items.get(id.id.0 as usize)
    }
}

#[derive(Default)]
pub struct ModuleBuilder {
    pub glyph_ids: u64,
    pub glyph_uniquer: HashMap<GlyphItem, AbsoulteRef>,
    pub items: Vec<FlatSvgItem>,
}

impl ModuleBuilder {
    pub fn build_glyph(&mut self, glyph: GlyphItem) -> AbsoulteRef {
        if let Some(id) = self.glyph_uniquer.get(&glyph) {
            return id.clone();
        }

        let id = DefId(self.glyph_ids);
        let abs_ref = AbsoulteRef {
            fingerprint: typst_affinite_hash(&glyph),
            id,
        };
        self.glyph_ids += 1;
        self.glyph_uniquer.insert(glyph, abs_ref.clone());
        abs_ref
    }

    pub fn build(&mut self, item: SvgItem) -> AbsoulteRef {
        let id = DefId(self.items.len() as u64);
        self.items.push(FlatSvgItem::None);

        let resolved_item = match item {
            SvgItem::Image(image) => FlatSvgItem::Image(image),
            SvgItem::Path(path) => FlatSvgItem::Path(path),
            SvgItem::Link(link) => FlatSvgItem::Link(link),
            SvgItem::Text(text) => {
                let glyphs = text
                    .content
                    .glyphs
                    .iter()
                    .cloned()
                    .map(|(offset, advance, glyph)| (offset, advance, self.build_glyph(glyph)))
                    .collect::<Arc<_>>();
                let shape = text.shape.clone();
                let content = text.content.content.clone();
                FlatSvgItem::Text(FlatTextItem {
                    content: Arc::new(FlatTextItemContent { content, glyphs }),
                    shape,
                })
            }
            SvgItem::Transformed(transformed) => {
                let item = &transformed.1;
                let item_id = self.build(*item.clone());
                let transform = transformed.0.clone();

                FlatSvgItem::Item(TransformedRef(transform, id.make_relative_ref(item_id)))
            }
            SvgItem::Group(group) => {
                let items = group
                    .0
                    .iter()
                    .map(|(point, item)| (*point, id.make_relative_ref(self.build(item.clone()))))
                    .collect::<Vec<_>>();
                FlatSvgItem::Group(GroupRef(items.into()))
            }
        };

        let fingerprint = typst_affinite_hash(&resolved_item);
        self.items[id.0 as usize] = resolved_item;
        AbsoulteRef { fingerprint, id }
    }

    pub fn finalize(self) -> Module {
        let mut glyphs = self.glyph_uniquer.into_iter().collect::<Vec<_>>();
        glyphs.sort_by(|(_, a), (_, b)| a.id.0.cmp(&b.id.0));
        Module {
            items: self.items,
            glyphs: glyphs.into_iter().map(|(a, _)| a).collect(),
        }
    }
}

impl SvgItem {
    pub fn flatten(self) -> (AbsoulteRef, Module) {
        let mut builder = ModuleBuilder::default();

        let entry_id = builder.build(self);
        (entry_id, builder.finalize())
    }
}
