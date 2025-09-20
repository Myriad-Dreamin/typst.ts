pub mod builder;

#[cfg(feature = "incr")]
mod incr;
pub(crate) mod utils;

pub use reflexo_typst::*;

use core::fmt;
use std::{fmt::Write, path::Path, sync::Arc};

use error::TypstSourceDiagnostic;
use font::cache::FontInfoCache;
use js_sys::{Array, JsString, Uint8Array};
use reflexo_typst::error::{long_diag_from_std, DiagMessage};
use reflexo_typst::font::web::BrowserFontSearcher;
use reflexo_typst::package::registry::JsRegistry;
use reflexo_typst::prelude::EcoVec;
use reflexo_typst::typst::foundations::IntoValue;
use reflexo_typst::vfs::browser::ProxyAccessModel;
use typst::diag::{SourceResult, Warned};
use wasm_bindgen::prelude::*;

use crate::utils::console_log;
#[cfg(feature = "incr")]
use incr::IncrServer;

macro_rules! take_diag {
    ($diagnostics_format:expr, $g:expr, $world:expr, $e:expr, $D:ty) => {
        match $e {
            Some(v) => v,
            None => {
                let diag: Arc<TDiagnosticsTask<$D>> = $g.compute::<TDiagnosticsTask<$D>>()?;
                let diag = diag.diagnostics();
                if $diagnostics_format >= 2 {
                    return Ok(convert_diag(diag, Some($world), $diagnostics_format));
                } else {
                    let diag = diag.collect::<Vec<_>>();
                    return Err(format!("{diag:?}").into());
                }
            }
        }
    };
}

/// In format of
///
/// ```log
/// // with package
/// cetz:0.2.0@lib.typ:2:9-3:15: error: unexpected type in `+` application
/// // without package
/// main.typ:2:9-3:15: error: unexpected type in `+` application
/// ```
struct UnixFmt(DiagMessage);

impl fmt::Display for UnixFmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.package.is_empty() {
        } else {
            f.write_str(&self.0.package)?;
            f.write_char('@')?;
        }
        f.write_str(&self.0.path)?;
        f.write_char(':')?;

        if let Some(r) = self.0.range.as_ref() {
            write!(f, "{}:{}:", r.start.line + 1, r.start.character + 1)?;
        }

        write!(f, " {}: {}", self.0.severity, self.0.message)
    }
}

fn convert_diag<'a>(
    e: impl Iterator<Item = &'a TypstSourceDiagnostic>,
    world: Option<&dyn TypstWorld>,
    diagnostics_format: u8,
) -> JsValue {
    fn convert_diag_object(e: DiagMessage) -> JsValue {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &"package".into(), &e.package.into()).unwrap();
        js_sys::Reflect::set(&obj, &"path".into(), &e.path.into()).unwrap();
        if let Some(range) = e.range {
            let rng = format!(
                "{}:{}-{}:{}",
                range.start.line, range.start.character, range.end.line, range.end.character
            )
            .into();
            js_sys::Reflect::set(&obj, &"range".into(), &rng).unwrap();
        } else {
            js_sys::Reflect::set(&obj, &"range".into(), &"".into()).unwrap();
        }
        js_sys::Reflect::set(&obj, &"severity".into(), &e.severity.to_string().into()).unwrap();
        js_sys::Reflect::set(&obj, &"message".into(), &e.message.as_str().into()).unwrap();
        obj.into()
    }

    let res = e
        .into_iter()
        .flat_map(move |e| long_diag_from_std(e, world))
        .map(|e| {
            if diagnostics_format == 3 {
                convert_diag_object(e)
            } else {
                format!("{}", UnixFmt(e)).into()
            }
        });

    let diag = Array::from_iter(res).into();

    let res = js_sys::Object::new();
    js_sys::Reflect::set(&res, &"diagnostics".into(), &diag).unwrap();
    res.into()
}

#[wasm_bindgen]
pub struct TypstCompiler {
    pub(crate) verse: TypstBrowserUniverse,
}

impl TypstCompiler {
    pub async fn new(
        access_model: ProxyAccessModel,
        registry: JsRegistry,
        searcher: BrowserFontSearcher,
    ) -> Result<Self, JsValue> {
        Ok(Self {
            verse: TypstBrowserUniverse::new(
                std::path::Path::new("/").to_owned(),
                None,
                access_model,
                registry,
                searcher.build(),
            ),
        })
    }
}

