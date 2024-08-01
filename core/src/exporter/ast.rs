use std::{io::Write, sync::Arc};

use typst::{
    diag::{At, FileError},
    syntax::Span,
};
use typst_ts_ast_exporter::dump_ast;

use crate::Transformer;

#[derive(Debug, Clone, Default)]
pub struct AstExporter {}

impl<W> Transformer<(Arc<typst::model::Document>, W)> for AstExporter
where
    W: std::io::Write,
{
    fn export(
        &self,
        world: &dyn typst::World,
        (_output, writer): (Arc<typst::model::Document>, W),
    ) -> typst::diag::SourceResult<()> {
        let mut writer = std::io::BufWriter::new(writer);

        let src = world.main();
        let path = src.id().vpath().as_rootless_path();
        dump_ast(&path.display().to_string(), &src, &mut writer)
            .map_err(|e| FileError::from_io(e, path))
            .at(Span::detached())?;

        writer.flush().unwrap();

        Ok(())
    }
}
