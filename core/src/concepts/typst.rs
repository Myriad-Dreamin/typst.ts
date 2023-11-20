pub(crate) mod well_known {
    pub type Bytes = typst::eval::Bytes;

    /// Although this is not good to expose this, we make an alias here to let it as
    /// a part of typst-ts.
    pub use typst::syntax::FileId as TypstFileId;

    pub use typst::geom::Abs as TypstAbs;

    pub use typst::doc::Document as TypstDocument;
}

pub use well_known::*;

pub mod prelude {
    pub use ecow::{eco_format, eco_vec, EcoString, EcoVec};
}
