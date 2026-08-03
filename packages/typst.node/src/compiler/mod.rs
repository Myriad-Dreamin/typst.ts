/// Dynamic boxed compiler trait for NodeJS.
pub mod boxed;
/// NodeJS bindings for the compiler.
pub mod node;
/// Wrapped Project compiler.
pub mod project;

use reflexo_typst::package::RegistryPathMapper;

use std::path::PathBuf;
use std::{borrow::Cow, collections::HashMap, fs, path::Path, sync::Arc};

use napi::{bindgen_prelude::*, Either};
use napi_derive::napi;
use rayon::prelude::*;
use reflexo_typst::config::{entry::EntryState, CompileFontOpts};
use reflexo_typst::error::prelude::{Result, WithContext};
use reflexo_typst::font::system::SystemFontSearcher;
use reflexo_typst::font_data::decode_font_data;
use reflexo_typst::package::registry::HttpRegistry;
use reflexo_typst::typst::{foundations::IntoValue, LazyHash};
use reflexo_typst::vfs::{system::SystemAccessModel, Vfs};
use reflexo_typst::{Bytes, Features, TypstDict, TypstSystemUniverse};

pub use boxed::BoxedCompiler;
pub use node::*;
pub use project::*;

/// A nullable boxed compiler wrapping.
///
/// This is for transferring boxed compiler between functions.
/// It will panic if the inner boxed compiler is already taken.
#[napi(js_name = "BoxedCompiler")]
pub struct JsBoxedCompiler(Option<BoxedCompiler>);

impl JsBoxedCompiler {
    pub fn assert_ref(&self) -> &BoxedCompiler {
        self.0.as_ref().expect("moved box compiler")
    }
    pub fn assert_mut(&mut self) -> &mut BoxedCompiler {
        self.0.as_mut().expect("moved box compiler")
    }

    /// Takes the inner compiler from the wrapper.
    pub fn grab(&mut self) -> BoxedCompiler {
        self.0.take().expect("moved box compiler")
    }
}

impl<T> From<T> for JsBoxedCompiler
where
    T: Into<BoxedCompiler>,
{
    fn from(t: T) -> Self {
        Self(Some(t.into()))
    }
}

impl From<Option<BoxedCompiler>> for JsBoxedCompiler {
    fn from(t: Option<BoxedCompiler>) -> Self {
        Self(t)
    }
}

#[napi(object)]
pub struct NodeAddFontPaths {
    /// Adds additional directories to search for fonts
    pub font_paths: Vec<String>,
}

#[napi(object)]
pub struct NodeAddFontBlobs {
    /// Adds additional memory fonts
    pub font_blobs: Vec<Buffer>,
}

#[napi(object, js_name = "CompileArgs")]
#[derive(Default)]
pub struct CompileArgs {
    /// Adds additional directories to search for fonts
    pub font_args: Option<Vec<Either<NodeAddFontPaths, NodeAddFontBlobs>>>,

    /// Path to typst workspace.
    pub workspace: Option<String>,

    /// Adds a string key-value pair visible through `sys.inputs`
    pub inputs: Option<HashMap<String, String>>,
}

fn is_woff2_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("woff2"))
}

fn load_woff2_file(path: &Path) -> Result<Bytes> {
    let data = fs::read(path)
        .map_err(|err| anyhow::anyhow!("failed to read WOFF2 font {}: {err}", path.display()))?;
    Ok(Bytes::new(
        decode_font_data(data).map_err(anyhow::Error::msg)?,
    ))
}

fn load_woff2_dir(path: &Path) -> Result<Vec<Bytes>> {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| match entry {
            Ok(entry) if entry.file_type().is_file() && is_woff2_path(entry.path()) => {
                Some(load_woff2_file(entry.path()))
            }
            Ok(_) => None,
            Err(err) => Some(Err(anyhow::Error::new(err).into())),
        })
        .collect()
}

pub fn abs_user_path(path: &str) -> Result<PathBuf> {
    use reflexo_typst::path::PathClean;
    let path = Path::new(path).clean();

    let path = if path.is_absolute() {
        path
    } else {
        let cwd = std::env::current_dir().context("failed to get current dir")?;
        cwd.join(path)
    };

    Ok(path.clean())
}

pub fn create_universe(args: Option<CompileArgs>) -> Result<TypstSystemUniverse> {
    let args = args.unwrap_or_default();
    let workspace_dir = abs_user_path(args.workspace.unwrap_or_default().as_str())?;

    let mut searcher = SystemFontSearcher::new();
    let mut memory_fonts = Vec::new();

    for arg in args.font_args.into_iter().flatten() {
        match arg {
            Either::A(p) => {
                for i in p.font_paths {
                    let path = Path::new(&i);
                    if path.is_dir() {
                        searcher.search_dir(path);
                        memory_fonts.extend(load_woff2_dir(path)?);
                    } else if is_woff2_path(path) {
                        searcher.font_paths.push(abs_user_path(&i)?);
                        memory_fonts.push(load_woff2_file(path)?);
                    } else {
                        let _ = searcher.search_file(path);
                    }
                }
            }
            Either::B(p) => {
                for b in p.font_blobs {
                    memory_fonts.push(Bytes::new(
                        decode_font_data(b.to_vec()).map_err(anyhow::Error::msg)?,
                    ));
                }
            }
        }
    }

    searcher.flush();
    searcher.add_memory_fonts(memory_fonts.into_par_iter());

    searcher.resolve_opts(CompileFontOpts {
        with_embedded_fonts: typst_ts_cli::font::fonts().map(Cow::Borrowed).collect(),
        ..CompileFontOpts::default()
    })?;

    let registry = Arc::new(HttpRegistry::default());
    let resolver = Arc::new(RegistryPathMapper::new(registry.clone()));
    let verse = TypstSystemUniverse::new_raw(
        EntryState::new_rooted(workspace_dir.into(), None),
        Features::default(),
        args.inputs.map(create_inputs),
        Vfs::new(resolver, SystemAccessModel {}),
        registry,
        Arc::new(searcher.build()),
        None,
    );

    Ok(verse)
}

/// Convert the input pairs to a dictionary.
fn create_inputs(inputs: HashMap<String, String>) -> Arc<LazyHash<TypstDict>> {
    Arc::new(LazyHash::new(
        inputs
            .iter()
            .map(|(k, v)| (k.as_str().into(), v.as_str().into_value()))
            .collect(),
    ))
}

#[cfg(test)]
mod font_data_tests {
    use reflexo_typst::font::cache::FontInfoCache;
    use reflexo_typst::font_data::decode_font_data;

    #[test]
    fn decodes_woff2_font_data() {
        let compressed = include_bytes!("../../../compiler/tests/fixtures/roboto.woff2");
        let decoded = decode_font_data(compressed.to_vec()).unwrap();

        assert_eq!(&decoded[..4], b"\0\x01\0\0");
        assert_eq!(FontInfoCache::from_data(&decoded).info.len(), 1);
    }

    #[test]
    fn rejects_invalid_woff2_font_data() {
        let error = decode_font_data(b"wOF2 invalid".to_vec()).unwrap_err();

        assert!(error.to_string().contains("failed to decode WOFF2 font:"));
    }
}
