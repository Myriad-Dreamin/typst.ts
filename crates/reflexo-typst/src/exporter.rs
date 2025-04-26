use std::sync::Arc;

use reflexo::error::prelude::*;
use tinymist_world::WorldComputeGraph;

#[cfg(feature = "ast")]
pub mod ast;

#[cfg(feature = "dynamic-layout")]
#[cfg(feature = "svg")]
pub mod dyn_svg;
#[cfg(feature = "html")]
pub mod html;
#[cfg(feature = "svg")]
pub mod svg;

pub mod text;

pub type DynComputation<F> = Arc<dyn Fn(&Arc<WorldComputeGraph<F>>) -> Result<()> + Send + Sync>;
