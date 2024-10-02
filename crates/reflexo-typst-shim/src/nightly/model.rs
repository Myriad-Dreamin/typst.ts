use typst::{diag::EcoString, model::Document};

/// Extension methods for `Document`.
pub trait TypstDocumentExt {
    /// Returns the title of the document.
    fn title(&self) -> Option<&EcoString>;
}

impl TypstDocumentExt for Document {
    fn title(&self) -> Option<&EcoString> {
        self.info.title.as_ref()
    }
}
