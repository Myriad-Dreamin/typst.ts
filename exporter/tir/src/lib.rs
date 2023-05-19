use byteorder::{LittleEndian, WriteBytesExt};
use std::{io::Write, sync::Arc};

use typst::diag::SourceResult;
use typst_ts_core::artifact_ir::{Artifact, ArtifactHeader};
use typst_ts_core::Exporter;

/// IR structure (in bytes)
/// =======================
/// [0 - 4] Magic number 'IRAR' (u32)
///
/// [4 - 8] Version number (u32)
///
/// [8 - 16] Length of metadata (u64)
///
/// [16 - 16 + metadata_len] Metadata (JSON)
///
/// [16 + metadata_len - end] global buffer (binary)

/// IR artifact exporter
const MAGIC_NUMBER: [u8; 4] = [b'I', b'R', b'A', b'R'];

#[derive(Debug, Clone, Default)]
pub struct IRArtifactExporter;

impl Exporter<Artifact, Vec<u8>> for IRArtifactExporter {
    /// Export the given IR artifact with given world.
    fn export<'a>(
        &'a self,
        _world: &'a dyn typst::World,
        output: Arc<Artifact>,
    ) -> SourceResult<Vec<u8>> {
        let metadata = serde_json::to_string(&ArtifactHeader {
            metadata: output.metadata.clone(),
            pages: output.pages.clone(),
        })
        .unwrap();
        let cap = metadata.len() + output.buffer.len() + 16;
        let mut writer = std::io::Cursor::new(Vec::with_capacity(cap));
        writer.write_all(&MAGIC_NUMBER).unwrap();

        writer.write_u32::<LittleEndian>(1).unwrap();
        writer
            .write_u64::<LittleEndian>(metadata.len() as u64)
            .unwrap();
        writer.write_all(metadata.as_bytes()).unwrap();
        writer.write_all(output.buffer.as_slice()).unwrap();

        Ok(writer.into_inner())
    }
}
