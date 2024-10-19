use anyhow::Result;
use vergen::{CargoBuilder, Emitter, RustcBuilder};
use vergen_gitcl::GitclBuilder;

fn main() -> Result<()> {
    let cargo = CargoBuilder::default()
        .opt_level(true)
        .features(true)
        .build()?;
    let rustc = RustcBuilder::default()
        .commit_hash(true)
        .semver(true)
        .host_triple(true)
        .channel(true)
        .llvm_version(true)
        .build()?;
    let gitcl = GitclBuilder::default().sha(false).build()?;

    // Emit the instructions
    Emitter::default()
        .add_instructions(&cargo)?
        .add_instructions(&rustc)?
        .add_instructions(&gitcl)?
        .emit()?;
    Ok(())
}
