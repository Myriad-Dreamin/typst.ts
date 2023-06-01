use std::{num::NonZeroUsize, vec};

use typst_ts_core::{artifact::doc::Frame, error::prelude::*, Artifact, ArtifactMeta};
use wasm_bindgen::prelude::*;

pub struct ArtifactJsBuilder {}

/// Destructures a JS `[key, value]` pair into a tuple of [`Deserializer`]s.
pub(crate) fn convert_pair(pair: JsValue) -> (JsValue, JsValue) {
    let pair = pair.unchecked_into::<js_sys::Array>();
    (pair.get(0), pair.get(1))
}

impl ArtifactJsBuilder {
    pub fn to_bool(&self, field: &str, val: &JsValue) -> ZResult<bool> {
        val.as_bool()
            .ok_or_else(|| error_once!("expected bool", field: field, val: format!("{:?}", val)))
    }

    pub fn to_f64(&self, field: &str, val: &JsValue) -> ZResult<f64> {
        val.as_f64()
            .ok_or_else(|| error_once!("expected f64", field: field, val: format!("{:?}", val)))
    }

    pub fn to_string(&self, field: &str, val: &JsValue) -> ZResult<String> {
        val.as_string()
            .ok_or_else(|| error_once!("expected string", field: field, val: format!("{:?}", val)))
    }

    pub fn into_array(&self, field: &str, val: JsValue) -> ZResult<js_sys::Array> {
        val.clone().dyn_into().map_err(error_once_map!(
            "expected array",
            field: field,
            val: format!("{:?}", val)
        ))
    }

    pub fn into_entries(&self, field: &str, val: JsValue) -> ZResult<js_sys::Array> {
        Ok(js_sys::Object::entries(val.dyn_ref().ok_or_else(|| {
            error_once!(
                "expect entries for object",
                field: field,
                val: format!("{:?}", val)
            )
        })?))
    }

    pub fn parse_tv(&self, field: &str, val: JsValue) -> ZResult<(String, Option<JsValue>)> {
        let mut t = String::default();
        let mut sub: Option<JsValue> = None;

        for (k, v) in self.into_entries(field, val)?.iter().map(convert_pair) {
            let k = self.to_string(field, &k)?;
            match k.as_str() {
                "t" => {
                    t = self.to_string(field, &v)?;
                }
                "v" => {
                    sub = Some(v);
                }
                _ => {
                    return Err(error_once!(
                        "unknown kv for",
                        field: field,
                        key: k,
                        val: format!("{:?}", v)
                    ))
                }
            }
        }

        Ok((t, sub))
    }

    pub fn parse_tv_expected(&self, field: &str, val: JsValue) -> ZResult<(String, JsValue)> {
        let (t, sub) = self.parse_tv(field, val.clone())?;

        Ok((
            t,
            sub.ok_or_else(|| {
                error_once!(
                    "expected value for {}",
                    field: field,
                    val: format!("{:?}", val)
                )
            })?,
        ))
    }

    fn parse_font_variant(&self, val: JsValue) -> ZResult<typst::font::FontVariant> {
        let mut variant = typst::font::FontVariant::default();
        for (k, v) in self
            .into_entries("font_variant", val)?
            .iter()
            .map(convert_pair)
        {
            let k = self.to_string("font_variant", &k)?;
            match k.as_str() {
                "weight" => {
                    variant.weight = typst::font::FontWeight::from_number(
                        self.to_f64("font_variant.weight", &v)? as u16,
                    );
                }
                "style" => {
                    let v = self.to_string("font_variant.style", &v)?;
                    match v.as_str() {
                        "normal" => variant.style = typst::font::FontStyle::Normal,
                        "italic" => variant.style = typst::font::FontStyle::Italic,
                        "oblique" => variant.style = typst::font::FontStyle::Oblique,
                        _ => {
                            return Err(error_once!(
                                "font_variant.unknown_style",
                                v: v,
                                val: format!("{:?}", v)
                            ));
                        }
                    }
                }
                "stretch" => {
                    variant.stretch = typst::font::FontStretch::from_ratio(
                        typst::geom::Ratio::new(self.to_f64("font_variant.stretch", &v)?),
                    );
                }
                _ => {
                    return Err(error_once!(
                        "font_variant.unknown_key",
                        k: k,
                        val: format!("{:?}", v)
                    ));
                }
            }
        }
        Ok(variant)
    }

