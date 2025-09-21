pub mod builder;

#[cfg(feature = "incr")]
mod incr;
pub(crate) mod utils;

pub use crate::builder::TypstFontResolver;
pub use reflexo_typst::*;

use core::fmt;
use std::{fmt::Write, path::Path, sync::Arc};

use error::TypstSourceDiagnostic;
use font::cache::FontInfoCache;
use js_sys::{Array, JsString, Uint8Array};
use reflexo_typst::error::{long_diag_from_std, DiagMessage};
use reflexo_typst::package::registry::JsRegistry;
use reflexo_typst::prelude::EcoVec;
use reflexo_typst::typst::diag::{SourceResult, Warned};
use reflexo_typst::typst::foundations::IntoValue;
use reflexo_typst::vfs::browser::ProxyAccessModel;
use wasm_bindgen::prelude::*;

use crate::font::FontResolverImpl;
use crate::utils::console_log;
#[cfg(feature = "incr")]
use incr::IncrServer;

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
    has_error: bool,
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
    js_sys::Reflect::set(&res, &"hasError".into(), &has_error.into()).unwrap();
    js_sys::Reflect::set(&res, &"diagnostics".into(), &diag).unwrap();
    res.into()
}

#[wasm_bindgen]
pub struct TypstCompiler {
    pub(crate) verse: TypstBrowserUniverse,
}

impl TypstCompiler {
    pub fn new(
        access_model: ProxyAccessModel,
        registry: JsRegistry,
        fonts: FontResolverImpl,
    ) -> Result<Self, JsValue> {
        Ok(Self {
            verse: TypstBrowserUniverse::new(
                std::path::Path::new("/").to_owned(),
                None,
                access_model,
                registry,
                fonts,
            ),
        })
    }
}

