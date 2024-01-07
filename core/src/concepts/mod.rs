mod takable;
use std::{path::Path, sync::Arc};

pub use takable::*;

mod deferred;
pub use deferred::*;

mod hash;
pub use hash::*;

pub mod cow_mut;

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
pub type ImmutBytes = Arc<[u8]>;
pub type ImmutPath = Arc<Path>;