    pub(crate) fn parse_font_info(
        &self,
        val: JsValue,
    ) -> ZResult<typst_ts_core::artifact::font::FontInfo> {
        let mut family = String::default();
        let mut variant = typst::font::FontVariant::default();
        let mut flags = typst::font::FontFlags::empty();
        let mut coverage = None;
        let mut ligatures = None;
        let mut coverage_hash = String::default();

        for (k, v) in self
            .into_entries("font_info", val)?
            .iter()
            .map(convert_pair)
        {
            let k = self.to_string("font_info", &k)?;
            match k.as_str() {
                "family" => {
                    family = self.to_string("font_info.family", &v)?;
                }
                "variant" => {
                    variant = self.parse_font_variant(v)?;
                }
                "flags" => {
                    flags = typst::font::FontFlags::from_bits(
                        self.to_f64("font_info.flags", &v)? as u32
                    )
                    .ok_or_else(|| {
                        error_once!(
                            "invalid flags",
                            field: "font_info.flags",
                            val: format!("{:?}", v)
                        )
                    })?;
                }
                "coverage" => {
                    // todo: from codepoint
                    coverage = serde_wasm_bindgen::from_value(v)
                        .map_err(map_string_err("font_info.coverage"))?;
                }
                "coverage_hash" => {
                    coverage_hash = self.to_string("font_info.coverage_hash", &v)?;
                }
                "ligatures" => {
                    ligatures = Some(
                        self.into_array("font_info.ligatures", v.clone())?
                            .iter()
                            .map(convert_pair)
                            .map(|(g, s)| {
                                Ok((
                                    self.to_f64("font_info.ligature_glyph", &g)? as u16,
                                    self.to_string("font_info.ligature_str", &s)?,
                                ))
                            })
                            .collect::<ZResult<_>>()?,
                    );
                }
                _ => {
                    return Err(error_once!(
                        "font_info.unknown_key",
                        k: k,
                        val: format!("{:?}", v)
                    ));
                }
            }
        }
        Ok(typst_ts_core::artifact::font::FontInfo {
            family,
            variant,
            flags: flags.bits(),
            coverage: coverage.unwrap_or_else(|| typst::font::Coverage::from_vec(vec![])),
            coverage_hash,
            ligatures: ligatures.unwrap_or_default(),
        })
    }

    pub fn parse_point(&self, val: JsValue) -> ZResult<typst_ts_core::artifact::geom::Point> {
        let mut x = typst::geom::Abs::default();
        let mut y = typst::geom::Abs::default();

        for (k, v) in self.into_entries("point", val)?.iter().map(convert_pair) {
            let k = self.to_string("point", &k)?;
            match k.as_str() {
                "x" => {
                    x = typst::geom::Abs::raw(self.to_f64("point.x", &v)?);
                }
                "y" => {
                    y = typst::geom::Abs::raw(self.to_f64("point.y", &v)?);
                }

                _ => {
                    return Err(error_once!(
                        "point.unknown_key",
                        k: k,
                        val: format!("{:?}", v)
                    ));
                }
            }
        }

        Ok(typst_ts_core::artifact::geom::Point {
            x: x.into(),
            y: y.into(),
        })
    }

    pub fn parse_transform(
        &self,
        val: JsValue,
    ) -> ZResult<typst_ts_core::artifact::geom::Transform> {
        let mut sx = typst::geom::Ratio::default();
        let mut ky = typst::geom::Ratio::default();
        let mut kx = typst::geom::Ratio::default();
        let mut sy = typst::geom::Ratio::default();
        let mut tx = typst::geom::Abs::default();
        let mut ty = typst::geom::Abs::default();

        for (k, v) in self
            .into_entries("transform", val)?
            .iter()
            .map(convert_pair)
        {
            let k = self.to_string("transform", &k)?;
            match k.as_str() {
                "sx" => {
                    sx = typst::geom::Ratio::new(self.to_f64("transform.sx", &v)?);
                }
                "ky" => {
                    ky = typst::geom::Ratio::new(self.to_f64("transform.ky", &v)?);
                }
                "kx" => {
                    kx = typst::geom::Ratio::new(self.to_f64("transform.kx", &v)?);
                }
                "sy" => {
                    sy = typst::geom::Ratio::new(self.to_f64("transform.sy", &v)?);
                }
                "tx" => {
                    tx = typst::geom::Abs::raw(self.to_f64("transform.tx", &v)?);
                }
                "ty" => {
                    ty = typst::geom::Abs::raw(self.to_f64("transform.ty", &v)?);
                }

                _ => {
                    return Err(error_once!(
                        "transform.unknown_key",
                        k: k,
                        val: format!("{:?}", v)
                    ));
                }
            }
        }

        Ok(typst_ts_core::artifact::geom::Transform {
            sx: sx.into(),
            ky: ky.into(),
            kx: kx.into(),
            sy: sy.into(),
            tx: tx.into(),
            ty: ty.into(),
        })
    }

