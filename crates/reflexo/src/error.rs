use core::fmt;
use std::{
    io,
    path::{Path, PathBuf},
    str::Utf8Error,
    string::FromUtf8Error,
};

use ecow::{eco_format, EcoString};
use serde::{Deserialize, Serialize};

use crate::debug_loc::CharRange;

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone)]
#[repr(u8)]
pub enum DiagSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

impl ToString for DiagSeverity {
    fn to_string(&self) -> String {
        match self {
            DiagSeverity::Error => "error".to_string(),
            DiagSeverity::Warning => "warning".to_string(),
            DiagSeverity::Information => "information".to_string(),
            DiagSeverity::Hint => "hint".to_string(),
        }
    }
}

/// <https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#diagnostic>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagMessage {
    pub package: String,
    pub path: String,
    pub message: String,
    pub severity: DiagSeverity,
    pub range: Option<CharRange>,
    // These field could be added to ErrorImpl::arguments
    // owner: Option<ImmutStr>,
    // source: ImmutStr,
}

impl DiagMessage {}

/// A result type with a file-related error.
pub type FileResult<T> = Result<T, FileError>;

/// An error that occurred while trying to load of a file.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum FileError {
    /// A file was not found at this path.
    NotFound(PathBuf),
    /// A file could not be accessed.
    AccessDenied,
    /// A directory was found, but a file was expected.
    IsDirectory,
    /// The file is not a Typst source file, but should have been.
    NotSource,
    /// The file was not valid UTF-8, but should have been.
    InvalidUtf8,
    /// Another error.
    ///
    /// The optional string can give more details, if available.
    Other(Option<EcoString>),
}

impl FileError {
    /// Create a file error from an I/O error.
    pub fn from_io(err: io::Error, path: &Path) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => Self::NotFound(path.into()),
            io::ErrorKind::PermissionDenied => Self::AccessDenied,
            io::ErrorKind::InvalidData
                if err
                    .to_string()
                    .contains("stream did not contain valid UTF-8") =>
            {
                Self::InvalidUtf8
            }
            _ => Self::Other(Some(eco_format!("{err}"))),
        }
    }
}

impl std::error::Error for FileError {}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotFound(path) => {
                write!(f, "file not found (searched at {})", path.display())
            }
            Self::AccessDenied => f.pad("failed to load file (access denied)"),
            Self::IsDirectory => f.pad("failed to load file (is a directory)"),
            Self::NotSource => f.pad("not a typst source file"),
            Self::InvalidUtf8 => f.pad("file is not valid utf-8"),
            Self::Other(Some(err)) => write!(f, "failed to load file ({err})"),
            Self::Other(None) => f.pad("failed to load file"),
        }
    }
}

impl From<Utf8Error> for FileError {
    fn from(_: Utf8Error) -> Self {
        Self::InvalidUtf8
    }
}

