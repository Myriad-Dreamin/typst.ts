use reflexo_vec2sema::{BrowserFontMetric, SemaTask};
use reflexo_vec2svg::{ir::Page, Module};
use web_sys::wasm_bindgen::JsCast;

#[derive(Default)]
pub struct SemanticsBackend {}

static FONT_METRICS: once_cell::sync::OnceCell<BrowserFontMetric> =
    once_cell::sync::OnceCell::new();

impl SemanticsBackend {
    pub(crate) fn render(&self, module: &Module, page: &Page, heavy: bool) -> String {
        let metric = FONT_METRICS.get_or_init(|| {
            let canvas = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("canvas")
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();
            BrowserFontMetric::new(&canvas)
        });

        let mut output = vec![];
        let mut t = SemaTask::new(heavy, *metric, page.size.x.0, page.size.y.0);
        let ts = tiny_skia::Transform::identity();
        t.render_semantics(module, ts, page.content, &mut output);
        output.concat()
    }
}
