use web_sys::{CanvasWindingRule, ImageBitmap, ImageData, OffscreenCanvas, Path2d};

pub trait CanvasDevice {
    #[doc = "The `restore()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/restore)"]
    fn restore(&self);
    #[doc = "The `save()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/save)"]
    fn save(&self);

    #[doc = "The `setTransform()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/setTransform)"]
    fn set_transform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64);
    #[doc = "Setter for the `fillStyle` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/fillStyle)"]
    fn set_fill_style(&self, value: &::wasm_bindgen::JsValue);

    #[doc = "The `fillRect()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/fillRect)"]
    fn fill_rect(&self, x: f64, y: f64, w: f64, h: f64);

    #[doc = "The `putImageData()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/putImageData)"]
    fn put_image_data(&self, imagedata: &ImageData, dx: f64, dy: f64);
    #[doc = "Getter for the `globalCompositeOperation` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/globalCompositeOperation)"]
    fn global_composite_operation(&self) -> String;
    #[doc = "Setter for the `globalCompositeOperation` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/globalCompositeOperation)"]
    fn set_global_composite_operation(&self, value: &str);
    #[doc = "The `drawImage()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/drawImage)"]
    fn draw_image_with_offscreen_canvas(&self, image: &OffscreenCanvas, dx: f64, dy: f64);

    #[doc = "The `clip()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/clip)"]
    fn clip_with_path_2d(&self, path: &Path2d);
    #[doc = "Setter for the `strokeStyle` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/strokeStyle)"]
    fn set_stroke_style(&self, value: &::wasm_bindgen::JsValue);
    #[doc = "The `stroke()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/stroke)"]
    fn stroke_with_path(&self, path: &Path2d);
    #[doc = "The `drawImage()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/drawImage)"]
    fn draw_image_with_image_bitmap_and_dw_and_dh(
        &self,
        image: &ImageBitmap,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
    );
    #[doc = "The `drawImage()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/OffscreenCanvasRenderingContext2D/drawImage)"]
    fn draw_image_with_offscreen_canvas_and_dw_and_dh(
        &self,
        image: &OffscreenCanvas,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
    );

    #[doc = "Setter for the `lineWidth` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/lineWidth)"]
    fn set_line_width(&self, value: f64);
    #[doc = "Setter for the `lineCap` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/lineCap)"]
    fn set_line_cap(&self, value: &str);
    #[doc = "Setter for the `lineJoin` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/lineJoin)"]
    fn set_line_join(&self, value: &str);
    #[doc = "Setter for the `miterLimit` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/miterLimit)"]
    fn set_miter_limit(&self, value: f64);
    #[doc = "Setter for the `lineDashOffset` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/lineDashOffset)"]
    fn set_line_dash_offset(&self, value: f64);
    #[doc = "The `setLineDash()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/setLineDash)"]
    fn set_line_dash(&self, segments: &::wasm_bindgen::JsValue);
    #[doc = "The `fill()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/fill)"]
    fn fill_with_path_2d(&self, path: &Path2d);
    #[doc = "The `fill()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/OffscreenCanvasRenderingContext2D/fill)"]
    fn fill_with_path_2d_and_winding(&self, path: &Path2d, winding: CanvasWindingRule);
}