#[wasm_bindgen]
pub fn get_font_info(buffer: Uint8Array) -> JsValue {
    serde_wasm_bindgen::to_value(&FontInfoCache::from_data(buffer.to_vec().as_slice())).unwrap()
}

// todo: design error handling
// todo: we return a string for now which is better than nothing
#[wasm_bindgen]
#[allow(non_snake_case)]
impl TypstCompiler {
    pub fn reset(&mut self) -> Result<(), JsValue> {
        // reset the world caches
        self.verse.evict(30);

        Ok(())
    }

    pub fn set_inputs(&mut self, inputs: JsValue) -> Result<(), JsValue> {
        let inputs: std::collections::HashMap<String, String> =
            serde_wasm_bindgen::from_value(inputs).map_err(|e| format!("{e:?}"))?;
        let inputs = inputs
            .into_iter()
            .map(|(k, v)| (k.into(), v.into_value()))
            .collect();
        self.verse
            .increment_revision(|verse| verse.set_inputs(Arc::new(LazyHash::new(inputs))));
        Ok(())
    }

    pub fn add_source(&mut self, path: &str, content: &str) -> bool {
        let path = Path::new(path).to_owned();
        match self
            .verse
            .map_shadow(&path, Bytes::from_string(content.to_owned()))
        {
            Ok(_) => true,
            Err(e) => {
                console_log!("Error: {:?}", e);
                false
            }
        }
    }

    pub fn map_shadow(&mut self, path: &str, content: &[u8]) -> bool {
        let path = Path::new(path).to_owned();
        match self.verse.map_shadow(&path, Bytes::new(content.to_owned())) {
            Ok(_) => true,
            Err(e) => {
                console_log!("Error: {:?}", e);
                false
            }
        }
    }

    pub fn unmap_shadow(&mut self, path: &str) -> bool {
        let path = Path::new(path).to_owned();
        match self.verse.unmap_shadow(&path) {
            Ok(_) => true,
            Err(e) => {
                console_log!("Error: {:?}", e);
                false
            }
        }
    }

    pub fn reset_shadow(&mut self) {
        self.verse.reset_shadow()
    }

    // todo: font manipulation
    // pub fn modify_font_data(&mut self, idx: usize, buffer: Uint8Array) {}
    // pub fn rebuild(&mut self) {}

    pub fn get_loaded_fonts(&mut self) -> Vec<JsString> {
        self.verse
            .font_resolver
            .loaded_fonts()
            .map(|s| format!("<{}, {:?}>", s.0, s.1).into())
            .collect()
    }

    #[cfg(feature = "ast")]
    pub fn get_ast(&mut self, main_file_path: String) -> Result<String, JsValue> {
        self.verse
            .increment_revision(|verse| verse.set_entry_file(Path::new(&main_file_path).into()))
            .map_err(|e| format!("{e:?}"))?;
        let world = self.verse.snapshot();

        // export ast
        let src = world.main();
        let src = world.source(src).unwrap();

        let mut cursor = std::io::Cursor::new(Vec::new());
        reflexo_typst::dump_ast(
            &src.id().vpath().as_rootless_path().display().to_string(),
            &src,
            &mut cursor,
        )
        .map_err(|e| format!("{e:?}"))?;
        let data = cursor.into_inner();

        let converted = ansi_to_html::convert(
            String::from_utf8(data)
                .map_err(|e| format!("{e:?}"))?
                .as_str(),
        )
        .map_err(|e| format!("{e:?}"))?;
        Ok(converted)
    }

    pub fn get_semantic_token_legend(&mut self) -> Result<JsValue, JsValue> {
        let tokens = self.verse.get_semantic_token_legend();
        serde_wasm_bindgen::to_value(tokens.as_ref()).map_err(|e| format!("{e:?}").into())
    }