    pub fn parse_frame_group(
        &self,
        val: JsValue,
    ) -> ZResult<typst_ts_core::artifact::doc::GroupItem> {
        let mut frame: Option<Frame> = None;
        let mut transform: Option<typst_ts_core::artifact::geom::Transform> = None;
        let mut clips = false;

        for (k, v) in self
            .into_entries("frame_group", val)?
            .iter()
            .map(convert_pair)
        {
            let k = self.to_string("frame_group", &k)?;
            match k.as_str() {
                "frame" => {
                    frame = Some(self.parse_frame(v)?);
                }
                "transform" => {
                    transform = Some(self.parse_transform(v)?);
                }
                "clips" => {
                    clips = self.to_bool("frame_group.clips", &v)?;
                }

                _ => {
                    return Err(error_once!(
                        "frame_group.unknown_key",
                        k: k,
                        val: format!("{:?}", v)
                    ));
                }
            }
        }

        Ok(typst_ts_core::artifact::doc::GroupItem {
            frame: frame.ok_or_else(|| error_once!("frame_group.frame.not_found"))?,
            transform: transform.ok_or_else(|| error_once!("frame_group.transform.not_found"))?,
            clips,
        })
    }

    pub fn parse_glyph(&self, val: JsValue) -> ZResult<typst_ts_core::artifact::doc::Glyph> {
        let mut id = None;
        let mut x_advance = None;
        let mut x_offset = None;
        let mut span = None;
        let mut range = None;

        for (k, v) in self.into_entries("glyph", val)?.iter().map(convert_pair) {
            let k = self.to_string("glyph", &k)?;
            match k.as_str() {
                "id" => {
                    id = Some(self.to_f64("glyph.id", &v)? as u16);
                }
                "x_advance" => {
                    x_advance =
                        Some(typst::geom::Em::new(self.to_f64("glyph.x_advance", &v)?).into());
                }
                "x_offset" => {
                    x_offset =
                        Some(typst::geom::Em::new(self.to_f64("glyph.x_offset", &v)?).into());
                }
                "span" => {
                    // todo: span self.to_f64(v)? as u16
                    let (v0, v1) = convert_pair(v);
                    span = Some((
                        (self.to_f64("glyph.span0", &v0))? as u16,
                        (self.to_f64("glyph.span1", &v1))? as u16,
                    ));
                }
                "range" => {
                    let (st, ed) = convert_pair(v);
                    range = Some((
                        self.to_f64("glyph.range0", &st)? as u16,
                        self.to_f64("glyph.range1", &ed)? as u16,
                    ));
                }

                _ => {
                    return Err(error_once!(
                        "glyph.unknown_key",
                        k: k,
                        val: format!("{:?}", v)
                    ));
                }
            }
        }

        Ok(typst_ts_core::artifact::doc::Glyph {
            id: id.ok_or_else(|| error_once!("glyph.id.not_found"))?,
            x_advance: x_advance.ok_or_else(|| error_once!("glyph.x_advance.not_found"))?,
            x_offset: x_offset.ok_or_else(|| error_once!("glyph.x_offset.not_found"))?,
            span: span.ok_or_else(|| error_once!("glyph.span.not_found"))?,
            range: range.ok_or_else(|| error_once!("glyph.range.not_found"))?,
        })
    }

    pub fn parse_frame_text(
        &self,
        val: JsValue,
    ) -> ZResult<typst_ts_core::artifact::doc::TextItem> {
        let mut font = None;
        let mut size = None;
        let mut fill = None;
        let mut lang = None;
        let mut text = None;
        let mut glyphs = vec![];

        for (k, v) in self
            .into_entries("frame_text", val)?
            .iter()
            .map(convert_pair)
        {
            let k = self.to_string("frame_text", &k)?;
            match k.as_str() {
                "font" => {
                    font = Some(self.to_f64("frame_text.font", &v)? as u32);
                }
                "size" => {
                    size = Some(typst::geom::Abs::raw(self.to_f64("frame_text.size", &v)?).into());
                }
                "fill" => {
                    fill = Some(self.parse_paint(v)?);
                }
                "lang" => {
                    lang = Some(self.to_string("frame_text.lang", &v)?);
                }
                "text" => {
                    text = Some(self.to_string("frame_text.text", &v)?);
                }
                "glyphs" => {
                    for arr in self.into_array("frame.glyphs", v)?.iter() {
                        glyphs.push(self.parse_glyph(arr)?);
                    }
                }

                _ => {
                    return Err(error_once!(
                        "frame_text.unknown_key",
                        k: k,
                        val: format!("{:?}", v)
                    ));
                }
            }
        }

        Ok(typst_ts_core::artifact::doc::TextItem {
            font: font.ok_or_else(|| error_once!("frame_text.font.not_found"))?,
            size: size.ok_or_else(|| error_once!("frame_text.size.not_found"))?,
            fill: fill.ok_or_else(|| error_once!("frame_text.fill.not_found"))?,
            lang: lang.ok_or_else(|| error_once!("frame_text.lang.not_found"))?,
            text: text.ok_or_else(|| error_once!("frame_text.text.not_found"))?,
            glyphs,
        })
    }

