pub(crate) mod well_known {
    pub type Bytes = typst::foundations::Bytes;

    /// Although this is not good to expose this, we make an alias here to let it as
    /// a part of typst-ts.
    pub use typst::syntax::FileId as TypstFileId;

    pub type TypstAbs = typst::layout::Abs;

    pub use typst::model::Document as TypstDocument;
}

pub use well_known::*;

pub mod prelude {
    pub use ecow::{eco_format, eco_vec, EcoString, EcoVec};
}