    #[cfg(feature = "semantic_tokens")]
    pub fn get_semantic_tokens(
        &mut self,
        offset_encoding: String,
        file_path: Option<String>,
        result_id: Option<String>,
    ) -> Result<js_sys::Object, JsValue> {
        use js_sys::Uint32Array;
        use reflexo_typst::parser::OffsetEncoding;
        if let Some(result_id) = result_id {
            return Err(
                error_once!("Not implemented", result_id: format!("{:?}", result_id)).into(),
            );
        }

        let tokens = self.verse.get_semantic_tokens(
            file_path,
            match offset_encoding.as_str() {
               "utf-16" => OffsetEncoding::Utf16,
              "utf-8" => OffsetEncoding::Utf8,
                _ => {
                    return Err(error_once!("Unsupported offset encoding", offset_encoding: offset_encoding).into());
                }
            },
        )?;
        let mut result = Vec::new();
        for token in tokens.iter() {
            result.push(token.delta_line);
            result.push(token.delta_start_character);
            result.push(token.length);
            result.push(token.token_type);
            result.push(token.token_modifiers);
        }

        let semantic_tokens = js_sys::Object::new();
        js_sys::Reflect::set(
            &semantic_tokens,
            &"data".into(),
            &Uint32Array::from(&result[..]).into(),
        )?;
        js_sys::Reflect::set(
            &semantic_tokens,
            &"resultId".into(),
            &JsString::from("").into(),
        )?;

        Ok(semantic_tokens)
    }

    pub fn snapshot(
        &mut self,
        root: Option<String>,
        main_file_path: Option<String>,
        inputs: Option<Vec<js_sys::Array>>,
    ) -> Result<TypstCompileWorld, JsValue> {
        let inputs = inputs.map(|inputs| Arc::new(LazyHash::new(convert_inputs(&inputs))));

        let entry = if let Some(root) = root {
            EntryState::new_workspace(Path::new(&root).into())
        } else {
            self.verse.entry_state()
        };

        let entry = if let Some(main_file_path) = main_file_path {
            entry
                .try_select_path_in_workspace(Path::new(&main_file_path))?
                .ok_or_else(|| error_once!("failed to select path", path: main_file_path))?
        } else {
            entry.clone()
        };

        let world = self.verse.snapshot_with(Some(TaskInputs {
            entry: Some(entry),
            inputs,
        }));

        Ok(TypstCompileWorld {
            graph: WorldComputeGraph::new(CompileSnapshot::from_world(world)),
        })
    }
    pub fn get_artifact(
        &mut self,
        fmt: String,
        diagnostics_format: u8,
    ) -> Result<JsValue, JsValue> {
        let mut w = self.snapshot(None, None, None)?;
        w.get_artifact(fmt, diagnostics_format)
    }

    pub fn query(
        &mut self,
        main_file_path: String,
        inputs: Option<Vec<js_sys::Array>>,
        selector: String,
        field: Option<String>,
    ) -> Result<String, JsValue> {
        let mut w = self.snapshot(None, Some(main_file_path), inputs)?;
        w.query(selector, field)
    }

    pub fn compile(
        &mut self,
        main_file_path: String,
        inputs: Option<Vec<js_sys::Array>>,
        fmt: String,
        diagnostics_format: u8,
    ) -> Result<JsValue, JsValue> {
        let mut w = self.snapshot(None, Some(main_file_path), inputs)?;
        w.get_artifact(fmt, diagnostics_format)
    }

    #[cfg(feature = "incr")]
    pub fn create_incr_server(&mut self) -> Result<IncrServer, JsValue> {
        Ok(IncrServer::default())
    }

    #[cfg(feature = "incr")]
    pub fn incr_compile(
        &mut self,
        main_file_path: String,
        inputs: Option<Vec<js_sys::Array>>,
        state: &mut IncrServer,
        diagnostics_format: u8,
    ) -> Result<JsValue, JsValue> {
        let mut w = self.snapshot(None, Some(main_file_path), inputs)?;
        w.incr_compile(state, diagnostics_format)
    }
}

#[wasm_bindgen]
pub struct TypstCompileWorld {
    graph: Arc<WorldComputeGraph<BrowserCompilerFeat>>,
}

impl TypstCompileWorld {
    pub fn get_artifact(
        &mut self,
        fmt: String,
        diagnostics_format: u8,
    ) -> Result<JsValue, JsValue> {
        #[cfg(feature = "svg")]
        use reflexo_vec2svg::DefaultExportFeature;
        #[cfg(feature = "svg")]
        type SvgModuleExport = WebSvgModuleExport<DefaultExportFeature>;
        #[cfg(feature = "pdf")]
        use reflexo_typst::task::ExportPdfTask;

        let g = &self.graph;
        let world = &self.graph.snap.world;

        let _ = g.provide::<FlagTask<CompilationTask<reflexo_typst::TypstPagedDocument>>>(Ok(
            FlagTask::flag(true),
        ));
        // todo: warning is ignored here.
        let doc = take_diag!(
            diagnostics_format,
            g,
            &world,
            g.shared_compile()?,
            reflexo_typst::TypstPagedDocument
        );

        let artifact_bytes: Bytes = match fmt.as_str() {
            #[cfg(feature = "svg")]
            "vector" => SvgModuleExport::run(&g, &doc, &ExportWebSvgModuleTask::default())?,
            #[cfg(feature = "pdf")]
            "pdf" => PdfExport::run(&g, &doc, &ExportPdfTask::default())?,
            "_dummy" => Bytes::new([]),
            fmt => {
                let _ = doc;
                return Err(error_once!("Unsupported fmt", format: fmt).into());
            }
        };

        let v: JsValue = Uint8Array::from(artifact_bytes.as_slice()).into();

        Ok(if diagnostics_format != 0 {
            let result = js_sys::Object::new();
            js_sys::Reflect::set(&result, &"result".into(), &v)?;
            result.into()
        } else {
            v
        })
    }

