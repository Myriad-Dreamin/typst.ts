//! Extension methods for `typst`'s `model` module.

use typst::{
    diag::EcoString,
    foundations::{Datetime, Smart},
    model::Document,
};

/// Extension methods for `Document`.
pub trait TypstDocumentExt {
    /// Returns the title of the document.
    fn title(&self) -> Option<&EcoString>;
    /// The document's author.
    fn author(&self) -> &Vec<EcoString>;
    /// The document's keywords.
    fn keywords(&self) -> &Vec<EcoString>;
    /// The document's creation date.
    fn date(&self) -> &Smart<Option<Datetime>>;
}

impl TypstDocumentExt for Document {
    fn title(&self) -> Option<&EcoString> {
        self.info.title.as_ref()
    }

    fn author(&self) -> &Vec<EcoString> {
        &self.info.author
    }

    fn keywords(&self) -> &Vec<EcoString> {
        &self.info.keywords
    }

    fn date(&self) -> &Smart<Option<Datetime>> {
        &self.info.date
    }
}