impl CanvasDevice for web_sys::CanvasRenderingContext2d {
    fn set_transform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        // .map_err(map_err("CanvasRenderTask.SetTransform"))
        self.set_transform(a, b, c, d, e, f).unwrap();
    }

    fn put_image_data(&self, imagedata: &ImageData, dx: f64, dy: f64) {
        self.put_image_data(imagedata, dx, dy).unwrap();
    }

    fn global_composite_operation(&self) -> String {
        self.global_composite_operation().unwrap()
    }

    fn set_global_composite_operation(&self, value: &str) {
        self.set_global_composite_operation(value).unwrap();
    }

    fn set_fill_style(&self, value: &::wasm_bindgen::JsValue) {
        self.set_fill_style(value);
    }

    fn fill_rect(&self, x: f64, y: f64, w: f64, h: f64) {
        self.fill_rect(x, y, w, h);
    }

    fn draw_image_with_offscreen_canvas(&self, image: &OffscreenCanvas, dx: f64, dy: f64) {
        self.draw_image_with_offscreen_canvas(image, dx, dy)
            .unwrap();
    }

    fn clip_with_path_2d(&self, path: &Path2d) {
        self.clip_with_path_2d(path);
    }

    fn set_stroke_style(&self, value: &::wasm_bindgen::JsValue) {
        self.set_stroke_style(value);
    }

    fn stroke_with_path(&self, path: &Path2d) {
        self.stroke_with_path(path);
    }

    fn draw_image_with_image_bitmap_and_dw_and_dh(
        &self,
        image: &ImageBitmap,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
    ) {
        self.draw_image_with_image_bitmap_and_dw_and_dh(image, dx, dy, dw, dh)
            .unwrap();
    }

    fn draw_image_with_offscreen_canvas_and_dw_and_dh(
        &self,
        image: &OffscreenCanvas,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
    ) {
        self.draw_image_with_offscreen_canvas_and_dw_and_dh(image, dx, dy, dw, dh)
            .unwrap();
    }

    fn set_line_width(&self, value: f64) {
        self.set_line_width(value);
    }

    fn set_line_cap(&self, value: &str) {
        self.set_line_cap(value);
    }

    fn set_line_join(&self, value: &str) {
        self.set_line_join(value);
    }

    fn set_miter_limit(&self, value: f64) {
        self.set_miter_limit(value);
    }

    fn set_line_dash_offset(&self, value: f64) {
        self.set_line_dash_offset(value);
    }

    fn restore(&self) {
        self.restore();
    }

    fn save(&self) {
        self.save();
    }

    fn set_line_dash(&self, segments: &::wasm_bindgen::JsValue) {
        self.set_line_dash(segments).unwrap();
    }

    fn fill_with_path_2d(&self, path: &Path2d) {
        self.fill_with_path_2d(path);
    }

    fn fill_with_path_2d_and_winding(&self, path: &Path2d, winding: CanvasWindingRule) {
        self.fill_with_path_2d_and_winding(path, winding);
    }
}

impl CanvasDevice for web_sys::OffscreenCanvasRenderingContext2d {
    fn set_transform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        self.set_transform(a, b, c, d, e, f).unwrap();
    }

    fn put_image_data(&self, imagedata: &ImageData, dx: f64, dy: f64) {
        self.put_image_data(imagedata, dx, dy).unwrap();
    }

    fn global_composite_operation(&self) -> String {
        self.global_composite_operation().unwrap()
    }

    fn set_global_composite_operation(&self, value: &str) {
        self.set_global_composite_operation(value).unwrap();
    }

    fn set_fill_style(&self, value: &::wasm_bindgen::JsValue) {
        self.set_fill_style(value);
    }

    fn fill_rect(&self, x: f64, y: f64, w: f64, h: f64) {
        self.fill_rect(x, y, w, h);
    }

    fn draw_image_with_offscreen_canvas(&self, image: &OffscreenCanvas, dx: f64, dy: f64) {
        self.draw_image_with_offscreen_canvas(image, dx, dy)
            .unwrap();
    }

    fn clip_with_path_2d(&self, path: &Path2d) {
        self.clip_with_path_2d(path);
    }

    fn set_stroke_style(&self, value: &::wasm_bindgen::JsValue) {
        self.set_stroke_style(value);
    }

    fn stroke_with_path(&self, path: &Path2d) {
        self.stroke_with_path(path);
    }

    fn draw_image_with_image_bitmap_and_dw_and_dh(
        &self,
        image: &ImageBitmap,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
    ) {
        self.draw_image_with_image_bitmap_and_dw_and_dh(image, dx, dy, dw, dh)
            .unwrap();
    }

    fn draw_image_with_offscreen_canvas_and_dw_and_dh(
        &self,
        image: &OffscreenCanvas,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
    ) {
        self.draw_image_with_offscreen_canvas_and_dw_and_dh(image, dx, dy, dw, dh)
            .unwrap();
    }

    fn set_line_width(&self, value: f64) {
        self.set_line_width(value);
    }

    fn set_line_cap(&self, value: &str) {
        self.set_line_cap(value);
    }

    fn set_line_join(&self, value: &str) {
        self.set_line_join(value);
    }

    fn set_miter_limit(&self, value: f64) {
        self.set_miter_limit(value);
    }

    fn set_line_dash_offset(&self, value: f64) {
        self.set_line_dash_offset(value);
    }

    fn restore(&self) {
        self.restore();
    }

    fn save(&self) {
        self.save();
    }

    fn set_line_dash(&self, segments: &::wasm_bindgen::JsValue) {
        self.set_line_dash(segments).unwrap();
    }

    fn fill_with_path_2d(&self, path: &Path2d) {
        self.fill_with_path_2d(path);
    }

    fn fill_with_path_2d_and_winding(&self, path: &Path2d, winding: CanvasWindingRule) {
        self.fill_with_path_2d_and_winding(path, winding);
    }
}
