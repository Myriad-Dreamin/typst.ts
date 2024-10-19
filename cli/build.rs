use anyhow::Result;
use vergen::{BuildBuilder, CargoBuilder, Emitter, RustcBuilder};
use vergen_gitcl::GitclBuilder;

fn main() -> Result<()> {
    let build = BuildBuilder::default().build_timestamp(true).build()?;
    let cargo = CargoBuilder::all_cargo()?;
    let rustc = RustcBuilder::default()
        .commit_hash(true)
        .semver(true)
        .host_triple(true)
        .channel(true)
        .llvm_version(true)
        .build()?;
    let gitcl = GitclBuilder::default()
        .sha(false)
        .describe(true, true, None)
        .build()?;

    // Emit the instructions
    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&rustc)?
        .add_instructions(&gitcl)?
        .emit()?;
    Ok(())
}
