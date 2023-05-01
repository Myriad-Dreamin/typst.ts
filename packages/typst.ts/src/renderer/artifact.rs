use std::{num::NonZeroUsize, vec};

use typst_ts_core::{artifact::doc::Frame, Artifact};
use wasm_bindgen::prelude::*;

pub struct ArtifactJsBuilder {}

/// Destructures a JS `[key, value]` pair into a tuple of [`Deserializer`]s.
pub(crate) fn convert_pair(pair: JsValue) -> (JsValue, JsValue) {
    let pair = pair.unchecked_into::<js_sys::Array>();
    (pair.get(0).into(), pair.get(1).into())
}

impl ArtifactJsBuilder {
    fn to_f64(&self, field: &str, val: &JsValue) -> Result<f64, JsValue> {
        Ok(val
            .as_f64()
            .ok_or_else(|| JsValue::from_str(&format!("expected f64 for {}, got {:?}", field, val)))
            .unwrap())
    }

    fn to_string(&self, field: &str, val: &JsValue) -> Result<String, JsValue> {
        Ok(val
            .as_string()
            .ok_or_else(|| {
                JsValue::from_str(&format!("expected string for {}, got {:?}", field, val))
            })
            .unwrap())
    }

    pub fn parse_tv(
        &self,
        field: &str,
        val: &JsValue,
    ) -> Result<(String, Option<JsValue>), JsValue> {
        let mut t = String::default();
        let mut sub: Option<JsValue> = None;

        for (k, v) in js_sys::Object::entries(val.dyn_ref().ok_or_else(|| {
            JsValue::from_str(&format!(
                "expected object for iterating {}, got {:?}",
                field, val
            ))
        })?)
        .iter()
        .map(convert_pair)
        {
            let k = self.to_string(field, &k)?;
            match k.as_str() {
                "t" => {
                    t = self.to_string(field, &v)?;
                }
                "v" => {
                    sub = Some(v);
                }
                _ => panic!("unknown key for {}: {}", field, k),
            }
        }

        Ok((t, sub))
    }

    pub fn parse_tv_expected(
        &self,
        field: &str,
        val: &JsValue,
    ) -> Result<(String, JsValue), JsValue> {
        let (t, sub) = self.parse_tv(field, &val)?;

        Ok((
            t,
            sub.ok_or_else(|| {
                JsValue::from_str(&format!("expected value for {}, got {:?}", field, val))
            })?,
        ))
    }

