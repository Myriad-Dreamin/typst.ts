use std::{io::Write, sync::Arc};
use byteorder::{LittleEndian, WriteBytesExt};

use typst::diag::SourceResult;
use typst_ts_core::artifact_ir::Artifact;

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
const MAGIC_NUMBER: [u8; 4] = ['I' as u8, 'R' as u8, 'A' as u8, 'R' as u8];

crate::serde_exporter!(IRArtifactExporter);

impl IRArtifactExporter {
    /// Export the given IR artifact with given world.
    pub fn export(&self, world: &dyn typst::World, output: Arc<Artifact>) -> SourceResult<()> {
        let metadata = serde_json::to_string(&output.metadata).unwrap();
        let cap = metadata.len() + output.buffer.len() + 16;
        let mut writer = std::io::Cursor::new(Vec::with_capacity(cap));
        writer.write(&MAGIC_NUMBER).unwrap();

        writer.write_u32::<LittleEndian>(1).unwrap();
        writer.write_u64::<LittleEndian>(metadata.len() as u64).unwrap();
        writer.write_all(metadata.as_bytes()).unwrap();
        writer.write_all(output.buffer.as_slice()).unwrap();

        crate::write_to_path(world, self.path.clone(), writer.get_ref())
    }
}
