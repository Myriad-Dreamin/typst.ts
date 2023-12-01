use core::fmt;
use std::io::Write;
use std::sync::Arc;

use typst_ts_core::exporter_utils::map_err;
use typst_ts_core::{Transformer, TypstDocument};

#[derive(Debug, Clone, Default)]
pub struct TextExporter {}

impl<W> Transformer<(Arc<TypstDocument>, W)> for TextExporter
where
    W: std::io::Write,
{
    fn export(
        &self,
        _world: &dyn typst::World,
        (output, writer): (Arc<TypstDocument>, W),
    ) -> typst::diag::SourceResult<()> {
        let mut w = std::io::BufWriter::new(writer);

        write!(w, "{}", FullTextDigest(output)).map_err(map_err)?;

        w.flush().unwrap();
        Ok(())
    }
}

struct FullTextDigest(Arc<TypstDocument>);

impl FullTextDigest {
    fn export_frame(f: &mut fmt::Formatter<'_>, doc: &typst::layout::Frame) -> fmt::Result {
        for (_, item) in doc.items() {
            Self::export_item(f, item)?;
        }

        Ok(())
    }

    fn export_item(f: &mut fmt::Formatter<'_>, item: &typst::layout::FrameItem) -> fmt::Result {
        use std::fmt::Write;
        use typst::introspection::Meta::*;
        use typst::layout::FrameItem::*;
        match item {
            Group(g) => Self::export_frame(f, &g.frame),
            Text(t) => f.write_str(t.text.as_str()),
            Meta(ContentHint(c), _) => f.write_char(*c),
            Meta(Link(..) | Elem(..) | PageNumbering(..) | PdfPageLabel(..) | Hide, _)
            | Shape(..)
            | Image(..) => Ok(()),
        }
    }
}

impl fmt::Display for FullTextDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for frame in self.0.pages.iter() {
            Self::export_frame(f, &frame)?;
        }
        Ok(())
    }
}
