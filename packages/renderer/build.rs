use anyhow::Result;
use vergen::EmitBuilder;

fn main() -> Result<()> {
    // Emit the instructions
    EmitBuilder::builder()
        .cargo_features()
        .cargo_opt_level()
        .build_timestamp()
        .git_sha(false)
        .emit()?;
    Ok(())
}
