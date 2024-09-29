use web_sys::{ImageData, OffscreenCanvas};

pub trait CanvasDevice {
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
}

impl CanvasDevice for web_sys::CanvasRenderingContext2d {
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
}
