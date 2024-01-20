pub(crate) mod well_known {
    pub type Bytes = typst::foundations::Bytes;

    /// Although this is not good to expose this, we make an alias here to let
    /// it as a part of typst-ts.
    pub use typst::syntax::FileId as TypstFileId;

    pub use typst::World as TypstWorld;

    pub use typst::layout::Abs as TypstAbs;

    pub use typst::model::Document as TypstDocument;

    pub use typst::text::Font as TypstFont;

    pub use typst::foundations::Dict as TypstDict;

    pub use typst::foundations::Datetime as TypstDatetime;

    pub use typst::{diag, foundations};
}

pub use well_known::*;

pub mod prelude {
    pub use comemo::Prehashed;
    pub use ecow::{eco_format, eco_vec, EcoString, EcoVec};
}
