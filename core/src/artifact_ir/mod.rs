use std::collections::HashMap;
use std::mem::size_of;
use std::slice;
use std::str::FromStr;
use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;
use typst::model::Locator;

use crate::artifact::core::BuildInfo;
use crate::font::get_font_coverage_hash;
use crate::typst_affinite_hash;
use crate::FontResolver;

pub mod doc;
use doc::*;

pub mod geom;
use geom::*;

pub mod image;
use image::*;

pub mod core;
use self::core::*;

pub(self) mod alloc;
use self::alloc::*;

pub use crate::artifact::core::ArtifactMeta;
pub type FontInfo = crate::artifact::font::FontInfo;
pub type TypstFontInfo = crate::artifact::font::TypstFontInfo;
pub type TypstFont = crate::artifact::font::TypstFont;
pub type FontCoverage = crate::artifact::font::FontCoverage;
pub type TypstFontFlags = crate::artifact::font::TypstFontFlags;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ArtifactHeader {
    /// Other metadata as json.
    pub metadata: ArtifactMeta,
    /// The page frames.
    pub pages: ItemArray<Frame>,
    /// The region offsets in buffer
    pub offsets: [u64; RegionKind::Length as usize],
}

#[derive(Clone, Debug)]
pub struct Artifact {
    /// memory buffer for ItemRef
    buffer: AlignedBuffer,
    /// Other metadata as json.
    pub metadata: ArtifactMeta,
    /// The page frames.
    pub pages: ItemArray<Frame>,
    /// The region offsets in buffer
    pub offsets: [u64; RegionKind::Length as usize],
}

impl Artifact {
    pub fn new(input: &[u8], header: ArtifactHeader) -> Self {
        let buffer = AlignedBuffer::with_size(input.len());
        Self {
            buffer,
            metadata: header.metadata,
            pages: header.pages,
            offsets: header.offsets,
        }
    }

    #[inline]
    pub fn with_initializer(
        cap: usize,
        initializer: impl FnOnce(&mut [u8]),
        header: ArtifactHeader,
    ) -> Self {
        let mut buffer = AlignedBuffer::with_size(cap);
        let buf_mut = buffer.as_mut_slice();
        initializer(buf_mut);

        Self {
            buffer,
            metadata: header.metadata,
            pages: header.pages,
            offsets: header.offsets,
        }
    }

    #[inline]
    pub fn get_buffer(&self) -> &[u8] {
        self.buffer.as_slice()
    }
}

type GlyphShapeOpaque = [u8; size_of::<GlyphShape>()];

#[repr(u8)]
enum RegionKind {
    General,
    String,
    Bytes,
    ByteSizes,
    StringSizes,
    Paint,
    Glyph,

    Length,
}

#[derive(Default)]
pub struct ArtifactBuilder {
    fonts: Vec<TypstFontInfo>,
    font_map: HashMap<TypstFontInfo, FontRef>,

    glyph_def_id_map: HashMap<GlyphShapeOpaque, GlyphShapeRef>,
    paint_shape_id_map: HashMap<TypstPaint, PaintRef>,
    shared_strings: HashMap<Arc<str>, StringRef>,

    counters: [u32; RegionKind::Length as usize],
    buffers: [Vec<u8>; RegionKind::Length as usize],

    _stat: std::collections::BTreeMap<u32, u32>, // for debug
}

impl ArtifactBuilder {
    fn buffer(&mut self) -> &mut Vec<u8> {
        &mut self.buffers[RegionKind::General as usize]
    }

    fn string_buffer(&mut self) -> &mut Vec<u8> {
        &mut self.buffers[RegionKind::String as usize]
    }

    fn string_size_buffer(&mut self) -> &mut Vec<u8> {
        &mut self.buffers[RegionKind::StringSizes as usize]
    }

    fn bytes_buffer(&mut self) -> &mut Vec<u8> {
        &mut self.buffers[RegionKind::Bytes as usize]
    }

    fn byte_size_buffer(&mut self) -> &mut Vec<u8> {
        &mut self.buffers[RegionKind::ByteSizes as usize]
    }

