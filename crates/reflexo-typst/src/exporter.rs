use std::sync::Arc;

use reflexo::error::prelude::*;
use tinymist_world::WorldComputeGraph;

mod prelude;

#[cfg(feature = "ast")]
pub mod ast;

pub mod json;

#[cfg(feature = "dynamic-layout")]
#[cfg(feature = "svg")]
pub mod dyn_svg;
#[cfg(feature = "svg")]
pub mod svg;

#[cfg(feature = "html")]
pub mod html;

pub mod text;

pub type DynComputation<F> = Arc<dyn Fn(&Arc<WorldComputeGraph<F>>) -> Result<()> + Send + Sync>;