    pub fn parse_size(&self, val: JsValue) -> ZResult<typst_ts_core::artifact::geom::Size> {
        let mut x = typst::geom::Abs::default();
        let mut y = typst::geom::Abs::default();

        for (k, v) in self.into_entries("size", val)?.iter().map(convert_pair) {
            let k = self.to_string("size", &k)?;
            match k.as_str() {
                "x" => {
                    x = typst::geom::Abs::raw(self.to_f64("size.x", &v)?);
                }
                "y" => {
                    y = typst::geom::Abs::raw(self.to_f64("size.y", &v)?);
                }

                _ => {
                    return Err(error_once!(
                        "size.unknown_key",
                        k: k,
                        val: format!("{:?}", v)
                    ));
                }
            }
        }

        Ok(typst_ts_core::artifact::geom::Axes {
            x: x.into(),
            y: y.into(),
        })
    }

    pub fn parse_path_item(
        &self,
        val: JsValue,
    ) -> ZResult<typst_ts_core::artifact::geom::PathItem> {
        let (t, sub) = self.parse_tv("path_item", val)?;
        match t.as_str() {
            "MoveTo" => Ok(typst_ts_core::artifact::geom::PathItem::MoveTo(
                self.parse_point(sub.ok_or_else(|| error_once!("path_item.MoveTo.v"))?)?,
            )),
            "LineTo" => Ok(typst_ts_core::artifact::geom::PathItem::LineTo(
                self.parse_point(sub.ok_or_else(|| error_once!("path_item.LineTo.v"))?)?,
            )),
            "CubicTo" => Ok({
                let sub = sub.ok_or_else(|| error_once!("path_item.CubicTo.v"))?;
                let a_sub = self.into_array("path_item.CubicTo", sub)?;
                typst_ts_core::artifact::geom::PathItem::CubicTo(
                    self.parse_point(a_sub.get(0))?,
                    self.parse_point(a_sub.get(1))?,
                    self.parse_point(a_sub.get(2))?,
                )
            }),
            "ClosePath" => Ok(typst_ts_core::artifact::geom::PathItem::ClosePath),
            _ => Err(error_once!(
                "path_item.unknown_type",
                t: t,
                val: format!("{:?}", sub)
            )),
        }
    }

    pub fn parse_geometry(&self, val: JsValue) -> ZResult<typst_ts_core::artifact::geom::Geometry> {
        let (t, sub) = self.parse_tv_expected("geometry", val)?;
        match t.as_str() {
            "Line" => Ok(typst_ts_core::artifact::geom::Geometry::Line(
                self.parse_point(sub)?,
            )),
            "Rect" => Ok(typst_ts_core::artifact::geom::Geometry::Rect(
                self.parse_size(sub)?,
            )),
            "Path" => Ok(typst_ts_core::artifact::geom::Geometry::Path({
                let mut res = vec![];

                for arr in self.into_array("geometry.Path", sub)?.iter() {
                    res.push(self.parse_path_item(arr)?);
                }
                typst_ts_core::artifact::geom::Path(res)
            })),
            _ => Err(error_once!(
                "geometry.unknown_type",
                t: t,
                val: format!("{:?}", sub)
            )),
        }
    }

    pub fn parse_rgba_color(
        &self,
        val: JsValue,
    ) -> ZResult<typst_ts_core::artifact::geom::RgbaColor> {
        let mut r = None;
        let mut g = None;
        let mut b = None;
        let mut a = None;

        for (k, v) in self
            .into_entries("rgba_color", val)?
            .iter()
            .map(convert_pair)
        {
            let k = self.to_string("rgba_color", &k)?;
            match k.as_str() {
                "r" => {
                    r = Some(self.to_f64("rgba_color.r", &v)? as u8);
                }
                "g" => {
                    g = Some(self.to_f64("rgba_color.g", &v)? as u8);
                }
                "b" => {
                    b = Some(self.to_f64("rgba_color.b", &v)? as u8);
                }
                "a" => {
                    a = Some(self.to_f64("rgba_color.a", &v)? as u8);
                }

                _ => {
                    return Err(error_once!(
                        "rgba_color.unknown_key",
                        k: k,
                        val: format!("{:?}", v)
                    ));
                }
            }
        }

        Ok(typst_ts_core::artifact::geom::RgbaColor {
            r: r.ok_or_else(|| error_once!("rgba_color.r.not_found"))?,
            g: g.ok_or_else(|| error_once!("rgba_color.g.not_found"))?,
            b: b.ok_or_else(|| error_once!("rgba_color.b.not_found"))?,
            a: a.ok_or_else(|| error_once!("rgba_color.a.not_found"))?,
        })
    }

