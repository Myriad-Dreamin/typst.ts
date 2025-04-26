/// Compiler trait for NodeJS.
pub mod compiler;

/// Error handling for NodeJS.
pub mod error;

pub use compiler::*;
pub use error::{map_node_error, NodeError};

use std::{collections::HashMap, sync::Arc};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use reflexo_typst::WorldComputeGraph;
use reflexo_typst::{SystemCompilerFeat, TypstDatetime, TypstDocument};
use serde::{Deserialize, Serialize};

/// A shared typst document object.
#[napi]
#[derive(Clone)]
pub struct NodeTypstDocument {
    /// The cache of exports.
    pub(crate) graph: Arc<WorldComputeGraph<SystemCompilerFeat>>,
    /// Inner document.
    pub(crate) doc: TypstDocument,
}

#[napi]
impl NodeTypstDocument {
    /// Gets the number of pages in the document.
    #[napi(getter)]
    pub fn num_of_pages(&self) -> u32 {
        self.doc.num_of_pages()
    }

    /// Gets the title of the document.
    #[napi(getter)]
    pub fn title(&self) -> Option<String> {
        self.doc.info().title.as_ref().map(ToString::to_string)
    }

    /// Gets the authors of the document.
    #[napi(getter)]
    pub fn authors(&self) -> Option<Vec<String>> {
        let authors = self.doc.info().author.iter();
        Some(authors.map(ToString::to_string).collect::<Vec<_>>())
    }

    /// Gets the keywords of the document.
    #[napi(getter)]
    pub fn keywords(&self) -> Option<Vec<String>> {
        let keywords = self.doc.info().keywords.iter();
        Some(keywords.map(ToString::to_string).collect::<Vec<_>>())
    }

    /// Gets the unix timestamp (in nanoseconds) of the document.
    ///
    /// Note: currently typst doesn't specify the timezone of the date, and we
    /// keep stupid and doesn't add timezone info to the date.
    #[napi(getter)]
    pub fn date(&self) -> Option<i64> {
        let datetime = self.doc.info().date.custom().flatten();
        datetime.and_then(typst_datetime_to_unix_nanoseconds)
    }

    /// Determines whether the date should be automatically generated.
    ///
    /// This happens when user specifies `date: auto` in the document
    /// explicitly.
    #[napi(getter)]
    pub fn enabled_auto_date(&self) -> bool {
        self.doc.info().date.is_auto()
    }
}

/// Converts a typst datetime to unix nanoseconds.
fn typst_datetime_to_unix_nanoseconds(datetime: TypstDatetime) -> Option<i64> {
    let year = datetime.year().unwrap_or_default();
    let month = datetime.month().unwrap_or_default() as u32;
    let day = datetime.day().unwrap_or_default() as u32;
    let hour = datetime.hour().unwrap_or_default() as u32;
    let minute = datetime.minute().unwrap_or_default() as u32;
    let second = datetime.second().unwrap_or_default() as u32;

    let date = chrono::NaiveDate::from_ymd_opt(year, month, day)?;
    let time = chrono::NaiveTime::from_hms_opt(hour, minute, second)?;

    let datetime = chrono::NaiveDateTime::new(date, time);

    datetime.and_utc().timestamp_nanos_opt()
}

/// A shared HTML output object.
#[napi]
#[derive(Clone)]
pub struct NodeHtmlOutput {
    inner: Arc<reflexo_typst::HtmlOutput>,
}

#[napi]
impl NodeHtmlOutput {
    /// Gets the title of the document.
    #[napi]
    pub fn title(&self) -> Option<String> {
        self.inner.title().map(ToString::to_string)
    }

    /// Gets the description of the document.
    #[napi]
    pub fn description(&self) -> Option<String> {
        self.inner.description().map(ToString::to_string)
    }

    /// Gets the body of the document.
    #[napi]
    pub fn body(&self) -> String {
        self.inner.body().to_string()
    }

    /// Gets the body of the document as bytes.
    #[napi]
    pub fn body_bytes(&self) -> Buffer {
        self.inner.body().to_string().into()
    }

    /// Gets the HTML of the document.
    #[napi]
    pub fn html(&self) -> String {
        self.inner.html().to_string()
    }

    /// Gets the HTML of the document as bytes.
    #[napi]
    pub fn html_bytes(&self) -> Buffer {
        self.inner.html().into()
    }
}

/// The arguments to compile a document.
///
/// If no `mainFileContent` or `mainFilePath` is specified, the compiler will
/// use the entry file specified in the constructor of `NodeCompiler`.
#[napi(object)]
#[derive(Serialize, Deserialize, Debug)]
pub struct CompileDocArgs {
    /// Specifies the main file content.
    /// Exclusive with `mainFilePath`.
    #[serde(rename = "mainFileContent")]
    pub main_file_content: Option<String>,

    /// Specifies path to the entry file.
    /// Exclusive with `mainFileContent`.
    #[serde(rename = "mainFilePath")]
    pub main_file_path: Option<String>,

    /// Passes `sys.inputs` as is in format of string key-value pairs.
    pub inputs: Option<HashMap<String, String>>,

    /// (Experimental) Whether to reset the cache before compilation.
    pub reset_read: Option<bool>,
}

/// Arguments to query the document.
#[napi(object)]
#[derive(Serialize, Deserialize, Debug)]
pub struct QueryDocArgs {
    /// The query selector.
    pub selector: String,
    /// An optional field to select on the element of the resultants.
    pub field: Option<String>,
}

/// Arguments to render a PDF.
#[napi(object)]
#[derive(Serialize, Deserialize, Debug)]
#[cfg(feature = "pdf")]
pub struct RenderPdfOpts {
    /// (Experimental) An optional PDF standard to be used to export PDF.
    ///
    /// Please check {@link types.PdfStandard} for a non-exhaustive list of
    /// standards.
    pub pdf_standard: Option<String>,

    /// An optional (creation) timestamp to be used to export PDF.
    ///
    /// This is used when you *enable auto timestamp* in the document.
    pub creation_timestamp: Option<i64>,
}