    fn parse_font_variant(&self, val: JsValue) -> Result<typst::font::FontVariant, JsValue> {
        let mut variant = typst::font::FontVariant::default();
        for (k, v) in js_sys::Object::entries(val.dyn_ref().unwrap())
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
                        _ => panic!("unknown FontStyle: {}", v),
                    }
                }
                "stretch" => {
                    variant.stretch = typst::font::FontStretch::from_ratio(
                        typst::geom::Ratio::new(self.to_f64("font_variant.stretch", &v)?),
                    );
                }
                _ => panic!("unknown key: {}", k),
            }
        }
        Ok(variant)
    }

    pub(crate) fn parse_font_info(
        &self,
        val: JsValue,
    ) -> Result<typst_ts_core::artifact::font::FontInfo, JsValue> {
        let mut family = String::default();
        let mut variant = typst::font::FontVariant::default();
        let mut flags = typst::font::FontFlags::from_bits(0 as u32).unwrap();
        let mut coverage = None;
        let mut ligatures = None;

        for (k, v) in js_sys::Object::entries(val.dyn_ref().unwrap())
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
                    .unwrap();
                }
                "coverage" => {
                    coverage = Some(typst::font::Coverage::from_vec(
                        v.dyn_into::<js_sys::Array>()?
                            .iter()
                            .map(|v| self.to_f64("font_info.coverage", &v).unwrap() as u32)
                            .collect(),
                    ));
                }
                "ligatures" => {
                    ligatures = Some(
                        v.dyn_into::<js_sys::Array>()?
                            .iter()
                            .map(convert_pair)
                            .map(|(g, s)| {
                                (
                                    self.to_f64("font_info.ligature_glyph", &g).unwrap() as u16,
                                    self.to_string("font_info.ligature_str", &s).unwrap(),
                                )
                            })
                            .collect(),
                    );
                }
                _ => panic!("unknown key: {}", k),
            }
        }
        Ok(typst_ts_core::artifact::font::FontInfo {
            family,
            variant,
            flags: flags.bits(),
            coverage: coverage.unwrap_or_else(|| typst::font::Coverage::from_vec(vec![])),
            ligatures: ligatures.unwrap_or_else(|| vec![]),
        })
    }

    pub fn parse_point(
        &self,
        val: JsValue,
    ) -> Result<typst_ts_core::artifact::geom::Point, JsValue> {
        let mut x = typst::geom::Abs::default();
        let mut y = typst::geom::Abs::default();

        for (k, v) in js_sys::Object::entries(val.dyn_ref().unwrap())
            .iter()
            .map(convert_pair)
        {
            let k = self.to_string("point", &k)?;
            match k.as_str() {
                "x" => {
                    x = typst::geom::Abs::raw(self.to_f64("point.x", &v)?).into();
                }
                "y" => {
                    y = typst::geom::Abs::raw(self.to_f64("point.y", &v)?).into();
                }
                _ => panic!("unknown key: {}", k),
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
    ) -> Result<typst_ts_core::artifact::geom::Transform, JsValue> {
        let mut sx = typst::geom::Ratio::default();
        let mut ky = typst::geom::Ratio::default();
        let mut kx = typst::geom::Ratio::default();
        let mut sy = typst::geom::Ratio::default();
        let mut tx = typst::geom::Abs::default();
        let mut ty = typst::geom::Abs::default();

        for (k, v) in js_sys::Object::entries(val.dyn_ref().unwrap())
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
                    tx = typst::geom::Abs::raw(self.to_f64("transform.tx", &v)?).into();
                }
                "ty" => {
                    ty = typst::geom::Abs::raw(self.to_f64("transform.ty", &v)?).into();
                }
                _ => panic!("unknown key: {}", k),
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
    ) -> Result<typst_ts_core::artifact::doc::GroupItem, JsValue> {
        let mut frame: Option<Frame> = None;
        let mut transform: Option<typst_ts_core::artifact::geom::Transform> = None;
        let mut clips = false;

        for (k, v) in js_sys::Object::entries(val.dyn_ref().unwrap())
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
                    clips = v.as_bool().unwrap();
                }
                _ => panic!("unknown key: {}", k),
            }
        }

        Ok(typst_ts_core::artifact::doc::GroupItem {
            frame: frame.unwrap(),
            transform: transform.unwrap(),
            clips,
        })
    }

    pub fn parse_glyph(
        &self,
        val: JsValue,
    ) -> Result<typst_ts_core::artifact::doc::Glyph, JsValue> {
        let mut id = None;
        let mut x_advance = None;
        let mut x_offset = None;
        let mut c = None;
        let mut span = None;
        let mut offset = None;

        for (k, v) in js_sys::Object::entries(val.dyn_ref().unwrap())
            .iter()
            .map(convert_pair)
        {
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
                "c" => {
                    c = Some(self.to_string("glyph.c", &v)?.chars().next().unwrap());
                }
                "span" => {
                    // todo: span self.to_f64(v)? as u16
                    span = Some(());
                }
                "offset" => {
                    offset = Some(self.to_f64("glyph.offset", &v)? as u16);
                }
                _ => panic!("unknown key: {}", k),
            }
        }

        Ok(typst_ts_core::artifact::doc::Glyph {
            id: id.unwrap(),
            x_advance: x_advance.unwrap(),
            x_offset: x_offset.unwrap(),
            c: c.unwrap(),
            span: span.unwrap(),
            offset: offset.unwrap(),
        })
    }

    pub fn parse_frame_text(
        &self,
        val: JsValue,
    ) -> Result<typst_ts_core::artifact::doc::TextItem, JsValue> {
        let mut font = None;
        let mut size = None;
        let mut fill = None;
        let mut lang = None;
        let mut glyphs = vec![];

        for (k, v) in js_sys::Object::entries(val.dyn_ref().unwrap())
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
                "glyphs" => {
                    for arr in v.dyn_into::<js_sys::Array>()?.iter() {
                        glyphs.push(self.parse_glyph(arr)?);
                    }
                }
                _ => panic!("unknown key: {}", k),
            }
        }

        Ok(typst_ts_core::artifact::doc::TextItem {
            font: font.unwrap(),
            size: size.unwrap(),
            fill: fill.unwrap(),
            lang: lang.unwrap(),
            glyphs,
        })
    }

    pub fn parse_size(&self, val: JsValue) -> Result<typst_ts_core::artifact::geom::Size, JsValue> {
        let mut x = typst::geom::Abs::default();
        let mut y = typst::geom::Abs::default();

        for (k, v) in js_sys::Object::entries(val.dyn_ref().unwrap())
            .iter()
            .map(convert_pair)
        {
            let k = self.to_string("size", &k)?;
            match k.as_str() {
                "x" => {
                    x = typst::geom::Abs::raw(self.to_f64("size.x", &v)?).into();
                }
                "y" => {
                    y = typst::geom::Abs::raw(self.to_f64("size.y", &v)?).into();
                }
                _ => panic!("unknown key: {}", k),
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
    ) -> Result<typst_ts_core::artifact::geom::PathItem, JsValue> {
        let (t, sub) = self.parse_tv_expected("path_item", &val)?;
        match t.as_str() {
            "MoveTo" => Ok(typst_ts_core::artifact::geom::PathItem::MoveTo(
                self.parse_point(sub)?,
            )),
            "LineTo" => Ok(typst_ts_core::artifact::geom::PathItem::LineTo(
                self.parse_point(sub)?,
            )),
            "CubicTo" => Ok({
                let a_sub = sub.dyn_into::<js_sys::Array>()?;
                typst_ts_core::artifact::geom::PathItem::CubicTo(
                    self.parse_point(a_sub.get(0))?,
                    self.parse_point(a_sub.get(1))?,
                    self.parse_point(a_sub.get(2))?,
                )
            }),
            "ClosePath" => Ok(typst_ts_core::artifact::geom::PathItem::ClosePath),
            _ => panic!("unknown path item: {}", t),
        }
    }

    pub fn parse_geometry(
        &self,
        val: JsValue,
    ) -> Result<typst_ts_core::artifact::geom::Geometry, JsValue> {
        let (t, sub) = self.parse_tv_expected("geometry", &val)?;
        match t.as_str() {
            "Line" => Ok(typst_ts_core::artifact::geom::Geometry::Line(
                self.parse_point(sub)?,
            )),
            "Rect" => Ok(typst_ts_core::artifact::geom::Geometry::Rect(
                self.parse_size(sub)?,
            )),
            "Path" => Ok(typst_ts_core::artifact::geom::Geometry::Path({
                let mut res = vec![];

                for arr in sub.dyn_into::<js_sys::Array>()?.iter() {
                    res.push(self.parse_path_item(arr)?);
                }
                typst_ts_core::artifact::geom::Path(res)
            })),
            _ => panic!("unknown geometry type: {}", t),
        }
    }

    pub fn parse_rgba_color(
        &self,
        val: JsValue,
    ) -> Result<typst_ts_core::artifact::geom::RgbaColor, JsValue> {
        let mut r = None;
        let mut g = None;
        let mut b = None;
        let mut a = None;

        for (k, v) in js_sys::Object::entries(val.dyn_ref().unwrap())
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
                _ => panic!("unknown key: {}", k),
            }
        }

        Ok(typst_ts_core::artifact::geom::RgbaColor {
            r: r.unwrap(),
            g: g.unwrap(),
            b: b.unwrap(),
            a: a.unwrap(),
        })
    }

    pub fn parse_cmyk_color(
        &self,
        val: JsValue,
    ) -> Result<typst_ts_core::artifact::geom::CmykColor, JsValue> {
        let mut c = None;
        let mut m = None;
        let mut y = None;
        let mut kk = None;

        for (k, v) in js_sys::Object::entries(val.dyn_ref().unwrap())
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
                _ => panic!("unknown key: {}", k),
            }
        }

        Ok(typst_ts_core::artifact::geom::CmykColor {
            c: c.unwrap(),
            m: m.unwrap(),
            y: y.unwrap(),
            k: kk.unwrap(),
        })
    }

    pub fn parse_color(
        &self,
        val: JsValue,
    ) -> Result<typst_ts_core::artifact::geom::Color, JsValue> {
        // /// An 8-bit luma color.
        // Luma(LumaColor),
        // /// An 8-bit RGBA color.
        // Rgba(RgbaColor),
        // /// An 8-bit CMYK color.
        // Cmyk(CmykColor),
        let (t, sub) = self.parse_tv_expected("color", &val)?;
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
            _ => panic!("unknown color type: {}", t),
        }
    }

    pub fn parse_paint(
        &self,
        val: JsValue,
    ) -> Result<typst_ts_core::artifact::geom::Paint, JsValue> {
        let (t, sub) = self.parse_tv_expected("paint", &val)?;
        match t.as_str() {
            "Solid" => Ok(typst_ts_core::artifact::geom::Paint::Solid(
                self.parse_color(sub)?,
            )),
            _ => panic!("unknown paint type: {}", t),
        }
    }

    pub fn parse_line_cap(
        &self,
        val: JsValue,
    ) -> Result<typst_ts_core::artifact::geom::LineCap, JsValue> {
        match self.to_string("line_cap", &val)?.as_str() {
            "Butt" => Ok(typst_ts_core::artifact::geom::LineCap::Butt),
            "Round" => Ok(typst_ts_core::artifact::geom::LineCap::Round),
            "Square" => Ok(typst_ts_core::artifact::geom::LineCap::Square),
            _ => panic!("unknown line cap type: {:?}", val),
        }
    }

    pub fn parse_line_join(
        &self,
        val: JsValue,
    ) -> Result<typst_ts_core::artifact::geom::LineJoin, JsValue> {
        match self.to_string("line_join", &val)?.as_str() {
            "Miter" => Ok(typst_ts_core::artifact::geom::LineJoin::Miter),
            "Round" => Ok(typst_ts_core::artifact::geom::LineJoin::Round),
            "Bevel" => Ok(typst_ts_core::artifact::geom::LineJoin::Bevel),
            _ => panic!("unknown line join type: {:?}", val),
        }
    }

    pub fn parse_dash_pattern(
        &self,
        val: JsValue,
    ) -> Result<
        typst_ts_core::artifact::geom::DashPattern<
            typst_ts_core::artifact::geom::Abs,
            typst_ts_core::artifact::geom::Abs,
        >,
        JsValue,
    > {
        // /// The dash array.
        // pub array: Vec<DT>,
        // /// The dash phase.
        // pub phase: T,
        let mut array = vec![];
        let mut phase = None;

        for (k, v) in js_sys::Object::entries(val.dyn_ref().unwrap())
            .iter()
            .map(convert_pair)
        {
            let k = self.to_string("dash_pattern", &k)?;
            match k.as_str() {
                "array" => {
                    for val in v.dyn_into::<js_sys::Array>().unwrap().iter() {
                        array.push(
                            typst::geom::Abs::raw(self.to_f64("dash_pattern.array", &val)?).into(),
                        );
                    }
                }
                "phase" => {
                    phase =
                        Some(typst::geom::Abs::raw(self.to_f64("dash_pattern.phase", &v)?).into());
                }
                _ => panic!("unknown key: {}", k),
            }
        }

        Ok(typst_ts_core::artifact::geom::DashPattern {
            array,
            phase: phase.unwrap(),
        })
    }

    pub fn parse_stroke(
        &self,
        val: JsValue,
    ) -> Result<typst_ts_core::artifact::geom::Stroke, JsValue> {
        let mut paint = None;
        let mut thickness = None;
        let mut line_cap = None;
        let mut line_join = None;
        let mut dash_pattern = None;
        let mut miter_limit = None;

        for (k, v) in js_sys::Object::entries(val.dyn_ref().unwrap())
            .iter()
            .map(convert_pair)
        {
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
                _ => panic!("unknown stroke key: {}", k),
            }
        }

        Ok(typst_ts_core::artifact::geom::Stroke {
            paint: paint.unwrap(),
            thickness: thickness.unwrap(),
            line_cap: line_cap.unwrap(),
            line_join: line_join.unwrap(),
            dash_pattern,
            miter_limit: miter_limit.unwrap(),
        })
    }

    pub fn parse_frame_shape(
        &self,
        val: JsValue,
    ) -> Result<typst_ts_core::artifact::geom::Shape, JsValue> {
        let mut geometry = None;
        let mut fill = None;
        let mut stroke = None;

        for (k, v) in js_sys::Object::entries(val.dyn_ref().unwrap())
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
                _ => panic!("unknown shape key: {}", k),
            }
        }

        Ok(typst_ts_core::artifact::geom::Shape {
            geometry: geometry.unwrap(),
            fill,
            stroke,
        })
    }

    pub fn parse_image(
        &self,
        val: JsValue,
    ) -> Result<typst_ts_core::artifact::image::Image, JsValue> {
        let mut data: Option<Vec<u8>> = None;
        let mut format = None;
        let mut width = None;
        let mut height = None;
        let mut alt = None;

        for (k, v) in js_sys::Object::entries(val.dyn_ref().unwrap())
            .iter()
            .map(convert_pair)
        {
            let k = self.to_string("image", &k)?;
            match k.as_str() {
                "data" => {
                    let data_raw = typst_ts_core::artifact::image::Image::decode_data(
                        &self.to_string("image.data.base64", &v)?,
                    )
                    .unwrap();
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
                _ => panic!("unknown image key: {}", k),
            }
        }

        Ok(typst_ts_core::artifact::image::Image {
            data: data.unwrap(),
            format: format.unwrap(),
            width: width.unwrap(),
            height: height.unwrap(),
            alt,
        })
    }

    pub fn parse_position(
        &self,
        val: JsValue,
    ) -> Result<typst_ts_core::artifact::doc::Position, JsValue> {
        // /// The page, starting at 1.
        // pub page: NonZeroUsize,
        // /// The exact coordinates on the page (from the top left, as usual).
        // pub point: Point,

        let mut page = None;
        let mut point = None;

        for (k, v) in js_sys::Object::entries(val.dyn_ref().unwrap())
            .iter()
            .map(convert_pair)
        {
            let k = self.to_string("position", &k)?;
            match k.as_str() {
                "page" => {
                    page = Some(
                        NonZeroUsize::new(self.to_f64("position.page", &v)? as usize).ok_or_else(
                            || JsValue::from_str(&format!("position.page: invalid value: {:?}", v)),
                        )?,
                    );
                }
                "point" => {
                    point = Some(self.parse_point(v)?);
                }
                _ => panic!("unknown position key: {}", k),
            }
        }

        Ok(typst_ts_core::artifact::doc::Position {
            page: page.unwrap(),
            point: point.unwrap(),
        })
    }

    pub fn parse_destination(
        &self,
        val: JsValue,
    ) -> Result<typst_ts_core::artifact::doc::Destination, JsValue> {
        let (t, sub) = self.parse_tv_expected("destination", &val)?;

        match t.as_str() {
            "Url" => {
                return Ok(typst_ts_core::artifact::doc::Destination::Url(
                    self.to_string("destination.Url", &sub)?,
                ));
            }
            "Position" => {
                return Ok(typst_ts_core::artifact::doc::Destination::Position(
                    self.parse_position(sub)?,
                ));
            }
            "Location" => {
                return Ok(typst_ts_core::artifact::doc::Destination::Location(
                    self.to_string("destination.Location", &sub)?,
                ));
            }
            _ => panic!("unknown destination type: {}", t),
        }
    }

    pub fn parse_frame_item(
        &self,
        val: JsValue,
    ) -> Result<typst_ts_core::artifact::doc::FrameItem, JsValue> {
        let (t, sub) = self.parse_tv("frame_item", &val)?;
        match t.as_str() {
            "Group" => {
                return Ok(typst_ts_core::artifact::doc::FrameItem::Group(
                    self.parse_frame_group(
                        sub.ok_or_else(|| JsValue::from_str("frame_item: missing sub for Group"))?,
                    )?,
                ));
            }
            "Text" => {
                return Ok(typst_ts_core::artifact::doc::FrameItem::Text(
                    self.parse_frame_text(
                        sub.ok_or_else(|| JsValue::from_str("frame_item: missing sub for Text"))?,
                    )?,
                ));
            }
            "Shape" => {
                return Ok(typst_ts_core::artifact::doc::FrameItem::Shape(
                    self.parse_frame_shape(
                        sub.ok_or_else(|| JsValue::from_str("frame_item: missing sub for Shape"))?,
                    )?,
                ));
            }
            "Image" => {
                return {
                    let arr = sub
                        .ok_or_else(|| JsValue::from_str("frame_item: missing sub for MetaLink"))?
                        .dyn_into::<js_sys::Array>()?;
                    Ok(typst_ts_core::artifact::doc::FrameItem::Image(
                        self.parse_image(arr.get(0))?,
                        self.parse_size(arr.get(1))?,
                    ))
                }
            }
            "MetaLink" => {
                return {
                    let arr = sub
                        .ok_or_else(|| JsValue::from_str("frame_item: missing sub for MetaLink"))?
                        .dyn_into::<js_sys::Array>()?;
                    Ok(typst_ts_core::artifact::doc::FrameItem::MetaLink(
                        self.parse_destination(arr.get(0))?,
                        self.parse_size(arr.get(1))?,
                    ))
                };
            }
            "None" => {
                return Ok(typst_ts_core::artifact::doc::FrameItem::None);
            }
            _ => panic!("unknown type: {}", t),
        }
    }

    pub fn parse_frame(
        &self,
        val: JsValue,
    ) -> Result<typst_ts_core::artifact::doc::Frame, JsValue> {
        let mut size = None;
        let mut baseline = None;
        let mut items = vec![];

        for (k, v) in js_sys::Object::entries(val.dyn_ref().unwrap())
            .iter()
            .map(convert_pair)
        {
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
                    for item in v.dyn_into::<js_sys::Array>()?.iter() {
                        let item = item.dyn_into::<js_sys::Array>()?;
                        items.push((
                            self.parse_point(item.get(0).into())?,
                            self.parse_frame_item(item.get(1).into())?,
                        ));
                    }
                }
                _ => panic!("unknown key: {}", k),
            }
        }

        Ok(typst_ts_core::artifact::doc::Frame {
            size: size.unwrap(),
            baseline,
            items,
        })
    }

    pub fn parse_build_info(
        &self,
        val: &JsValue,
    ) -> Result<typst_ts_core::artifact::BuildInfo, JsValue> {
        let mut version = None;
        let mut compiler = None;

        for (k, v) in js_sys::Object::entries(val.dyn_ref().unwrap())
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
                _ => panic!("unknown key: {}", k),
            }
        }

        Ok(typst_ts_core::artifact::BuildInfo {
            version: version.unwrap(),
            compiler: compiler.unwrap(),
        })
    }

    pub fn from_value(&self, val: JsValue) -> Result<Artifact, JsValue> {
        let mut artifact = Artifact {
            build: None,
            pages: vec![],
            fonts: vec![],
            title: None,
            author: vec![],
        };

        for (k, v) in js_sys::Object::entries(val.dyn_ref().ok_or("typst: not a js object")?)
            .iter()
            .map(convert_pair)
        {
            let k = k.as_string().ok_or("typst: artifact not a js string")?;
            match k.as_str() {
                "build" => {
                    artifact.build = Some(self.parse_build_info(&v)?);
                }
                "pages" => {
                    for arr in v.dyn_into::<js_sys::Array>()?.iter() {
                        artifact.pages.push(self.parse_frame(arr)?);
                    }
                }
                "fonts" => {
                    for arr in v.dyn_into::<js_sys::Array>()?.iter() {
                        artifact.fonts.push(self.parse_font_info(arr)?);
                    }
                }
                "title" => {
                    artifact.title = if v.is_null() {
                        None
                    } else {
                        Some(v.as_string().ok_or_else(|| {
                            JsValue::from_str(&format!("typst: title not a js string: {:?}", v))
                        })?)
                    }
                }
                "author" => {
                    for arr in v
                        .dyn_ref::<js_sys::Array>()
                        .ok_or("typst: author not a array")?
                        .iter()
                    {
                        artifact.author.push(arr.as_string().ok_or_else(|| {
                            JsValue::from_str(&format!("typst: author not a js string: {:?}", v))
                        })?);
                    }
                }
                _ => panic!("unknown key: {}", k),
            }
        }

        Ok(artifact)
    }
}