    pub fn parse_cmyk_color(
        &self,
        val: JsValue,
    ) -> ZResult<typst_ts_core::artifact::geom::CmykColor> {
        let mut c = None;
        let mut m = None;
        let mut y = None;
        let mut kk = None;

        for (k, v) in self
            .into_entries("cmyk_color", val)?
            .iter()
            .map(convert_pair)
        {
            let k = self.to_string("cmyk_color", &k)?;
            match k.as_str() {
                "c" => {
                    c = Some(self.to_f64("cmyk_color.c", &v)? as u8);
                }
                "m" => {
                    m = Some(self.to_f64("cmyk_color.m", &v)? as u8);
                }
                "y" => {
                    y = Some(self.to_f64("cmyk_color.y", &v)? as u8);
                }
                "k" => {
                    kk = Some(self.to_f64("cmyk_color.k", &v)? as u8);
                }

                _ => {
                    return Err(error_once!(
                        "cmyk_color.unknown_key",
                        k: k,
                        val: format!("{:?}", v)
                    ));
                }
            }
        }

        Ok(typst_ts_core::artifact::geom::CmykColor {
            c: c.ok_or_else(|| error_once!("cmyk_color.c.not_found"))?,
            m: m.ok_or_else(|| error_once!("cmyk_color.m.not_found"))?,
            y: y.ok_or_else(|| error_once!("cmyk_color.y.not_found"))?,
            k: kk.ok_or_else(|| error_once!("cmyk_color.k.not_found"))?,
        })
    }

    pub fn parse_color(&self, val: JsValue) -> ZResult<typst_ts_core::artifact::geom::Color> {
        // /// An 8-bit luma color.
        // Luma(LumaColor),
        // /// An 8-bit RGBA color.
        // Rgba(RgbaColor),
        // /// An 8-bit CMYK color.
        // Cmyk(CmykColor),
        let (t, sub) = self.parse_tv_expected("color", val)?;
        match t.as_str() {
            "Luma" => Ok(typst_ts_core::artifact::geom::Color::Luma(
                typst_ts_core::artifact::geom::LumaColor(self.to_f64("color.Luma", &sub)? as u8),
            )),
            "Rgba" => Ok(typst_ts_core::artifact::geom::Color::Rgba(
                self.parse_rgba_color(sub)?,
            )),
            "Cmyk" => Ok(typst_ts_core::artifact::geom::Color::Cmyk(
                self.parse_cmyk_color(sub)?,
            )),
            _ => Err(error_once!(
                "color.unknown_type",
                t: t,
                val: format!("{:?}", sub)
            )),
        }
    }

    pub fn parse_paint(&self, val: JsValue) -> ZResult<typst_ts_core::artifact::geom::Paint> {
        let (t, sub) = self.parse_tv_expected("paint", val)?;
        match t.as_str() {
            "Solid" => Ok(typst_ts_core::artifact::geom::Paint::Solid(
                self.parse_color(sub)?,
            )),
            _ => Err(error_once!(
                "paint.unknown_type",
                t: t,
                val: format!("{:?}", sub)
            )),
        }
    }

    pub fn parse_line_cap(&self, val: JsValue) -> ZResult<typst_ts_core::artifact::geom::LineCap> {
        match self.to_string("line_cap", &val)?.as_str() {
            "Butt" => Ok(typst_ts_core::artifact::geom::LineCap::Butt),
            "Round" => Ok(typst_ts_core::artifact::geom::LineCap::Round),
            "Square" => Ok(typst_ts_core::artifact::geom::LineCap::Square),
            _ => Err(error_once!(
                "line_cap.unknown_type",
                val: format!("{:?}", val)
            )),
        }
    }

    pub fn parse_line_join(
        &self,
        val: JsValue,
    ) -> ZResult<typst_ts_core::artifact::geom::LineJoin> {
        match self.to_string("line_join", &val)?.as_str() {
            "Miter" => Ok(typst_ts_core::artifact::geom::LineJoin::Miter),
            "Round" => Ok(typst_ts_core::artifact::geom::LineJoin::Round),
            "Bevel" => Ok(typst_ts_core::artifact::geom::LineJoin::Bevel),
            _ => Err(error_once!(
                "line_join.unknown_type",
                val: format!("{:?}", val)
            )),
        }
    }

    pub fn parse_dash_pattern(
        &self,
        val: JsValue,
    ) -> ZResult<
        typst_ts_core::artifact::geom::DashPattern<
            typst_ts_core::artifact::geom::Abs,
            typst_ts_core::artifact::geom::Abs,
        >,
    > {
        let mut array = vec![];
        let mut phase = None;

        for (k, v) in self
            .into_entries("dash_pattern", val)?
            .iter()
            .map(convert_pair)
        {
            let k = self.to_string("dash_pattern", &k)?;
            match k.as_str() {
                "array" => {
                    for val in self.into_array("dash_pattern.array", v)?.iter() {
                        array.push(
                            typst::geom::Abs::raw(self.to_f64("dash_pattern.array", &val)?).into(),
                        );
                    }
                }
                "phase" => {
                    phase =
                        Some(typst::geom::Abs::raw(self.to_f64("dash_pattern.phase", &v)?).into());
                }

                _ => {
                    return Err(error_once!(
                        "dash_pattern.unknown_key",
                        k: k,
                        val: format!("{:?}", v)
                    ));
                }
            }
        }

        Ok(typst_ts_core::artifact::geom::DashPattern {
            array,
            phase: phase.ok_or_else(|| error_once!("dash_pattern.phase.not_found"))?,
        })
    }

