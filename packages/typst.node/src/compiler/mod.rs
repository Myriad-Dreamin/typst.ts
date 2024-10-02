/// Dynamic boxed compiler trait for NodeJS.
pub mod boxed;

pub use boxed::{BoxedCompiler, NodeCompilerTrait};

use std::{borrow::Cow, collections::HashMap, path::Path, sync::Arc};

use napi::{bindgen_prelude::*, Either};
use napi_derive::napi;
use reflexo_typst::config::{entry::EntryState, CompileFontOpts};
use reflexo_typst::error::prelude::*;
use reflexo_typst::font::system::SystemFontSearcher;
use reflexo_typst::package::http::HttpRegistry;
use reflexo_typst::vfs::{system::SystemAccessModel, Vfs};
use reflexo_typst::{compat::LazyHash, typst::foundations::IntoValue};
use reflexo_typst::{
    Bytes, CompileDriver, PureCompiler, TypstDict, TypstSystemUniverse, TypstSystemWorld,
};

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
pub struct NodeCompileArgs {
    /// Adds additional directories to search for fonts
    pub font_args: Option<Vec<Either<NodeAddFontPaths, NodeAddFontBlobs>>>,

    /// Path to typst workspace.
    pub workspace: Option<String>,

    /// Adds a string key-value pair visible through `sys.inputs`
    pub inputs: Option<HashMap<String, String>>,
}

pub fn create_driver(
    args: Option<NodeCompileArgs>,
) -> ZResult<CompileDriver<PureCompiler<TypstSystemWorld>>> {
    use reflexo_typst::path::PathClean;
    let args = args.unwrap_or_default();
    let workspace_dir = Path::new(args.workspace.unwrap_or_default().as_str()).clean();

    let workspace_dir = if workspace_dir.is_absolute() {
        workspace_dir
    } else {
        let cwd = std::env::current_dir().context("failed to get current dir")?;
        cwd.join(workspace_dir)
    };

    let workspace_dir = workspace_dir.clean();

    let mut searcher = SystemFontSearcher::new();

    for arg in args.font_args.into_iter().flatten() {
        match arg {
            Either::A(p) => {
                for i in p.font_paths {
                    let path = Path::new(&i);
                    if path.is_dir() {
                        searcher.search_dir(path);
                    } else {
                        let _ = searcher.search_file(path);
                    }
                }
            }
            Either::B(p) => {
                for b in p.font_blobs {
                    searcher.add_memory_font(Bytes::from(b.to_vec()));
                }
            }
        }
    }

    searcher.resolve_opts(CompileFontOpts {
        with_embedded_fonts: typst_ts_cli::font::fonts().map(Cow::Borrowed).collect(),
        ..CompileFontOpts::default()
    })?;

    let world = TypstSystemUniverse::new_raw(
        EntryState::new_rooted(workspace_dir.into(), None),
        args.inputs.map(create_inputs),
        Vfs::new(SystemAccessModel {}),
        HttpRegistry::default(),
        Arc::new(searcher.into()),
    );

    Ok(CompileDriver::new(std::marker::PhantomData, world))
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
