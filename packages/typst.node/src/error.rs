use core::fmt;
use std::{cell::OnceCell, fmt::Write, sync::Arc};

use napi_derive::napi;
use reflexo_typst::error::{long_diag_from_std, prelude::WithContext, TypstSourceDiagnostic};
use reflexo_typst::typst::diag::Warned;
use reflexo_typst::typst::prelude::*;
use reflexo_typst::{TypstPagedDocument, TypstWorld};

use crate::NodeTypstPagedDocument;

/// The error status of a node error.
pub enum NodeErrorStatus {
    Raw(napi::Error),
    Error(reflexo_typst::error::Error),
    Diagnostics(EcoVec<TypstSourceDiagnostic>),
}

impl fmt::Display for NodeErrorStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeErrorStatus::Raw(e) => write!(f, "{e}"),
            NodeErrorStatus::Error(e) => write!(f, "{e}"),
            NodeErrorStatus::Diagnostics(diagnostics) => {
                let mut linefeed = false;
                for (idx, diagnostic) in diagnostics.iter().enumerate() {
                    if linefeed {
                        f.write_char('\n')?;
                    } else {
                        linefeed = true;
                    }
                    write!(f, "{idx}: {}", diagnostic.message)?;
                }
                Ok(())
            }
        }
    }
}

/// A node error.
#[napi]
pub struct NodeError(OnceCell<String>, NodeErrorStatus);

#[napi]
impl NodeError {
    /// Gets the kind of the error.
    #[napi(getter)]
    pub fn kind(&self) -> String {
        match &self.1 {
            NodeErrorStatus::Raw(_) => "raw".to_string(),
            NodeErrorStatus::Error(_) => "error".to_string(),
            NodeErrorStatus::Diagnostics(_) => "diagnostics".to_string(),
        }
    }

    /// Gets the short diagnostics of the error.
    ///
    /// To retrieve the full diagnostics, please use
    /// `NodeCompiler.fetch_diagnostics`.
    #[napi(getter)]
    pub fn short_diagnostics(&self) -> napi::Result<Vec<serde_json::Value>, NodeError> {
        self.get_json_diagnostics(None)
    }

    /// Gets the compilation status
    ///
    /// If the error is an error, it will return `internal_error`.
    ///
    /// Otherwise, if diagnostics contains any error, it will return `error`.
    ///
    /// Otherwise, if diagnostics contains any warning, it will return
    /// `warning`.
    ///
    /// Otherwise, it will return `ok`.
    #[napi(getter)]
    pub fn compilation_status(&self) -> String {
        let stat = match &self.1 {
            NodeErrorStatus::Raw(_) | NodeErrorStatus::Error(_) => "internal_error",
            NodeErrorStatus::Diagnostics(_) => {
                let (has_error, has_warning) = self.get_diagnostics().fold(
                    (false, false),
                    |(has_error, has_warning), diag| {
                        (
                            has_error
                                || diag.severity == reflexo_typst::typst::diag::Severity::Error,
                            has_warning
                                || diag.severity == reflexo_typst::typst::diag::Severity::Warning,
                        )
                    },
                );

                if has_error {
                    "error"
                } else if has_warning {
                    "warning"
                } else {
                    "ok"
                }
            }
        };

        stat.to_owned()
    }

    pub fn get_json_diagnostics(
        &self,
        world: Option<&dyn TypstWorld>,
    ) -> napi::Result<Vec<serde_json::Value>, NodeError> {
        self.get_diagnostics()
            .flat_map(move |e| long_diag_from_std(e, world))
            .map(serde_json::to_value)
            .collect::<Result<_, _>>()
            .context("failed to serialize diagnostics")
            .map_err(map_node_error)
    }

    pub fn get_diagnostics<'a>(&self) -> impl Iterator<Item = TypstSourceDiagnostic> + 'a {
        let iter = match &self.1 {
            NodeErrorStatus::Raw(_) | NodeErrorStatus::Error(_) => None,
            NodeErrorStatus::Diagnostics(diagnostics) => Some(diagnostics.clone().into_iter()),
        };

        iter.into_iter().flatten()
    }
}

impl From<napi::Error> for NodeError {
    fn from(e: napi::Error) -> Self {
        NodeError(OnceCell::new(), NodeErrorStatus::Raw(e))
    }
}

impl From<reflexo_typst::error::Error> for NodeError {
    fn from(e: reflexo_typst::error::Error) -> Self {
        NodeError(OnceCell::new(), NodeErrorStatus::Error(e))
    }
}

impl From<EcoVec<TypstSourceDiagnostic>> for NodeError {
    fn from(e: EcoVec<TypstSourceDiagnostic>) -> Self {
        NodeError(OnceCell::new(), NodeErrorStatus::Diagnostics(e))
    }
}

impl AsRef<str> for NodeError {
    fn as_ref(&self) -> &str {
        self.0.get_or_init(|| self.1.to_string())
    }
}

// |e| napi::Error::from_status(NodeError::new(e))
pub fn map_node_error(e: impl Into<NodeError>) -> napi::Error<NodeError> {
    let e = e.into();
    let reason = e.as_ref().to_owned();
    napi::Error::new(e, reason)
}

/// Result of single typst compilation.
#[napi]
pub struct NodeTypstCompileResult {
    result: Option<NodeTypstPagedDocument>,
    // todo: better warning structure
    warnings: Option<NodeError>,
    error: Option<NodeError>,
}

#[napi]
impl NodeTypstCompileResult {
    /// Gets the result of compilation.
    #[napi(getter)]
    pub fn result(&self) -> Option<NodeTypstPagedDocument> {
        self.result.clone()
    }

    /// Takes the result of compilation.
    #[napi]
    pub fn take_warnings(&mut self) -> Option<NodeError> {
        self.warnings.take()
    }

    /// Takes the diagnostics of compilation.
    #[napi]
    pub fn take_diagnostics(&mut self) -> Option<NodeError> {
        self.error.take()
    }
}

impl<E> From<Result<Warned<Arc<TypstPagedDocument>>, E>> for NodeTypstCompileResult
where
    E: Into<NodeError>,
{
    fn from(res: Result<Warned<Arc<TypstPagedDocument>>, E>) -> Self {
        match res {
            Ok(result) => NodeTypstCompileResult {
                result: Some(NodeTypstPagedDocument(result.output)),
                warnings: if result.warnings.is_empty() {
                    None
                } else {
                    Some(result.warnings.into())
                },
                error: None,
            },
            Err(e) => NodeTypstCompileResult {
                result: None,
                warnings: None,
                error: Some(e.into()),
            },
        }
    }
}