    pub fn parse_stroke(&self, val: JsValue) -> ZResult<typst_ts_core::artifact::geom::Stroke> {
        let mut paint = None;
        let mut thickness = None;
        let mut line_cap = None;
        let mut line_join = None;
        let mut dash_pattern = None;
        let mut miter_limit = None;

        for (k, v) in self.into_entries("stroke", val)?.iter().map(convert_pair) {
            let k = self.to_string("stroke", &k)?;
            match k.as_str() {
                "paint" => {
                    paint = Some(self.parse_paint(v)?);
                }
                "thickness" => {
                    thickness =
                        Some(typst::geom::Abs::raw(self.to_f64("stroke.thickness", &v)?).into());
                }
                "line_cap" => {
                    line_cap = Some(self.parse_line_cap(v)?);
                }
                "line_join" => {
                    line_join = Some(self.parse_line_join(v)?);
                }
                "dash_pattern" => {
                    dash_pattern = if v.is_null() {
                        None
                    } else {
                        Some(self.parse_dash_pattern(v)?)
                    };
                }
                "miter_limit" => {
                    miter_limit = Some(typst_ts_core::artifact::geom::Scalar(
                        self.to_f64("stroke.miter_limit", &v)?,
                    ));
                }
                _ => {
                    return Err(error_once!(
                        "stroke.unknown_key",
                        k: k,
                        val: format!("{:?}", v)
                    ));
                }
            }
        }

        Ok(typst_ts_core::artifact::geom::Stroke {
            paint: paint.ok_or_else(|| error_once!("stroke.paint.not_found"))?,
            thickness: thickness.ok_or_else(|| error_once!("stroke.thickness.not_found"))?,
            line_cap: line_cap.ok_or_else(|| error_once!("stroke.line_cap.not_found"))?,
            line_join: line_join.ok_or_else(|| error_once!("stroke.line_join.not_found"))?,
            dash_pattern,
            miter_limit: miter_limit.ok_or_else(|| error_once!("stroke.miter_limit.not_found"))?,
        })
    }

    pub fn parse_frame_shape(&self, val: JsValue) -> ZResult<typst_ts_core::artifact::geom::Shape> {
        let mut geometry = None;
        let mut fill = None;
        let mut stroke = None;

        for (k, v) in self
            .into_entries("frame_shape", val)?
            .iter()
            .map(convert_pair)
        {
            let k = self.to_string("frame_shape", &k)?;
            match k.as_str() {
                "geometry" => {
                    geometry = Some(self.parse_geometry(v)?);
                }
                "fill" => {
                    fill = if !v.is_null() {
                        Some(self.parse_paint(v)?)
                    } else {
                        None
                    };
                }
                "stroke" => {
                    stroke = if !v.is_null() {
                        Some(self.parse_stroke(v)?)
                    } else {
                        None
                    };
                }
                _ => {
                    return Err(error_once!(
                        "shape.unknown_key",
                        k: k,
                        val: format!("{:?}", v)
                    ));
                }
            }
        }

        Ok(typst_ts_core::artifact::geom::Shape {
            geometry: geometry.ok_or_else(|| error_once!("frame_shape.geometry.not_found"))?,
            fill,
            stroke,
        })
    }

    pub fn parse_image(&self, val: JsValue) -> ZResult<typst_ts_core::artifact::image::Image> {
        let mut data: Option<Vec<u8>> = None;
        let mut format = None;
        let mut width = None;
        let mut height = None;
        let mut alt = None;

        for (k, v) in self.into_entries("image", val)?.iter().map(convert_pair) {
            let k = self.to_string("image", &k)?;
            match k.as_str() {
                "data" => {
                    let data_raw = typst_ts_core::artifact::image::Image::decode_data(
                        &self.to_string("image.data.base64", &v)?,
                    )
                    .map_err(error_once_map!("image.data.base64.decode_error"))?;
                    data = Some(data_raw);
                }
                "format" => {
                    format = Some(self.to_string("image.format", &v)?);
                }
                "width" => {
                    width = Some(self.to_f64("image.width", &v)? as u32);
                }
                "height" => {
                    height = Some(self.to_f64("image.height", &v)? as u32);
                }
                "alt" => {
                    alt = if !v.is_null() {
                        Some(self.to_string("image.alt", &v)?)
                    } else {
                        None
                    };
                }
                _ => {
                    return Err(error_once!(
                        "image.unknown_key",
                        k: k,
                        val: format!("{:?}", v)
                    ));
                }
            }
        }

        Ok(typst_ts_core::artifact::image::Image {
            data: data.ok_or_else(|| error_once!("image.data.not_found"))?,
            format: format.ok_or_else(|| error_once!("image.format.not_found"))?,
            width: width.ok_or_else(|| error_once!("image.width.not_found"))?,
            height: height.ok_or_else(|| error_once!("image.height.not_found"))?,
            alt,
        })
    }

