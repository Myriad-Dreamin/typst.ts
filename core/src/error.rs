use core::fmt;

use ecow::EcoString;
use typst::{diag::FileError, syntax::Source};

use crate::ImmutStr;

pub use typst::diag::SourceDiagnostic as TypstSourceDiagnostic;

pub use typst::diag::FileError as TypstFileError;

#[derive(Debug, Clone)]
pub struct DiagMessage {
    pub resource: ImmutStr,
    pub message: String,
    pub code: u32,
    pub severity: u32,
    pub start_line_number: u32,
    pub start_column: u32,
    pub end_line_number: u32,
    pub end_column: u32,
    // These field could be added to ErrorImpl::arguments
    // owner: Option<ImmutStr>,
    // source: ImmutStr,
}

pub trait ErrorConverter {
    // todo: file_id to path
    /// Convert typst.ts diagnostic to error
    /// It has a simple implementation.
    /// If you want to customize it, you can implement it yourself.
    fn convert_typst(&self, world: &dyn typst::World, diag: TypstSourceDiagnostic) -> Error {
        let mut arguments = Vec::new();
        arguments.push((
            "severity",
            match diag.severity {
                typst::diag::Severity::Error => "error".to_string(),
                typst::diag::Severity::Warning => "warning".to_string(),
            },
        ));

        if let Some(id) = diag.span.id() {
            if let Some(pkg) = id.package() {
                arguments.push(("package", pkg.to_string()));
            };
            arguments.push(("path", format!("{:?}", id.vpath())));

            if let Some((rng, src)) = world
                .source(id)
                .ok()
                .and_then(|src| Some((src.find(diag.span)?.range(), src)))
            {
                let resolve_off =
                    |src: &Source, off: usize| src.byte_to_line(off).zip(src.byte_to_column(off));
                if let Some((l, c)) = resolve_off(&src, rng.start) {
                    arguments.push(("start_line", l.to_string()));
                    arguments.push(("start_column", c.to_string()));
                }
                if let Some((l, c)) = resolve_off(&src, rng.start) {
                    arguments.push(("end_line", l.to_string()));
                    arguments.push(("end_column", c.to_string()));
                }
            }
        }

        Error::new(
            "typst",
            ErrKind::Msg(format!(
                "{msg}, hints: {hints:?}, traces: {traces:?}",
                msg = diag.message,
                hints = diag.hints,
                traces = diag.trace
            )),
            arguments.into_boxed_slice(),
        )
    }
}

impl DiagMessage {}

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
