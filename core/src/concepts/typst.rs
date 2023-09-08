pub type Bytes = typst::eval::Bytes;

/// Although this is not good to expose this, we make an alias here to let it as
/// a part of typst-ts.
pub use typst::syntax::FileId as TypstFileId;

pub use typst::geom::Abs as TypstAbs;

pub use typst::doc::Document as TypstDocument;