    pub fn parse_position(&self, val: JsValue) -> ZResult<typst_ts_core::artifact::doc::Position> {
        // /// The page, starting at 1.
        // pub page: NonZeroUsize,
        // /// The exact coordinates on the page (from the top left, as usual).
        // pub point: Point,

        let mut page = None;
        let mut point = None;

        for (k, v) in self.into_entries("position", val)?.iter().map(convert_pair) {
            let k = self.to_string("position", &k)?;
            match k.as_str() {
                "page" => {
                    page = Some(
                        NonZeroUsize::new(self.to_f64("position.page", &v)? as usize).ok_or_else(
                            || error_once!("position.page.invalid_value", val: format!("{:?}", v)),
                        )?,
                    );
                }
                "point" => {
                    point = Some(self.parse_point(v)?);
                }
                _ => {
                    return Err(error_once!(
                        "position.unknown_key",
                        k: k,
                        val: format!("{:?}", v)
                    ));
                }
            }
        }

        Ok(typst_ts_core::artifact::doc::Position {
            page: page.ok_or_else(|| error_once!("position.page.not_found"))?,
            point: point.ok_or_else(|| error_once!("position.point.not_found"))?,
        })
    }

    pub fn parse_destination(
        &self,
        val: JsValue,
    ) -> ZResult<typst_ts_core::artifact::doc::Destination> {
        let (t, sub) = self.parse_tv_expected("destination", val)?;

        match t.as_str() {
            "Url" => Ok(typst_ts_core::artifact::doc::Destination::Url(
                self.to_string("destination.Url", &sub)?,
            )),
            "Position" => Ok(typst_ts_core::artifact::doc::Destination::Position(
                self.parse_position(sub)?,
            )),
            "Location" => Ok(typst_ts_core::artifact::doc::Destination::Location(
                self.to_string("destination.Location", &sub)?,
            )),
            _ => Err(error_once!(
                "destination.unknown_type",
                t: t,
                val: format!("{:?}", sub)
            )),
        }
    }

    pub fn parse_frame_item(
        &self,
        val: JsValue,
    ) -> ZResult<typst_ts_core::artifact::doc::FrameItem> {
        let (t, sub) = self.parse_tv("frame_item", val)?;
        match t.as_str() {
            "Group" => Ok(typst_ts_core::artifact::doc::FrameItem::Group(
                self.parse_frame_group(sub.ok_or_else(|| error_once!("frame_item.group"))?)?,
            )),
            "Text" => Ok(typst_ts_core::artifact::doc::FrameItem::Text(
                self.parse_frame_text(sub.ok_or_else(|| error_once!("frame_item.text"))?)?,
            )),
            "Shape" => Ok(typst_ts_core::artifact::doc::FrameItem::Shape(
                self.parse_frame_shape(sub.ok_or_else(|| error_once!("frame_item.shape"))?)?,
            )),
            "Image" => {
                let sub = sub.ok_or_else(|| error_once!("frame_item.image"))?;
                let arr = self.into_array("frame_item", sub)?;
                Ok(typst_ts_core::artifact::doc::FrameItem::Image(
                    self.parse_image(arr.get(0))?,
                    self.parse_size(arr.get(1))?,
                ))
            }
            "MetaLink" => {
                let sub = sub.ok_or_else(|| error_once!("frame_item.meta_link"))?;
                let arr = self.into_array("frame_item", sub)?;
                Ok(typst_ts_core::artifact::doc::FrameItem::MetaLink(
                    self.parse_destination(arr.get(0))?,
                    self.parse_size(arr.get(1))?,
                ))
            }
            "None" => Ok(typst_ts_core::artifact::doc::FrameItem::None),
            _ => Err(error_once!(
                "frame_item.unknown_type",
                t: t,
                val: format!("{:?}", sub)
            )),
        }
    }

    pub fn parse_frame(&self, val: JsValue) -> ZResult<typst_ts_core::artifact::doc::Frame> {
        let mut size = None;
        let mut baseline = None;
        let mut items = vec![];

        for (k, v) in self.into_entries("frame", val)?.iter().map(convert_pair) {
            let k = self.to_string("frame", &k)?;
            match k.as_str() {
                "size" => {
                    size = Some(self.parse_size(v)?);
                }
                "baseline" => {
                    baseline = if !v.is_null() {
                        Some(typst::geom::Abs::raw(self.to_f64("frame.baseline", &v)?).into())
                    } else {
                        None
                    };
                }
                "items" => {
                    for item in self.into_array("frame.items", v)?.iter() {
                        let item = self.into_array("frame.items", item)?;
                        items.push((
                            self.parse_point(item.get(0))?,
                            self.parse_frame_item(item.get(1))?,
                        ));
                    }
                }

                _ => {
                    return Err(error_once!(
                        "frame.unknown_key",
                        k: k,
                        val: format!("{:?}", v)
                    ));
                }
            }
        }

        Ok(typst_ts_core::artifact::doc::Frame {
            size: size.ok_or_else(|| error_once!("frame.size.not_found"))?,
            baseline,
            items,
        })
    }

