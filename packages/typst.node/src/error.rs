#![allow(missing_docs)]
#![allow(unused)]

use core::fmt;
use std::{cell::OnceCell, fmt::Write, sync::Arc};

use napi_derive::napi;
use reflexo_typst::diag::print_diagnostics;
use reflexo_typst::error::{long_diag_from_std, prelude::WithContext, TypstSourceDiagnostic};
use reflexo_typst::system::SystemWorldComputeGraph;
use reflexo_typst::typst::diag::{SourceResult, Warned};
use reflexo_typst::typst::prelude::*;
use reflexo_typst::{DiagnosticFormat, TypstPagedDocument, TypstWorld};

use crate::{NodeHtmlOutput, NodeTypstDocument};

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
                let (has_error, has_warning) = self.iter_diagnostics().fold(
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

    pub fn diagnostics(&self) -> Option<&EcoVec<TypstSourceDiagnostic>> {
        match &self.1 {
            NodeErrorStatus::Raw(_) | NodeErrorStatus::Error(_) => None,
            NodeErrorStatus::Diagnostics(diagnostics) => Some(diagnostics),
        }
    }

    pub fn iter_diagnostics(&self) -> impl Iterator<Item = &TypstSourceDiagnostic> + '_ {
        self.diagnostics().into_iter().flatten()
    }

    pub fn get_json_diagnostics(
        &self,
        world: Option<&dyn TypstWorld>,
    ) -> napi::Result<Vec<serde_json::Value>, NodeError> {
        self.iter_diagnostics()
            .flat_map(move |e| long_diag_from_std(e, world))
            .map(serde_json::to_value)
            .collect::<Result<_, _>>()
            .context("failed to serialize diagnostics")
            .map_err(map_node_error)
    }

    pub fn error_message(&self) -> Option<&str> {
        match &self.1 {
            NodeErrorStatus::Raw(..) | NodeErrorStatus::Error(..) => Some(self.as_ref()),
            NodeErrorStatus::Diagnostics(_) => None,
        }
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

/// Result of single typst execution.
pub struct ExecResultRepr<T> {
    result: Option<T>,
    graph: Option<Arc<SystemWorldComputeGraph>>,
    warnings: Option<Box<NodeError>>,
    error: Option<Box<NodeError>>,
}

impl<T> From<T> for ExecResultRepr<T> {
    fn from(result: T) -> Self {
        Self {
            result: Some(result),
            graph: None,
            warnings: None,
            error: None,
        }
    }
}

impl<T> From<Warned<SourceResult<T>>> for ExecResultRepr<T> {
    fn from(warned: Warned<SourceResult<T>>) -> Self {
        let warnings = if warned.warnings.is_empty() {
            None
        } else {
            Some(NodeError::from(warned.warnings.clone()))
        };
        match warned.output {
            Ok(result) => Self {
                graph: None,
                result: Some(result),
                warnings: warnings.map(Box::new),
                error: None,
            },
            Err(err) => Self {
                graph: None,
                result: None,
                warnings: warnings.map(Box::new),
                error: Some(Box::new(err.into())),
            },
        }
    }
}

impl<T> From<Result<T, NodeError>> for ExecResultRepr<T> {
    fn from(result: Result<T, NodeError>) -> Self {
        match result {
            Ok(result) => Self {
                graph: None,
                result: Some(result),
                warnings: None,
                error: None,
            },
            Err(error) => Self {
                graph: None,
                result: None,
                warnings: None,
                error: Some(Box::new(error)),
            },
        }
    }
}

impl<T> ExecResultRepr<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> ExecResultRepr<U> {
        ExecResultRepr {
            graph: self.graph,
            result: self.result.map(f),
            warnings: self.warnings,
            error: self.error,
        }
    }

    pub fn and_then<U, F: FnOnce(T) -> Result<U, NodeError>>(self, f: F) -> ExecResultRepr<U> {
        match self.result {
            Some(result) => match f(result) {
                Ok(result) => ExecResultRepr {
                    graph: self.graph,
                    result: Some(result),
                    warnings: self.warnings,
                    error: self.error,
                },
                Err(error) => ExecResultRepr {
                    graph: self.graph,
                    result: None,
                    warnings: self.warnings,
                    error: Some(Box::new(error)),
                },
            },
            None => ExecResultRepr {
                graph: self.graph,
                result: None,
                warnings: self.warnings,
                error: self.error,
            },
        }
    }

    pub fn to_result(self) -> Result<Option<T>, NodeError> {
        match self.error {
            Some(error) => Err(*error),
            None => Ok(self.result),
        }
    }

    pub fn from_result(result: Result<ExecResultRepr<T>, NodeError>) -> Self {
        match result {
            Ok(result) => Self {
                graph: result.graph,
                result: result.result,
                warnings: result.warnings,
                error: result.error,
            },
            Err(error) => Self {
                graph: None,
                result: None,
                warnings: None,
                error: Some(Box::new(error)),
            },
        }
    }

    pub fn to_napi_result(self) -> napi::Result<Option<T>, NodeError> {
        self.to_result().map_err(map_node_error)
    }

    pub(crate) fn with_graph(mut self, graph: Arc<SystemWorldComputeGraph>) -> ExecResultRepr<T> {
        self.graph = Some(graph);
        self
    }
}

