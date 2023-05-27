use std::fmt::Write;

use wasm_bindgen::JsValue;
use web_sys::Path2d;

#[derive(Default)]
pub struct SvgPath2DBuilder(pub String);

/// See: https://developer.mozilla.org/en-US/docs/Web/SVG/Tutorial/Paths
impl SvgPath2DBuilder {
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        write!(
            &mut self.0,
            "M {} {} H {} V {} H {} Z",
            x,
            y,
            x + w,
            y + h,
            x
        )
        .unwrap();
    }

    pub fn build(&self) -> Result<Path2d, JsValue> {
        Path2d::new_with_path_string(&self.0)
    }
}

impl ttf_parser::OutlineBuilder for SvgPath2DBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        write!(&mut self.0, "M {} {} ", x, y).unwrap();
    }

    fn line_to(&mut self, x: f32, y: f32) {
        write!(&mut self.0, "L {} {} ", x, y).unwrap();
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        write!(&mut self.0, "Q {} {} {} {} ", x1, y1, x, y).unwrap();
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        write!(&mut self.0, "C {} {} {} {} {} {} ", x1, y1, x2, y2, x, y).unwrap();
    }

    fn close(&mut self) {
        write!(&mut self.0, "Z ").unwrap();
    }
}