    pub fn parse_build_info(&self, val: JsValue) -> ZResult<typst_ts_core::artifact::BuildInfo> {
        let mut version = None;
        let mut compiler = None;

        for (k, v) in self
            .into_entries("build_info", val)?
            .iter()
            .map(convert_pair)
        {
            let k = self.to_string("build_info", &k)?;
            match k.as_str() {
                "version" => {
                    version = Some(self.to_string("build_info.version", &v)?);
                }
                "compiler" => {
                    compiler = Some(self.to_string("build_info.compiler", &v)?);
                }

                _ => {
                    return Err(error_once!(
                        "build_info.unknown_key",
                        k: k,
                        val: format!("{:?}", v)
                    ));
                }
            }
        }

        Ok(typst_ts_core::artifact::BuildInfo {
            version: version.ok_or_else(|| error_once!("build_info.version.not_found"))?,
            compiler: compiler.ok_or_else(|| error_once!("build_info.compiler.not_found"))?,
        })
    }

    pub fn parse_artifact(&self, val: JsValue) -> ZResult<Artifact> {
        let mut meta = ArtifactMeta::default();
        let mut pages = vec![];

        for (k, v) in self.into_entries("artifact", val)?.iter().map(convert_pair) {
            let k = self.to_string("artifact", &k)?;
            match k.as_str() {
                "build" => {
                    meta.build = Some(self.parse_build_info(v)?);
                }
                "pages" => {
                    for arr in self.into_array("artifact.pages", v)?.iter() {
                        pages.push(self.parse_frame(arr)?);
                    }
                }
                "fonts" => {
                    for arr in self.into_array("artifact.fonts", v)?.iter() {
                        meta.fonts.push(self.parse_font_info(arr)?);
                    }
                }
                "title" => {
                    meta.title = if v.is_null() {
                        None
                    } else {
                        Some(self.to_string("artifact.title", &v)?)
                    }
                }
                "author" => {
                    for arr in self.into_array("artifact.author", v)?.iter() {
                        meta.author.push(self.to_string("artifact.author", &arr)?);
                    }
                }
                _ => {
                    return Err(error_once!(
                        "artifact.unknown_key",
                        k: k,
                        val: format!("{:?}", v)
                    ));
                }
            }
        }

        Ok(Artifact { meta, pages })
    }
}

pub fn artifact_from_js_string(val: &str) -> ZResult<Artifact> {
    let val = js_sys::JSON::parse(val).map_err(map_err("ArtifactJsBuilder.ParseJson"))?;
    ArtifactJsBuilder {}
        .parse_artifact(val)
        .map_err(wrap_err("ArtifactJsBuilder.ArtifactFmt"))
}

pub fn page_from_js_string(val: &str) -> ZResult<Frame> {
    let val = js_sys::JSON::parse(val).map_err(map_err("ArtifactJsBuilder.ParseJson"))?;
    ArtifactJsBuilder {}
        .parse_frame(val)
        .map_err(wrap_err("ArtifactJsBuilder.PageFmt"))
}

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use typst_ts_core::artifact_ir::Artifact as IRArtifact;
    use typst_ts_core::Artifact;
    use typst_ts_test_common::web_artifact::{MAIN_ARTIFACT_IR, MAIN_ARTIFACT_JSON};
    use wasm_bindgen_test::*;

    use crate::artifact_ir::ir_artifact_from_bin;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn artifact_deserialization() {
        let artifact = String::from_utf8_lossy(MAIN_ARTIFACT_JSON);

        let window = web_sys::window().expect("should have a window in this context");
        let performance = window
            .performance()
            .expect("performance should be available");
        let serde_task = {
            let start = performance.now();
            let artifact: Artifact = serde_json::from_str(&artifact).unwrap();
            let end = performance.now();

            (end - start, artifact)
        };

        self::console_log!("serde.json {:.3}ms", serde_task.0);

        let js_task = {
            let start = performance.now();
            let artifact = super::artifact_from_js_string(&artifact).unwrap();
            let end = performance.now();

            (end - start, artifact)
        };

        self::console_log!("js.json: {:.3}ms", js_task.0);

        #[cfg(feature = "serde_rmp_debug")]
        {
            let rmp_task = {
                let artifact = include_bytes!("../../main.artifact.rmp");
                let start = performance.now();
                let artifact: Artifact = rmp_serde::from_slice(artifact.as_slice()).unwrap();
                let end = performance.now();

                (end - start, artifact)
            };

            self::console_log!("rmp: {:.3}ms", rmp_task.0);
        }

        {
            let ir_task = {
                let artifact = MAIN_ARTIFACT_IR;
                let start = performance.now();
                let artifact: IRArtifact = ir_artifact_from_bin(artifact).unwrap();
                let end = performance.now();

                (end - start, artifact)
            };

            self::console_log!("ir: {:.3}ms", ir_task.0);
        }
    }
}