pub fn artifact_from_js_string(val: String) -> Result<Artifact, JsValue> {
    let val = js_sys::JSON::parse(&val).unwrap();
    ArtifactJsBuilder {}.from_value(val)
}

pub fn page_from_js_string(val: String) -> Result<Frame, JsValue> {
    let val = js_sys::JSON::parse(&val).unwrap();
    ArtifactJsBuilder {}.parse_frame(val)
}

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use typst_ts_core::Artifact;
    use typst_ts_core::artifact_ir::{Artifact as IRArtifact, ArtifactMetadata as IRArtifactMetadata};
    use wasm_bindgen_test::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn deserialize_from_ir_bin(input: &[u8]) -> IRArtifact {
        use std::io::Read;
        use byteorder::{LittleEndian, ReadBytesExt};
        let mut reader = std::io::Cursor::new(input);
        let mut magic = [0; 4];
        reader.read(&mut magic).unwrap();
        assert_eq!(magic, ['I' as u8, 'R' as u8, 'A' as u8, 'R' as u8]);
        assert_eq!(reader.read_i32::<LittleEndian>().unwrap(), 1);
        let meta_len = reader.read_u64::<LittleEndian>().unwrap();
        let mut meta = vec![0; meta_len as usize];
        reader.read_exact(&mut meta).unwrap();
        let meta = String::from_utf8(meta).unwrap();
        let meta = serde_json::from_str(&meta).unwrap();
        let mut buffer = vec![];
        reader.read_to_end(&mut buffer).unwrap();
    
        IRArtifact {
            metadata: meta,
            buffer,
        }
    }

    #[wasm_bindgen_test]
    fn artifact_deserialization() {
        let artifact = include_bytes!("../../main.artifact.json");
        let artifact = String::from_utf8_lossy(artifact);

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

        console_log!("serde.json {}ms", serde_task.0);

        let js_task = {
            let start = performance.now();
            let artifact = super::artifact_from_js_string(artifact.to_string()).unwrap();
            let end = performance.now();

            (end - start, artifact)
        };

        console_log!("js.json: {}ms", js_task.0);

        #[cfg(feature = "serde_rmp_debug")]
        {
            let rmp_task = {
                let artifact = include_bytes!("../../main.artifact.rmp");
                let start = performance.now();
                let artifact: Artifact = rmp_serde::from_slice(artifact.as_slice()).unwrap();
                let end = performance.now();

                (end - start, artifact)
            };

            console_log!("rmp: {}ms", rmp_task.0);
        }

        {
            let ir_task = {
                let artifact = include_bytes!("../../main.artifact_ir.bin");
                let start = performance.now();
                let artifact: IRArtifact = deserialize_from_ir_bin(artifact);
                let end = performance.now();

                (end - start, artifact)
            };

            console_log!("ir: {}ms", ir_task.0);
        }
    }
}
