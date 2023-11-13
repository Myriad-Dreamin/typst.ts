mod takable;
use std::{path::Path, sync::Arc};

pub use takable::*;

mod hash;
pub use hash::*;

mod query;
pub use query::*;

mod read;
pub use read::*;

mod marker;
pub use marker::*;

/// Re-export of the typst crate.
pub mod typst;
pub use typst::well_known::*;

pub type ImmutStr = Arc<str>;
pub type ImmutPath = Arc<Path>;