impl<T> ExecResultRepr<ExecResultRepr<T>> {
    pub fn flatten(self) -> ExecResultRepr<T> {
        match self.result {
            Some(result) => result,
            None => ExecResultRepr {
                result: None,
                graph: self.graph,
                warnings: self.warnings,
                error: self.error,
            },
        }
    }
}

fn print_node_diagnostic(
    graph: &Option<Arc<SystemWorldComputeGraph>>,
    warnings: &Option<Box<NodeError>>,
    error: &Option<Box<NodeError>>,
) -> Option<()> {
    let warnings = warnings.iter().flat_map(|e| e.iter_diagnostics());
    let errors = error.iter().flat_map(|e| e.iter_diagnostics());
    let world = &graph.as_ref()?.snap.world;

    print_diagnostics(world, warnings.chain(errors), DiagnosticFormat::Human);
    Some(())
}

macro_rules! impl_exec_result {
    ($name:ident, $t:ty) => {
        #[napi]
        pub struct $name(ExecResultRepr<$t>);

        impl From<Warned<SourceResult<$t>>> for $name {
            fn from(warned: Warned<SourceResult<$t>>) -> Self {
                Self(ExecResultRepr::from(warned))
            }
        }

        impl From<ExecResultRepr<$t>> for $name {
            fn from(result: ExecResultRepr<$t>) -> Self {
                Self(result)
            }
        }

        #[napi]
        impl $name {
            /// Gets the result of execution.
            #[napi(getter)]
            pub fn result(&self) -> Option<$t> {
                self.0.result.clone()
            }

            /// Takes the result of execution.
            #[napi]
            pub fn take_warnings(&mut self) -> Option<NodeError> {
                self.0.warnings.take().map(|e| *e)
            }

            /// Takes the error of execution.
            #[napi]
            pub fn take_error(&mut self) -> Option<NodeError> {
                self.0.error.take().map(|e| *e)
            }

            /// Takes the diagnostics of execution.
            #[napi]
            pub fn take_diagnostics(&mut self) -> Option<NodeError> {
                self.0.error.take().map(|e| *e)
            }

            /// Whether the execution has error.
            #[napi]
            pub fn has_error(&self) -> bool {
                self.0.error.is_some()
            }

            /// Prints the errors during execution.
            #[napi]
            pub fn print_errors(&self) {
                print_node_diagnostic(&self.0.graph, &self.0.warnings, &self.0.error);
                if let Some(error) = self.0.error.as_ref().and_then(|e| e.error_message()) {
                    eprintln!("{error}");
                }
            }

            /// Prints the diagnostics of execution.
            #[napi]
            pub fn print_diagnostics(&self) {
                print_node_diagnostic(&self.0.graph, &self.0.warnings, &self.0.error);
            }
        }
    };
}

impl_exec_result!(NodeHtmlOutputExecResult, NodeHtmlOutput);
impl_exec_result!(NodeTypstCompileResult, NodeTypstDocument);