    pub fn query(&mut self, selector: String, field: Option<String>) -> Result<String, JsValue> {
        let g = &self.graph;

        // todo: diagnostics
        let doc = g.compile().output.map_err(|e| format!("{e:?}"))?;
        // todo: query html?
        let elements: Vec<typst::foundations::Content> = g
            .query(selector, &TypstDocument::Paged(doc))
            .map_err(|e| format!("{e:?}"))?;

        let mapped: Vec<_> = elements
            .into_iter()
            .filter_map(|c| match &field {
                Some(field) => c.get_by_name(field).ok(),
                _ => Some(c.into_value()),
            })
            .collect();

        Ok(serde_json::to_string_pretty(&mapped).map_err(|e| format!("{e:?}"))?)
    }

    #[cfg(feature = "incr")]
    pub fn incr_compile(
        &mut self,
        state: &mut IncrServer,
        diagnostics_format: u8,
    ) -> Result<JsValue, JsValue> {
        let g = &self.graph;
        let _ = g.provide::<FlagTask<CompilationTask<reflexo_typst::TypstPagedDocument>>>(Ok(
            FlagTask::flag(true),
        ));
        let doc = take_diag!(
            diagnostics_format,
            g,
            &g.snap.world,
            g.shared_compile()?,
            TypstPagedDocument
        );

        let v = Uint8Array::from(state.update(doc).as_slice()).into();
        Ok(if diagnostics_format != 0 {
            let result = js_sys::Object::new();
            js_sys::Reflect::set(&result, &"result".into(), &v)?;
            result.into()
        } else {
            v
        })
    }
}

struct CompilationDiagnostics {
    errors: Option<EcoVec<typst::diag::SourceDiagnostic>>,
    warnings: Option<EcoVec<typst::diag::SourceDiagnostic>>,
}

impl CompilationDiagnostics {
    fn from_result<T>(result: &Option<Warned<SourceResult<T>>>) -> Self {
        let errors = result
            .as_ref()
            .and_then(|r| r.output.as_ref().map_err(|e| e.clone()).err());
        let warnings = result.as_ref().map(|r| r.warnings.clone());

        Self { errors, warnings }
    }
}

pub struct TDiagnosticsTask<D> {
    diag: CompilationDiagnostics,
    _phantom: std::marker::PhantomData<D>,
}

