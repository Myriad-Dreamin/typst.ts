use reflexo_vec2canvas::BrowserFontMetric;
use reflexo_vec2sema::SemaTask;
use reflexo_vec2svg::{ir::Page, Module};

#[derive(Default)]
pub struct SemanticsBackend {}

static FONT_METRICS: std::sync::OnceLock<BrowserFontMetric> = std::sync::OnceLock::new();

impl SemanticsBackend {
    pub(crate) fn render(&self, module: &Module, page: &Page, heavy: bool) -> String {
        let metric = FONT_METRICS.get_or_init(BrowserFontMetric::from_env);

        let mut output = vec![];
        let mut t = SemaTask::new(heavy, *metric, page.size.x.0, page.size.y.0);
        let ts = tiny_skia::Transform::identity();
        t.render_semantics(module, ts, page.content, &mut output);
        output.concat()
    }
}
