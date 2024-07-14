/// Dynamic boxed compiler trait for NodeJS.
pub mod boxed;

pub use boxed::{BoxedCompiler, NodeCompilerTrait};

use std::{borrow::Cow, collections::HashMap, path::Path, sync::Arc};

use napi::{bindgen_prelude::*, Either};
use napi_derive::napi;
use typst_ts_compiler::{
    font::system::SystemFontSearcher,
    package::http::HttpRegistry,
    vfs::{system::SystemAccessModel, Vfs},
    CompileDriver, PureCompiler, TypstSystemUniverse, TypstSystemWorld,
};
use typst_ts_core::{
    config::{compiler::EntryState, CompileFontOpts},
    error::prelude::*,
    typst::{foundations::IntoValue, prelude::Prehashed},
    Bytes,
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
    use typst_ts_core::path::PathClean;
    let args = args.unwrap_or_default();
    let workspace_dir = Path::new(args.workspace.unwrap_or_default().as_str()).clean();

    let workspace_dir = if workspace_dir.is_absolute() {
        workspace_dir
    } else {
        let cwd = std::env::current_dir().context("failed to get current dir")?;
        cwd.join(workspace_dir)
    };

    let workspace_dir = workspace_dir.clean();

    // Convert the input pairs to a dictionary.
    let inputs = args.inputs.map(|inputs| {
        Arc::new(Prehashed::new(
            inputs
                .iter()
                .map(|(k, v)| (k.as_str().into(), v.as_str().into_value()))
                .collect(),
        ))
    });

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
        inputs,
        Vfs::new(SystemAccessModel {}),
        HttpRegistry::default(),
        Arc::new(searcher.into()),
    );

    Ok(CompileDriver::new(std::marker::PhantomData, world))
}
