use tiny_skia as sk;
mod path2d;
mod utils;

pub mod geom;

pub mod ir;
pub mod vm;

mod lowering;
pub use lowering::{GlyphLowerBuilder, LowerBuilder};

pub mod flat_ir;
pub mod flat_vm;

#[cfg(feature = "rkyv")]
pub mod stream;

#[cfg(feature = "vector-bbox")]
pub mod bbox;