impl From<FromUtf8Error> for FileError {
    fn from(_: FromUtf8Error) -> Self {
        Self::InvalidUtf8
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ErrKind {
    None,
    Msg(String),
    Diag(DiagMessage),
    File(FileError),
    Inner(Error),
}

pub trait ErrKindExt {
    fn to_error_kind(self) -> ErrKind;
}

impl ErrKindExt for ErrKind {
    fn to_error_kind(self) -> Self {
        self
    }
}

impl ErrKindExt for FileError {
    fn to_error_kind(self) -> ErrKind {
        ErrKind::File(self)
    }
}

impl ErrKindExt for std::io::Error {
    fn to_error_kind(self) -> ErrKind {
        ErrKind::File(FileError::from_io(self, std::path::Path::new("")))
    }
}

impl ErrKindExt for String {
    fn to_error_kind(self) -> ErrKind {
        ErrKind::Msg(self)
    }
}

impl ErrKindExt for &str {
    fn to_error_kind(self) -> ErrKind {
        ErrKind::Msg(self.to_string())
    }
}

impl ErrKindExt for &String {
    fn to_error_kind(self) -> ErrKind {
        ErrKind::Msg(self.to_string())
    }
}

impl ErrKindExt for EcoString {
    fn to_error_kind(self) -> ErrKind {
        ErrKind::Msg(self.to_string())
    }
}

impl ErrKindExt for &dyn std::fmt::Display {
    fn to_error_kind(self) -> ErrKind {
        ErrKind::Msg(self.to_string())
    }
}

impl ErrKindExt for serde_json::Error {
    fn to_error_kind(self) -> ErrKind {
        ErrKind::Msg(self.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct ErrorImpl {
    loc: &'static str,
    kind: ErrKind,
    arguments: Box<[(&'static str, String)]>,
}

/// This type represents all possible errors that can occur in typst.ts
#[derive(Debug, Clone)]
pub struct Error {
    /// This `Box` allows us to keep the size of `Error` as small as possible. A
    /// larger `Error` type was substantially slower due to all the functions
    /// that pass around `Result<T, Error>`.
    err: Box<ErrorImpl>,
}

impl Error {
    pub fn new(loc: &'static str, kind: ErrKind, arguments: Box<[(&'static str, String)]>) -> Self {
        Self {
            err: Box::new(ErrorImpl {
                loc,
                kind,
                arguments,
            }),
        }
    }

    pub fn loc(&self) -> &'static str {
        self.err.loc
    }

    pub fn kind(&self) -> &ErrKind {
        &self.err.kind
    }

    pub fn arguments(&self) -> &[(&'static str, String)] {
        &self.err.arguments
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err = &self.err;
        match &err.kind {
            ErrKind::File(e) => write!(f, "{}: {} with {:?}", err.loc, e, err.arguments),
            ErrKind::Msg(msg) => write!(f, "{}: {} with {:?}", err.loc, msg, err.arguments),
            ErrKind::Diag(diag) => {
                write!(f, "{}: {} with {:?}", err.loc, diag.message, err.arguments)
            }
            ErrKind::Inner(e) => write!(f, "{}: {} with {:?}", err.loc, e, err.arguments),
            ErrKind::None => write!(f, "{}: with {:?}", err.loc, err.arguments),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(feature = "web")]
impl ErrKindExt for wasm_bindgen::JsValue {
    fn to_error_kind(self) -> ErrKind {
        ErrKind::Msg(format!("{:?}", self))
    }
}

#[cfg(feature = "web")]
impl From<Error> for wasm_bindgen::JsValue {
    fn from(e: Error) -> Self {
        js_sys::Error::new(&e.to_string()).into()
    }
}

#[cfg(feature = "web")]
impl From<&Error> for wasm_bindgen::JsValue {
    fn from(e: &Error) -> Self {
        js_sys::Error::new(&e.to_string()).into()
    }
}

pub mod prelude {

    use super::ErrKindExt;
    use crate::Error;

    pub type ZResult<T> = Result<T, Error>;

    pub trait WithContext<T>: Sized {
        fn context(self, loc: &'static str) -> ZResult<T>;

        fn with_context<F>(self, loc: &'static str, f: F) -> ZResult<T>
        where
            F: FnOnce() -> Box<[(&'static str, String)]>;
    }

    impl<T, E: ErrKindExt> WithContext<T> for Result<T, E> {
        fn context(self, loc: &'static str) -> ZResult<T> {
            self.map_err(|e| Error::new(loc, e.to_error_kind(), Box::new([])))
        }

        fn with_context<F>(self, loc: &'static str, f: F) -> ZResult<T>
        where
            F: FnOnce() -> Box<[(&'static str, String)]>,
        {
            self.map_err(|e| Error::new(loc, e.to_error_kind(), f()))
        }
    }

    pub fn map_string_err<T: ToString>(loc: &'static str) -> impl Fn(T) -> Error {
        move |e| Error::new(loc, e.to_string().to_error_kind(), Box::new([]))
    }

    pub fn map_into_err<S: ErrKindExt, T: Into<S>>(loc: &'static str) -> impl Fn(T) -> Error {
        move |e| Error::new(loc, e.into().to_error_kind(), Box::new([]))
    }

    pub fn map_err<T: ErrKindExt>(loc: &'static str) -> impl Fn(T) -> Error {
        move |e| Error::new(loc, e.to_error_kind(), Box::new([]))
    }

    pub fn wrap_err(loc: &'static str) -> impl Fn(Error) -> Error {
        move |e| Error::new(loc, crate::ErrKind::Inner(e), Box::new([]))
    }

    pub fn map_string_err_with_args<
        T: ToString,
        Args: IntoIterator<Item = (&'static str, String)>,
    >(
        loc: &'static str,
        arguments: Args,
    ) -> impl FnOnce(T) -> Error {
        move |e| {
            Error::new(
                loc,
                e.to_string().to_error_kind(),
                arguments.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            )
        }
    }

    pub fn map_into_err_with_args<
        S: ErrKindExt,
        T: Into<S>,
        Args: IntoIterator<Item = (&'static str, String)>,
    >(
        loc: &'static str,
        arguments: Args,
    ) -> impl FnOnce(T) -> Error {
        move |e| {
            Error::new(
                loc,
                e.into().to_error_kind(),
                arguments.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            )
        }
    }

    pub fn map_err_with_args<T: ErrKindExt, Args: IntoIterator<Item = (&'static str, String)>>(
        loc: &'static str,
        arguments: Args,
    ) -> impl FnOnce(T) -> Error {
        move |e| {
            Error::new(
                loc,
                e.to_error_kind(),
                arguments.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            )
        }
    }

    pub fn wrap_err_with_args<Args: IntoIterator<Item = (&'static str, String)>>(
        loc: &'static str,
        arguments: Args,
    ) -> impl FnOnce(Error) -> Error {
        move |e| {
            Error::new(
                loc,
                crate::ErrKind::Inner(e),
                arguments.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            )
        }
    }

    pub fn _error_once(loc: &'static str, args: Box<[(&'static str, String)]>) -> Error {
        Error::new(loc, crate::ErrKind::None, args)
    }

    #[macro_export]
    macro_rules! error_once {
        ($loc:expr, $($arg_key:ident: $arg:expr),+ $(,)?) => {
            _error_once($loc, Box::new([$((stringify!($arg_key), $arg.to_string())),+]))
        };
        ($loc:expr $(,)?) => {
            _error_once($loc, Box::new([]))
        };
    }

    #[macro_export]
    macro_rules! error_once_map {
        ($loc:expr, $($arg_key:ident: $arg:expr),+ $(,)?) => {
            map_err_with_args($loc, [$((stringify!($arg_key), $arg.to_string())),+])
        };
        ($loc:expr $(,)?) => {
            map_err($loc)
        };
    }

    #[macro_export]
    macro_rules! error_once_map_string {
        ($loc:expr, $($arg_key:ident: $arg:expr),+ $(,)?) => {
            map_string_err_with_args($loc, [$((stringify!($arg_key), $arg.to_string())),+])
        };
        ($loc:expr $(,)?) => {
            map_string_err($loc)
        };
    }

    pub use error_once;
    pub use error_once_map;
    pub use error_once_map_string;
}

#[test]
fn test_send() {
    fn is_send<T: Send>() {}
    is_send::<Error>();
}
