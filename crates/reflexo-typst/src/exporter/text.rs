use core::fmt;
use std::io::Write;
use std::sync::Arc;

use crate::exporter_utils::map_err;
use crate::{Transformer, TypstDocument};

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
        #[cfg(not(feature = "no-content-hint"))]
        use std::fmt::Write;
        use typst::layout::FrameItem::*;
        match item {
            Group(g) => Self::export_frame(f, &g.frame),
            Text(t) => f.write_str(t.text.as_str()),
            #[cfg(not(feature = "no-content-hint"))]
            ContentHint(c) => f.write_char(*c),
            Link(..) | Tag(..) | Shape(..) | Image(..) => Ok(()),
        }
    }
}

impl fmt::Display for FullTextDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for page in self.0.pages.iter() {
            Self::export_frame(f, &page.frame)?;
        }
        Ok(())
    }
}