impl<F: CompilerFeat, D: typst::TypstDocumentTrait + Send + Sync + 'static> WorldComputable<F>
    for TDiagnosticsTask<D>
{
    type Output = Self;

    fn compute(graph: &Arc<WorldComputeGraph<F>>) -> Result<Self> {
        // let paged = graph.compute::<PagedCompilationTask>()?.clone();
        // let html = graph.compute::<HtmlCompilationTask>()?.clone();
        let diag = graph.compute::<CompilationTask<D>>()?;

        Ok(Self {
            diag: CompilationDiagnostics::from_result(&diag),
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<D> TDiagnosticsTask<D> {
    pub fn error_cnt(&self) -> usize {
        self.diag.errors.as_ref().map_or(0, |e| e.len())
    }

    pub fn warning_cnt(&self) -> usize {
        self.diag.warnings.as_ref().map_or(0, |e| e.len())
    }

    pub fn diagnostics(&self) -> impl Iterator<Item = &typst::diag::SourceDiagnostic> {
        self.diag
            .errors
            .iter()
            .chain(self.diag.warnings.iter())
            .flatten()
    }
}

// Convert the input pairs to a dictionary.
fn convert_inputs(inputs: &[js_sys::Array]) -> typst::foundations::Dict {
    inputs
        .iter()
        .map(|j| {
            (
                j.get(0).as_string().unwrap_or_default().into(),
                j.get(1).as_string().into_value(),
            )
        })
        .collect()
}

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    #![allow(clippy::await_holding_lock)]

    use reflexo_vec2svg::MultiVecDocument;
    use sha2::Digest;
    use typst_ts_test_common::web_artifact::get_corpus;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;

    use crate::builder::TypstCompilerBuilder;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    async fn get_source(name: &str) -> Vec<u8> {
        let array_buffer = get_corpus(format!("{}.typ", name)).await.unwrap();
        js_sys::Uint8Array::new(&array_buffer).to_vec()
    }

    async fn get_ir_artifact(name: &str) -> Vec<u8> {
        let array_buffer = get_corpus(format!("{}.artifact.sir.in", name))
            .await
            .unwrap();
        js_sys::Uint8Array::new(&array_buffer).to_vec()
    }

    fn hash_bytes<T: AsRef<[u8]>>(bytes: T) -> String {
        format!("sha256:{}", hex::encode(sha2::Sha256::digest(bytes)))
    }

    fn render_svg(artifact: &[u8]) -> String {
        let doc = MultiSvgDocument::from_slice(artifact);
        type UsingExporter = reflexo_vec2svg::SvgExporter<reflexo_vec2svg::SvgExportFeature>;

        let node = doc.layouts[0].unwrap_single();
        let view = node.pages(&doc.module).unwrap();
        UsingExporter::render_flat_svg(&doc.module, view.pages())
    }

    async fn render_test_template(point: &str, source: &[u8], artifact: &[u8]) {
        let window = web_sys::window().expect("should have a window in this context");
        let performance = window
            .performance()
            .expect("performance should be available");

        let mut compiler = TypstCompilerBuilder::new().unwrap();
        compiler.set_dummy_access_model().await.unwrap();
        let mut compiler = compiler.build().await.unwrap();
        let start = performance.now();
        if !compiler.add_source(
            &format!("/{point}.typ"),
            std::str::from_utf8(source).unwrap(),
            true,
        ) {
            panic!("Failed to add source {point}");
        }
        let end = performance.now();
        let time_used = end - start;

        let browser_artifact = compiler.compile(format!("/{point}.typ")).unwrap();

        let x_svg = render_svg(&browser_artifact);
        let y_svg = render_svg(artifact);

        let x_hash = hash_bytes(&x_svg);
        let y_hash = hash_bytes(&y_svg);

        use base64::Engine;
        let e = base64::engine::general_purpose::STANDARD;
        let x = web_sys::HtmlImageElement::new().unwrap();
        x.set_src(&format!("data:image/svg+xml;base64,{}", e.encode(x_svg)));
        x.set_attribute("style", "flex: 1;").unwrap();
        let y = web_sys::HtmlImageElement::new().unwrap();
        y.set_src(&format!("data:image/svg+xml;base64,{}", e.encode(y_svg)));
        y.set_attribute("style", "flex: 1;").unwrap();

        let div = window
            .document()
            .unwrap()
            .create_element("div")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();

        div.set_attribute("style", "display block; border: 1px solid #000;")
            .unwrap();

        let title = window
            .document()
            .unwrap()
            .create_element("div")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();

        title.set_inner_html(&format!(
            "{point} => {time_used:.3}ms, hash_cmp: {x_hash} v.s. {y_hash}",
        ));

        div.append_child(&title).unwrap();

        let cmp = window
            .document()
            .unwrap()
            .create_element("div")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        cmp.set_attribute("style", "display: flex;").unwrap();

        cmp.append_child(&x).unwrap();
        cmp.append_child(&y).unwrap();

        div.append_child(&cmp).unwrap();

        let body = window.document().unwrap().body().unwrap();

        body.append_child(&div).unwrap();
    }

    async fn render_test_from_corpus(path: &str) {
        let point = path.replace('/', "_");
        let ir_point = format!("{}_artifact_ir", point);

        render_test_template(
            &ir_point,
            &get_source(path).await,
            &get_ir_artifact(path).await,
        )
        .await;
    }

    macro_rules! make_test_point {
        ($name:ident, $($path:literal),+ $(,)?) => {
            #[wasm_bindgen_test]
            async fn $name() {
                $(
                    render_test_from_corpus($path).await;
                )*
            }
        };
    }

    make_test_point!(test_render_math_main, "math/main");
    make_test_point!(test_render_math_undergradmath, "math/undergradmath");
}
