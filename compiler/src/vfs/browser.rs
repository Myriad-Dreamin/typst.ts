use std::path::{Path, PathBuf};

use typst::diag::{FileError, FileResult};
use wasm_bindgen::prelude::*;

use typst_ts_core::Bytes;

use crate::Time;

use super::AccessModel;

/// Provides proxy access model from typst compiler to some JavaScript
/// implementation.
#[derive(Debug)]
pub struct ProxyAccessModel {
    /// The `this` value when calling the JavaScript functions
    pub context: JsValue,
    /// The JavaScript function to get the mtime of a file
    pub mtime_fn: js_sys::Function,
    /// The JavaScript function to check if a path corresponds to a file or a
    /// directory
    pub is_file_fn: js_sys::Function,
    /// The JavaScript function to get the real path of a file
    pub real_path_fn: js_sys::Function,
    /// The JavaScript function to get the content of a file
    pub read_all_fn: js_sys::Function,
}

impl AccessModel for ProxyAccessModel {
    type RealPath = PathBuf;

    fn mtime(&self, src: &Path) -> FileResult<Time> {
        self.mtime_fn
            .call1(&self.context, &src.to_string_lossy().as_ref().into())
            .map(|v| {
                let v = v.as_f64().unwrap();
                Time::UNIX_EPOCH + std::time::Duration::from_secs_f64(v)
            })
            .map_err(|e| {
                web_sys::console::error_3(
                    &"typst_ts::compiler::ProxyAccessModel::mtime failure".into(),
                    &src.to_string_lossy().as_ref().into(),
                    &e,
                );
                FileError::AccessDenied
            })
    }

    fn is_file(&self, src: &Path) -> FileResult<bool> {
        self.is_file_fn
            .call1(&self.context, &src.to_string_lossy().as_ref().into())
            .map(|v| v.as_bool().unwrap())
            .map_err(|e| {
                web_sys::console::error_3(
                    &"typst_ts::compiler::ProxyAccessModel::is_file failure".into(),
                    &src.to_string_lossy().as_ref().into(),
                    &e,
                );
                FileError::AccessDenied
            })
    }

    fn real_path(&self, src: &Path) -> FileResult<Self::RealPath> {
        self.real_path_fn
            .call1(&self.context, &src.to_string_lossy().as_ref().into())
            .map(|v| Path::new(&v.as_string().unwrap()).to_owned())
            .map_err(|e| {
                web_sys::console::error_3(
                    &"typst_ts::compiler::ProxyAccessModel::real_path failure".into(),
                    &src.to_string_lossy().as_ref().into(),
                    &e,
                );
                FileError::AccessDenied
            })
    }

    fn content(&self, src: &Path) -> FileResult<Bytes> {
        let data = self
            .read_all_fn
            .call1(&self.context, &src.to_string_lossy().as_ref().into())
            .map_err(|e| {
                web_sys::console::error_3(
                    &"typst_ts::compiler::ProxyAccessModel::read_all failure".into(),
                    &src.to_string_lossy().as_ref().into(),
                    &e,
                );
                FileError::AccessDenied
            })?;

        let data = if let Some(data) = data.dyn_ref::<js_sys::Uint8Array>() {
            Bytes::from(data.to_vec())
        } else {
            return Err(FileError::AccessDenied);
        };

        Ok(data)
    }
}