    fn paint_buffer(&mut self) -> &mut Vec<u8> {
        &mut self.buffers[RegionKind::Paint as usize]
    }

    fn glyph_buffer(&mut self) -> &mut Vec<u8> {
        &mut self.buffers[RegionKind::Glyph as usize]
    }

    pub fn push_item<T: Sized + HasItemRefKind>(&mut self, item: &T) -> ItemRef<T> {
        let idx = item_align_up(self.buffer().len());
        self.buffer().resize(idx, 0);
        unsafe {
            let raw_item =
                slice::from_raw_parts(item as *const T as *const u8, std::mem::size_of::<T>());
            self.buffer().extend_from_slice(raw_item);
            #[cfg(feature = "trace_ir_alloca")]
            self._stat
                .entry(T::ITEM_REF_KIND as u32)
                .and_modify(|e| *e += raw_item.len() as u32);
        }
        ItemRef {
            offset: idx as u32,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn push_bytes(&mut self, s: &[u8]) -> BytesRef {
        let cnt = &mut self.counters[RegionKind::Bytes as usize];
        let id = *cnt;
        *cnt += 1;
        self.byte_size_buffer()
            .extend_from_slice(&(s.len() as u64).to_le_bytes());
        self.bytes_buffer().extend_from_slice(s);
        #[cfg(feature = "trace_ir_alloca")]
        self._stat
            .entry(b'b' as u32)
            .and_modify(|e| *e += s.len() as u32);
        BytesRef { id }
    }

    pub fn push_string(&mut self, s: String) -> StringRef {
        let cnt = &mut self.counters[RegionKind::String as usize];
        let id = *cnt;
        *cnt += 1;
        self.string_size_buffer()
            .extend_from_slice(&(s.as_bytes().len() as u64).to_le_bytes());
        self.string_buffer().extend_from_slice(s.as_bytes());
        #[cfg(feature = "trace_ir_alloca")]
        self._stat
            .entry(b's' as u32)
            .and_modify(|e| *e += s.as_bytes().len() as u32);
        StringRef { id }
    }

    pub fn push_shared_string(&mut self, s: String) -> StringRef {
        if let Some(id) = self.shared_strings.get(s.as_str()) {
            return *id;
        }

        let string_repr = s.as_str().into();
        let id = self.push_string(s);
        self.shared_strings.insert(string_repr, id);
        id
    }

    pub fn push_array<T: Sized + HasItemRefKind>(
        &mut self,
        arr: impl ExactSizeIterator<Item = T>,
    ) -> ItemArray<T> {
        let start_idx = self.buffer().len();
        let arr_len = arr.len();
        for ele in arr {
            self.push_item(&ele);
        }

        #[cfg(feature = "trace_ir_alloca")]
        self._stat
            .entry(T::ITEM_REF_KIND as u32)
            .and_modify(|e| *e += (arr_len * std::mem::size_of::<T>()) as u32);
        ItemArray {
            start: start_idx as u32,
            size: arr_len as u32,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn write_paint(&mut self, paint: &TypstPaint) -> PaintRef {
        if let Some(paint_ref) = self.paint_shape_id_map.get(paint) {
            return *paint_ref;
        }

        let cnt = &mut self.counters[RegionKind::Paint as usize];
        let paint_ref = *cnt as i32;
        *cnt += 1;
        self.paint_shape_id_map.insert(paint.clone(), paint_ref);

        let paint_buffer = self.paint_buffer();

        match paint {
            TypstPaint::Solid(color) => {
                paint_buffer.push(b's');
                match color {
                    TypstColor::Luma(luma_color) => {
                        paint_buffer.push(b'l');
                        paint_buffer.push(luma_color.0);
                    }
                    TypstColor::Rgba(rgba_color) => {
                        paint_buffer.push(b'r');
                        paint_buffer.push(rgba_color.r);
                        paint_buffer.push(rgba_color.g);
                        paint_buffer.push(rgba_color.b);
                        paint_buffer.push(rgba_color.a);
                    }
                    TypstColor::Cmyk(cmyk_color) => {
                        paint_buffer.push(b'c');
                        paint_buffer.push(cmyk_color.c);
                        paint_buffer.push(cmyk_color.m);
                        paint_buffer.push(cmyk_color.y);
                        paint_buffer.push(cmyk_color.k);
                    }
                };
            }
        }

        paint_ref
    }

    pub fn write_optional_paint(&mut self, paint: &Option<TypstPaint>) -> PaintRef {
        match paint {
            Some(paint) => self.write_paint(paint),
            None => -1,
        }
    }

    pub fn write_font(&mut self, font: &TypstFont) -> FontRef {
        if let Some(font) = self.font_map.get(font.info()) {
            return *font;
        }

        if self.fonts.len() >= u32::MAX as usize {
            panic!("too many fonts");
        }

        let font_ref = self.fonts.len() as u32;
        self.font_map.insert(font.info().clone(), font_ref);
        self.fonts.push(font.info().clone());
        font_ref
    }

    pub fn write_glyph_shape(&mut self, glyph: GlyphShape) -> GlyphShapeRef {
        let transmuted =
            unsafe { std::mem::transmute::<GlyphShape, [u8; size_of::<GlyphShape>()]>(glyph) };

        if let Some(glyph_ref) = self.glyph_def_id_map.get(&transmuted) {
            return *glyph_ref;
        }

        let cnt = &mut self.counters[RegionKind::Glyph as usize];
        let glyph_ref = *cnt;
        *cnt += 1;
        self.glyph_buffer().extend_from_slice(&transmuted);
        self.glyph_def_id_map.insert(transmuted, glyph_ref);

        glyph_ref
    }

    pub fn write_glyph(&mut self, glyph: TypstGlyph) -> GlyphItem {
        let glyph_shape_ref = self.write_glyph_shape(GlyphShape {
            id: glyph.id,
            range_width: glyph.range.len() as u16,
            x_advance: glyph.x_advance.into(),
            x_offset: glyph.x_offset.into(),
        });

        GlyphItem {
            shape: glyph_shape_ref,
            span: ((), glyph.span.1), // todo
            range_start: glyph.range.start,
        }
    }

    pub fn write_text_item(&mut self, text: &TypstTextItem) -> TextItem {
        let idx = self.write_font(&text.font);

        let glyphs = text
            .glyphs
            .iter()
            .map(|g| self.write_glyph(g.clone()))
            .collect::<Vec<_>>()
            .into_iter();
        let glyphs = self.push_array(glyphs);

        TextItem {
            font: idx,
            size: text.size.into(),
            fill: self.write_paint(&text.fill),
            lang: self.push_shared_string(text.lang.as_str().to_string()),
            text: self.push_string(text.text.as_str().to_string()),
            glyphs,
        }
    }

    pub fn write_group_item(&mut self, group: &TypstGroupItem) -> GroupItem {
        GroupItem {
            frame: self.write_frame(&group.frame),
            transform: group.transform.into(),
            clips: group.clips,
        }
    }

    pub fn write_image(&mut self, image: &TypstImage) -> Image {
        let data = self.push_bytes(image.data().as_slice());
        let format = self.push_shared_string(
            match image.format() {
                ImageFormat::Raster(r) => match r {
                    RasterFormat::Png => "png",
                    RasterFormat::Jpg => "jpg",
                    RasterFormat::Gif => "gif",
                },
                ImageFormat::Vector(v) => match v {
                    VectorFormat::Svg => "svg",
                },
            }
            .to_string(),
        );
        return Image {
            data,
            data_len: image.data().len() as u64,
            format,
            width: image.width(),
            height: image.height(),
            alt: image.alt().map(|s| self.push_string(s.to_string())),
        };
    }

    pub fn write_dash_pattern<
        OutT,
        OutDT: HasItemRefKind,
        InT: Into<OutT>,
        InDT: Into<OutDT> + Clone,
    >(
        &mut self,
        dash_pattern: TypstDashPattern<InT, InDT>,
    ) -> DashPattern<OutT, OutDT> {
        DashPattern {
            array: self.push_array::<OutDT>(dash_pattern.array.iter().map(|v| (*v).clone().into())),
            phase: dash_pattern.phase.into(),
        }
    }

    pub fn write_stroke(&mut self, stroke: &TypstStroke) -> Stroke {
        Stroke {
            paint: self.write_paint(&stroke.paint),
            thickness: stroke.thickness.into(),
            line_cap: stroke.line_cap.clone().into(),
            line_join: stroke.line_join.clone().into(),
            dash_pattern: stroke
                .dash_pattern
                .clone()
                .map(|d| self.write_dash_pattern(d)),
            miter_limit: stroke.miter_limit.into(),
        }
    }

    pub fn write_path(&mut self, path: &TypstPath) -> Path {
        let items = path.0.iter().map(|item| item.clone().into());

        Path(self.push_array(items))
    }

    pub fn write_geometry(&mut self, geometry: &TypstGeometry) -> Geometry {
        match geometry {
            TypstGeometry::Line(p) => Geometry::Line((*p).into()),
            TypstGeometry::Rect(s) => Geometry::Rect((*s).into()),
            TypstGeometry::Path(p) => Geometry::Path(self.write_path(p)),
        }
    }

    pub fn write_shape(&mut self, shape: &TypstShape) -> Shape {
        Shape {
            geometry: self.write_geometry(&shape.geometry),
            fill: self.write_optional_paint(&shape.fill),
            stroke: shape.stroke.as_ref().map(|s| self.write_stroke(s)),
        }
    }

    pub fn write_frame_item(&mut self, item: &TypstFrameItem) -> FrameItem {
        match item {
            TypstFrameItem::Text(text) => FrameItem::Text(self.write_text_item(text)),
            TypstFrameItem::Group(group) => FrameItem::Group(self.write_group_item(group)),
            TypstFrameItem::Shape(shape, _) => FrameItem::Shape(self.write_shape(shape)),
            TypstFrameItem::Image(image, size, _) => {
                FrameItem::Image(self.write_image(image), (*size).into())
            }
            TypstFrameItem::Meta(meta, size) => match meta {
                TypstMeta::Link(dest) => FrameItem::MetaLink(
                    match dest {
                        TypstDestination::Url(url) => {
                            Destination::Url(self.push_string(url.as_str().to_string()))
                        }
                        TypstDestination::Position(pos) => Destination::Position(Position {
                            page: pos.page,
                            point: pos.point.into(),
                        }),
                        TypstDestination::Location(loc) => {
                            // todo: we have no idea to preserve information about the location
                            Destination::Location(
                                self.push_string(format!("{:?}", typst_affinite_hash(loc))),
                            )
                        }
                    },
                    (*size).into(),
                ),
                TypstMeta::Elem(_) => FrameItem::None,
                TypstMeta::Hide => FrameItem::None,
                TypstMeta::PageNumbering(_) => FrameItem::None,
            },
        }
    }

    pub fn write_frame(&mut self, frame: &TypstFrame) -> Frame {
        let items = frame
            .items()
            .map(|item| {
                let fi = self.write_frame_item(&item.1);
                ItemWithPos {
                    item: self.push_item(&fi),
                    pos: item.0.into(),
                }
            })
            .collect::<Vec<_>>()
            .into_iter();
        Frame {
            size: Axes {
                x: frame.width().into(),
                y: frame.height().into(),
            },
            baseline: if frame.has_baseline() {
                Some(frame.baseline().into())
            } else {
                None
            },
            items: self.push_array(items),
        }
    }

    fn build_buffer(&mut self) -> (AlignedBuffer, [u64; RegionKind::Length as usize]) {
        let mut offsets = [0u64; RegionKind::Length as usize];
        let mut cap = 0;
        for (i, buf) in self.buffers.iter().enumerate() {
            let offset = item_align_up(cap);
            offsets[i] = offset as u64;
            cap = offset + buf.len();
        }

        let mut buffer = AlignedBuffer::with_size(cap);
        let buf_mut = buffer.as_mut_slice();

        for (i, buf) in self.buffers.iter().enumerate() {
            buf_mut[offsets[i] as usize..offsets[i] as usize + buf.len()]
                .copy_from_slice(buf.as_slice());
            if i == self.buffers.len() - 1 {
                buf_mut[offsets[i] as usize + buf.len()..].fill(0);
            } else {
                buf_mut[offsets[i] as usize + buf.len()..offsets[i + 1] as usize].fill(0);
            }
        }

        (buffer, offsets)
    }
}

impl From<&TypstDocument> for Artifact {
    fn from(typst_doc: &TypstDocument) -> Self {
        let mut builder = ArtifactBuilder::default();

        #[cfg(feature = "trace_ir_alloca")]
        {
            assert!(b'b' as u32 >= ItemRefKind::MAX as u32);
            assert!(b's' as u32 >= ItemRefKind::MAX as u32);
            for i in 0..ItemRefKind::MAX as u32 {
                builder._stat.insert(i, 0);
            }
            builder._stat.insert(b'b' as u32, 0);
            builder._stat.insert(b's' as u32, 0);
        }

        let raw_pages = typst_doc
            .pages
            .iter()
            .map(|f| builder.write_frame(f))
            .collect::<Vec<_>>()
            .into_iter();
        let pages = builder.push_array(raw_pages);

        #[cfg(feature = "trace_ir_alloca")]
        println!("_stat: {:?}\n", builder._stat);

        let (buffer, offsets) = builder.build_buffer();
        let metadata = ArtifactMeta {
            build: Some(BuildInfo {
                compiler: "typst-ts-cli".to_string(),
                version: crate::build_info::VERSION.to_string(),
            }),
            fonts: builder
                .fonts
                .into_iter()
                .map(|info| FontInfo {
                    family: info.family,
                    variant: info.variant,
                    flags: info.flags.bits(),
                    coverage: FontCoverage::from_vec(info.coverage.iter().take(1).collect()),
                    coverage_hash: get_font_coverage_hash(&info.coverage),
                    ligatures: vec![],
                })
                .collect(),
            title: typst_doc.title.as_ref().map(|s| s.to_string()),
            author: typst_doc.author.iter().map(|s| s.to_string()).collect(),
        };

        Self {
            buffer,
            metadata,
            pages,
            offsets,
        }
    }
}

pub(crate) struct TypeDocumentParser<'a> {
    sp: Locator<'static>,
    fonts: Vec<TypstFont>,
    paints: Vec<TypstPaint>,
    buffer: &'a [u8],
    glyph_shapes: &'a [GlyphShape],
    strings: Vec<&'a str>,
    bytes: Vec<&'a [u8]>,
}

impl<'a> TypeDocumentParser<'a> {
    pub fn new(buffers: &[&'a [u8]]) -> Self {
        let paint_buffer = &buffers[RegionKind::Paint as usize];
        let mut paints = vec![];
        let mut t = 0;
        while t < paint_buffer.len() {
            match paint_buffer[t] {
                b's' => {
                    t += 1;
                    let color = match paint_buffer[t] {
                        b'l' => {
                            t += 1;
                            let color = paint_buffer[t];
                            TypstColor::Luma(TypstLumaColor(color))
                        }
                        b'r' => {
                            // this should remove extra bound checks
                            let rgba = &paint_buffer[t + 1..t + 5];
                            t += 5;
                            TypstColor::Rgba(TypstRgbaColor::new(
                                rgba[0], rgba[1], rgba[2], rgba[3],
                            ))
                        }
                        b'c' => {
                            let cmyk = &paint_buffer[t + 1..t + 5];
                            t += 5;
                            TypstColor::Cmyk(TypstCmykColor::new(
                                cmyk[0], cmyk[1], cmyk[2], cmyk[3],
                            ))
                        }
                        _ => panic!("Unknown color type in paint region {}", paint_buffer.len()),
                    };
                    paints.push(TypstPaint::Solid(color));
                }
                0 => break,
                _ => panic!("Unknown paint type in paint region {}", paint_buffer.len()),
            }
        }

        let glyph_shapes = buffers[RegionKind::Glyph as usize];
        let glyph_shapes = unsafe {
            std::slice::from_raw_parts(
                glyph_shapes.as_ptr() as *const GlyphShape,
                glyph_shapes.len() / std::mem::size_of::<GlyphShape>(),
            )
        };

        let string_sizes = buffers[RegionKind::StringSizes as usize];
        let (string_sizes, _string_sizes_raii) = get_sizes_array(string_sizes);

        let mut strings = Vec::<&'a str>::new();
        let mut string_offset = 0u64;
        let s = buffers[RegionKind::String as usize];
        for &sz in string_sizes {
            strings.push(
                std::str::from_utf8(&s[string_offset as usize..(string_offset + sz) as usize])
                    .unwrap(),
            );
            string_offset += sz;
        }

        let byte_sizes = buffers[RegionKind::ByteSizes as usize];
        let (byte_sizes, _byte_sizes_raii) = get_sizes_array(byte_sizes);

        let mut bytes = Vec::<&'a [u8]>::new();
        let mut bytes_offset = 0u64;
        let s = buffers[RegionKind::Bytes as usize];
        for &sz in byte_sizes {
            bytes.push(&s[bytes_offset as usize..(bytes_offset + sz) as usize]);
            bytes_offset += sz;
        }

        Self {
            sp: Locator::new(),
            fonts: Vec::new(),
            paints,
            buffer: buffers[RegionKind::General as usize],
            glyph_shapes,
            strings,
            bytes,
        }
    }

    pub fn get_string(&self, string: StringRef) -> &'a str {
        self.strings[string.id as usize]
    }

    pub fn get_bytes(&self, bytes: BytesRef) -> &'a [u8] {
        self.bytes[bytes.id as usize]
    }

    pub fn get_font(&mut self, font: FontRef) -> TypstFont {
        if let Some(f) = self.fonts.get(font as usize) {
            return f.clone();
        }
        panic!("Out of bounds font index {}", font);
    }

    pub fn parse_glyph(&mut self, glyph: &GlyphItem) -> TypstGlyph {
        let shape = &self.glyph_shapes[glyph.shape as usize];
        TypstGlyph {
            id: shape.id,
            x_advance: shape.x_advance.into(),
            x_offset: shape.x_offset.into(),
            span: (TypstSpan::detached(), glyph.span.1),
            range: glyph.range_start..(glyph.range_start + shape.range_width),
        }
    }

    pub fn parse_group_item(&mut self, group: &GroupItem) -> TypstGroupItem {
        TypstGroupItem {
            frame: self.parse_frame(&group.frame),
            transform: group.transform.into(),
            clips: group.clips,
        }
    }

    pub fn parse_text_item(&mut self, text: &TextItem) -> TypstTextItem {
        TypstTextItem {
            font: self.get_font(text.font),
            size: text.size.into(),
            fill: self.paints[text.fill as usize].clone(),
            lang: TypstLang::from_str(self.get_string(text.lang)).unwrap(),
            text: self.get_string(text.text).into(),
            glyphs: text
                .glyphs
                .iter(self.buffer)
                .map(|g| self.parse_glyph(g))
                .collect(),
        }
    }

    pub fn parse_image(&mut self, image: &Image) -> TypstImage {
        TypstImage::new_raw(
            self.get_bytes(image.data).into(),
            match self.get_string(image.format) {
                "png" => ImageFormat::Raster(RasterFormat::Png),
                "jpg" => ImageFormat::Raster(RasterFormat::Jpg),
                "gif" => ImageFormat::Raster(RasterFormat::Gif),
                "svg" => ImageFormat::Vector(VectorFormat::Svg),
                _ => panic!("Unknown image format {}", self.get_string(image.format)),
            },
            TypstAxes {
                x: image.width,
                y: image.height,
            },
            image.alt.map(|s| self.get_string(s).into()),
        )
        .unwrap()
    }

    pub fn parse_location(&mut self, loc: &str) -> Option<TypstLocation> {
        let loc_hash = loc.parse().ok()?;
        Some(self.sp.locate(loc_hash))
    }

    pub fn parse_geometry(&mut self, geometry: &Geometry) -> TypstGeometry {
        match geometry {
            Geometry::Line(p) => TypstGeometry::Line((*p).into()),
            Geometry::Rect(s) => TypstGeometry::Rect((*s).into()),
            Geometry::Path(p) => TypstGeometry::Path({
                let items: Vec<_> =
                    p.0.iter(self.buffer)
                        .map(|item| item.clone().into())
                        .collect();
                TypstPath(items)
            }),
        }
    }

    pub fn parse_dash_pattern<
        OutT,
        OutDT,
        InT: Into<OutT>,
        InDT: Into<OutDT> + Clone + HasItemRefKind,
    >(
        &mut self,
        dash_pattern: DashPattern<InT, InDT>,
    ) -> TypstDashPattern<OutT, OutDT> {
        TypstDashPattern {
            array: dash_pattern
                .array
                .iter(self.buffer)
                .map(|v| (*v).clone().into())
                .collect(),
            phase: dash_pattern.phase.into(),
        }
    }

    pub fn parse_stroke(&mut self, stroke: &Stroke) -> TypstStroke {
        TypstStroke {
            paint: self.paints[stroke.paint as usize].clone(),
            thickness: stroke.thickness.into(),
            line_cap: stroke.line_cap.clone().into(),
            line_join: stroke.line_join.clone().into(),
            dash_pattern: stroke
                .dash_pattern
                .clone()
                .map(|d| self.parse_dash_pattern(d)),
            miter_limit: stroke.miter_limit.into(),
        }
    }

    pub fn parse_shape(&mut self, shape: &Shape) -> TypstShape {
        TypstShape {
            geometry: self.parse_geometry(&shape.geometry),
            fill: if shape.fill < 0 {
                None
            } else {
                Some(self.paints[shape.fill as usize].clone())
            },
            stroke: shape.stroke.as_ref().map(|s| self.parse_stroke(s)),
        }
    }

    pub fn parse_frame_item(&mut self, item: &FrameItem) -> TypstFrameItem {
        match item {
            FrameItem::Group(group) => TypstFrameItem::Group(self.parse_group_item(group)),
            FrameItem::Text(text) => TypstFrameItem::Text(self.parse_text_item(text)),
            FrameItem::Shape(shape) => {
                TypstFrameItem::Shape(self.parse_shape(shape), TypstSpan::detached())
            }
            FrameItem::Image(image, size) => TypstFrameItem::Image(
                self.parse_image(image),
                (*size).into(),
                TypstSpan::detached(),
            ),
            FrameItem::MetaLink(dest, size) => {
                let dest = match dest {
                    Destination::Url(url) => {
                        TypstDestination::Url(TypstEcoString::from(self.get_string(*url)))
                    }
                    Destination::Position(pos) => TypstDestination::Position(TypstPosition {
                        page: pos.page,
                        point: pos.point.into(),
                    }),
                    Destination::Location(loc) => {
                        match self.parse_location(self.get_string(*loc)) {
                            Some(loc) => TypstDestination::Location(loc),
                            None => panic!("Invalid location: {}", self.get_string(*loc)),
                        }
                    }
                };

                TypstFrameItem::Meta(TypstMeta::Link(dest), (*size).into())
            }
            FrameItem::None => panic!("None frame item"),
        }
    }

    pub fn parse_frame(&mut self, frame: &Frame) -> TypstFrame {
        let mut parsed_frame = TypstFrame::new(frame.size.into());
        if let Some(bl) = frame.baseline {
            parsed_frame.set_baseline(bl.into())
        }
        let items = frame.items.iter(self.buffer);

        for ItemWithPos { pos, item } in items {
            let item = item.deref(self.buffer);
            match item {
                FrameItem::None => continue,
                _ => {
                    parsed_frame.push((*pos).into(), self.parse_frame_item(item));
                }
            };
        }

        parsed_frame
    }
}

impl Artifact {
    pub fn to_document<T: FontResolver>(self, font_resolver: &T) -> TypstDocument {
        let buffers = self
            .offsets
            .iter()
            .enumerate()
            .map(|(i, &offset)| {
                let buf = self.buffer.as_slice();
                if i == self.offsets.len() - 1 {
                    &buf[offset as usize..]
                } else {
                    &buf[offset as usize..self.offsets[i + 1] as usize]
                }
            })
            .collect::<Arc<[_]>>();

        let mut builder = TypeDocumentParser::new(buffers.as_ref());
        for font in self.metadata.fonts {
            let font_info = TypstFontInfo {
                family: font.family,
                variant: font.variant,
                flags: TypstFontFlags::from_bits(font.flags).unwrap(),
                coverage: font.coverage,
            };

            // todo: font alternative
            let mut alternative_text = 'c';
            if let Some(codepoint) = font_info.coverage.iter().next() {
                alternative_text = std::char::from_u32(codepoint).unwrap();
            };
            let idx = font_resolver
                .font_book()
                .select_fallback(
                    Some(&font_info),
                    font.variant,
                    &alternative_text.to_string(),
                )
                .unwrap();
            let font = font_resolver.font(idx).unwrap();
            builder.fonts.push(font);
        }

        let pages = self
            .pages
            .iter(builder.buffer)
            .map(|f| builder.parse_frame(f))
            .collect();

        TypstDocument {
            pages,
            title: self.metadata.title.map(TypstEcoString::from),
            author: self
                .metadata
                .author
                .into_iter()
                .map(TypstEcoString::from)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::artifact_ir::*;

    fn build_simple_refs(builder: &mut ArtifactBuilder) -> ItemArray<FrameItem> {
        let lang_str = builder.push_shared_string("en".into());
        let text_src = builder.push_shared_string("W".to_string());
        let glyph = builder.write_glyph(TypstGlyph {
            id: 45,
            x_advance: TypstEm::one(),
            x_offset: TypstEm::one(),
            span: (TypstSpan::detached(), 0),
            range: 0..1,
        });
        let glyphs = builder.push_array(vec![glyph].into_iter());

        let item1 = builder.push_item(&FrameItem::Text(TextItem {
            font: 77,
            size: TypstAbs::zero().into(),
            fill: 1,
            text: text_src,
            lang: lang_str,
            glyphs,
        }));

        let item2 = builder.push_item(&FrameItem::Shape(Shape {
            fill: -1,
            stroke: None,
            geometry: Geometry::Rect(Size {
                x: TypstAbs::zero().into(),
                y: TypstAbs::zero().into(),
            }),
        }));

        let items = vec![item1, item2];

        ItemArray {
            start: items.first().map(|x| x.offset).unwrap(),
            size: items.len() as u32,
            phantom: std::marker::PhantomData,
        }
    }

    #[test]
    fn test_item_ref_array() {
        let mut builder = ArtifactBuilder::default();
        let refs = build_simple_refs(&mut builder);
        assert_eq!(refs.len(), 2);

        let (view_buffer, _) = builder.build_buffer();
        let align_view = &view_buffer.as_slice()[..builder.buffer().len()];

        let mut it = refs.iter(align_view);
        assert_eq!(it.len(), 2);
        if let Some(FrameItem::Text(x)) = it.next() {
            assert_eq!(x.glyphs.len(), 1);
            if let Some(x) = x.glyphs.iter(align_view).next() {
                assert_eq!(x.range_start, 0);
            } else {
                panic!("Expected glyph item");
            }
        } else {
            panic!("Expected text item");
        }

        let item = it.next();
        if let Some(FrameItem::Shape(x)) = item {
            assert_eq!(
                x.geometry,
                Geometry::Rect(Size {
                    x: TypstAbs::zero().into(),
                    y: TypstAbs::zero().into(),
                })
            );
        } else {
            panic!("Expected shape item, got {item:?}");
        }

        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_item_ref_option() {
        let mut builder = ArtifactBuilder::default();

        let raw_item = builder.push_item(&Frame {
            size: Axes {
                x: TypstAbs::zero().into(),
                y: TypstAbs::zero().into(),
            },
            baseline: Some(TypstAbs::raw(1.2).into()),
            items: Default::default(),
        });

        let (view_buffer, _) = builder.build_buffer();
        let align_view = &view_buffer.as_slice()[..builder.buffer().len()];

        let item = raw_item.deref(align_view);
        assert!(matches!(item.baseline, Some(_)));
    }
}
