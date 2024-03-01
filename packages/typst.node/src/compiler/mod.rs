/// Dynamic boxed compiler trait for NodeJS.
pub mod boxed;

pub use boxed::{BoxedCompiler, NodeCompilerTrait};

use std::{borrow::Cow, collections::HashMap, path::Path, sync::Arc};

use napi::Either;
use napi_derive::napi;
use typst_ts_compiler::{
    font::system::SystemFontSearcher,
    package::http::HttpRegistry,
    service::CompileDriver,
    vfs::{system::SystemAccessModel, Vfs},
    TypstSystemWorld,
};
use typst_ts_core::{
    config::CompileOpts,
    error::prelude::*,
    typst::{foundations::IntoValue, prelude::Prehashed},
    Bytes, TypstDict,
};

/// let [`CompileDriver`] boxable.
impl NodeCompilerTrait for CompileDriver {}

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
    pub font_blobs: Vec<Vec<u8>>,
}

#[napi(object, js_name = "CompileArgs")]
#[derive(Default)]
pub struct NodeCompileArgs {
    /// Adds additional directories to search for fonts
    pub font_args: Vec<Either<NodeAddFontPaths, NodeAddFontBlobs>>,

    /// Path to typst workspace.
    pub workspace: String,

    /// Entry file.
    pub entry: String,

    /// Adds a string key-value pair visible through `sys.inputs`
    pub inputs: HashMap<String, String>,
}

pub fn create_driver(args: NodeCompileArgs) -> ZResult<CompileDriver> {
    use typst_ts_core::path::PathClean;
    let workspace_dir = Path::new(args.workspace.as_str()).clean();
    let entry_file_path = Path::new(args.entry.as_str()).clean();

    let workspace_dir = if workspace_dir.is_absolute() {
        workspace_dir
    } else {
        let cwd = std::env::current_dir().context("failed to get current dir")?;
        cwd.join(workspace_dir)
    };

    let entry_file_path = if entry_file_path.is_absolute() {
        entry_file_path
    } else {
        let cwd = std::env::current_dir().context("failed to get current dir")?;
        cwd.join(entry_file_path)
    };

    let workspace_dir = workspace_dir.clean();
    let entry_file_path = entry_file_path.clean();

    if !entry_file_path.starts_with(&workspace_dir) {
        return Err(error_once!(
            "entry file path must be in workspace directory",
            workspace_dir: workspace_dir.display()
        ));
    }

    // Convert the input pairs to a dictionary.
    let inputs: TypstDict = args
        .inputs
        .iter()
        .map(|(k, v)| (k.as_str().into(), v.as_str().into_value()))
        .collect();

    let mut searcher = SystemFontSearcher::new();

    for arg in args.font_args {
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
                    searcher.add_memory_font(Bytes::from(b));
                }
            }
        }
    }

    searcher.resolve_opts(CompileOpts {
        with_embedded_fonts: typst_ts_cli::font::fonts().map(Cow::Borrowed).collect(),
        ..CompileOpts::default()
    })?;

    let mut world = TypstSystemWorld::new_raw(
        workspace_dir.clone(),
        Vfs::new(SystemAccessModel {}),
        HttpRegistry::default(),
        searcher.into(),
    );
    world.set_inputs(Arc::new(Prehashed::new(inputs)));

    Ok(CompileDriver {
        world,
        entry_file: entry_file_path.to_owned(),
    })
}