/// @deprecated use TypstFontResolverBuilder instead
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

    pub fn set_fonts(&mut self, fonts: &TypstFontResolver) -> Result<(), JsValue> {
        self.verse
            .increment_revision(|verse| verse.set_fonts(fonts.fonts.clone()));
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
        self.compile(None, None, fmt, diagnostics_format)
    }

    pub fn compile(
        &mut self,
        main_file_path: Option<String>,
        inputs: Option<Vec<js_sys::Array>>,
        fmt: String,
        diagnostics_format: u8,
    ) -> Result<JsValue, JsValue> {
        let mut w = self.snapshot(None, main_file_path, inputs)?;
        let fmt = match fmt.as_str() {
            "vector" => 0u8,
            "pdf" => 1,
            "_dummy" => 2,
            _ => return Err(error_once!("Unsupported fmt", fmt: fmt).into()),
        };
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
        let _doc = w.compile(0, 0)?;
        w.query(0, selector, field)
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

type CFlag<D> = FlagTask<CompilationTask<D>>;
type PagedCFlag = CFlag<reflexo_typst::TypstPagedDocument>;
type HtmlCFlag = CFlag<reflexo_typst::TypstHtmlDocument>;

#[wasm_bindgen]
pub struct TypstCompileWorld {
    graph: Arc<WorldComputeGraph<BrowserCompilerFeat>>,
}

#[wasm_bindgen]
impl TypstCompileWorld {
    pub fn compile(&mut self, kind: u8, diagnostics_format: u8) -> Result<JsValue, JsValue> {
        match kind {
            0 => {
                self.do_compile_paged()?;
                self.get_diag::<TypstPagedDocument>(diagnostics_format)
            }
            1 => {
                self.do_compile_html()?;
                self.get_diag::<TypstHtmlDocument>(diagnostics_format)
            }
            _ => Err(error_once!("invalid kind", kind: kind).into()),
        }
    }

    pub fn title(&self, kind: u8) -> Result<Option<String>, JsValue> {
        Ok(self
            .get_doc(kind)?
            .and_then(|doc| Some(doc.info().title.as_ref()?.to_string())))
    }

    pub fn get_artifact(&mut self, fmt: u8, diagnostics_format: u8) -> Result<JsValue, JsValue> {
        #[cfg(feature = "svg")]
        use reflexo_vec2svg::DefaultExportFeature;
        #[cfg(feature = "svg")]
        type SvgModuleExport = WebSvgModuleExport<DefaultExportFeature>;
        #[cfg(feature = "pdf")]
        use reflexo_typst::task::ExportPdfTask;

        let Some(doc) = self.do_compile_paged()? else {
            return self.get_diag::<TypstPagedDocument>(diagnostics_format);
        };
        let artifact_bytes: Bytes = match fmt {
            #[cfg(feature = "svg")]
            0 => SvgModuleExport::run(&self.graph, &doc, &ExportWebSvgModuleTask::default())?,
            #[cfg(feature = "pdf")]
            1 => PdfExport::run(&self.graph, &doc, &ExportPdfTask::default())?,
            2 => Bytes::new([]),
            _ => {
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

    pub fn query(
        &mut self,
        kind: u8,
        selector: String,
        field: Option<String>,
    ) -> Result<String, JsValue> {
        // todo: diagnostics
        let doc = self
            .get_doc(kind)?
            .ok_or_else(|| error_once!("document is not compiled"))?;

        // todo: query snapshot.query should directly return a error?
        let elements: Vec<typst::foundations::Content> =
            reflexo_typst::query::retrieve(&self.graph.snap.world, &selector, &doc)
                .map_err(|e| JsValue::from(e.as_str()))?;

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
        let Some(doc) = self.do_compile_paged()? else {
            return self.get_diag::<TypstPagedDocument>(diagnostics_format);
        };
        let v = Uint8Array::from(state.update(doc).as_slice()).into();
        Ok(if diagnostics_format != 0 {
            let result = js_sys::Object::new();
            js_sys::Reflect::set(&result, &"result".into(), &v)?;
            result.into()
        } else {
            v
        })
    }

    fn get_diag<D: TypstDocumentTrait + Send + Sync + 'static>(
        &self,
        diagnostics_format: u8,
    ) -> Result<JsValue, JsValue> {
        let diag = self.graph.compute::<TDiagnosticsTask<D>>()?;
        if diagnostics_format >= 2 {
            Ok(convert_diag(
                diag.diagnostics(),
                Some(&self.graph.snap.world),
                diag.error_cnt() > 0,
                diagnostics_format,
            ))
        } else if diag.error_cnt() > 0 {
            let diag = diag.diagnostics().collect::<Vec<_>>();
            return Err(format!("{diag:?}").into());
        } else {
            Ok(JsValue::UNDEFINED)
        }
    }

    fn do_compile_html(&mut self) -> Result<Option<Arc<TypstHtmlDocument>>, JsValue> {
        let g = &self.graph;
        let _ = g.provide::<HtmlCFlag>(Ok(FlagTask::flag(true)));
        Ok(g.shared_compile_html()?)
    }

    fn do_compile_paged(&mut self) -> Result<Option<Arc<TypstPagedDocument>>, JsValue> {
        let g = &self.graph;
        let _ = g.provide::<PagedCFlag>(Ok(FlagTask::flag(true)));
        Ok(g.shared_compile()?)
    }

    fn get_doc(&self, kind: u8) -> Result<Option<TypstDocument>, JsValue> {
        Ok(match kind {
            0 => self
                .get_doc_t::<TypstPagedDocument>()?
                .map(TypstDocument::Paged),
            1 => self
                .get_doc_t::<TypstHtmlDocument>()?
                .map(TypstDocument::Html),
            _ => return Err(error_once!("invalid kind", kind: kind).into()),
        })
    }

    fn get_doc_t<D: TypstDocumentTrait + Send + Sync + 'static>(
        &self,
    ) -> Result<Option<Arc<D>>, JsValue> {
        // todo: don't coupled me with compilation.
        self.graph
            .get::<CFlag<D>>()
            .ok_or_else(|| error_once!("document is not compiled"))??;
        Ok(self
            .graph
            .get::<OptionDocumentTask<D>>()
            .ok_or_else(|| error_once!("document did not compile"))??
            .as_ref()
            .clone())
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
