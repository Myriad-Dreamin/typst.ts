// Core type system/concepts of typst-ts.
// #![warn(missing_docs)]
// #![warn(missing_debug_implementations)]
// #![warn(missing_copy_implementations)]

mod concepts;
pub use concepts::*;

// Core data structures of typst-ts.
// todo: move me to compiler
pub mod cache;
pub mod config;
pub mod debug_loc;
pub mod error;
pub mod font;
pub mod package;

// Core mechanism of typst-ts.
pub(crate) mod exporter;

// Intermediate representation of typst-ts.
pub mod vector;

pub mod hash {
    pub use reflexo::hash::*;

    /// This function maintain hash function corresponding to Typst
    /// Typst changed the hash function from [`siphasher::sip128::SipHasher`] to
    ///   [`siphasher::sip128::SipHasher13`] since commit
    ///   <https://github.com/typst/typst/commit/d0afba959d18d1c2c646b99e6ddd864b1a91deb2>
    /// Commit log:
    /// This seems to significantly improves performance. Inspired by
    /// rust-lang/rust#107925
    ///
    /// Update: Use Typst's new util function `typst::util::hash128`
    #[inline]
    pub fn typst_affinite_hash<T: std::hash::Hash>(t: &T) -> u128 {
        typst::util::hash128(t)
    }
}

pub use exporter::{builtins as exporter_builtins, utils as exporter_utils};
pub use exporter::{
    DynExporter, DynGenericExporter, DynPolymorphicExporter, Exporter, GenericExporter,
    GenericTransformer, Transformer,
};
pub use font::{FontLoader, FontResolver, FontSlot};
pub use reflexo::*;

pub mod build_info {
    /// The version of the typst-ts-core crate.
    pub static VERSION: &str = env!("CARGO_PKG_VERSION");
}

pub mod program_meta {
    /// inform the user that this is a bug.
    pub const REPORT_BUG_MESSAGE: &str =
        "This is a bug, please report to https://github.com/Myriad-Dreamin/typst.ts/issues/new";
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_hash128() {
        assert_eq!(typst::util::hash128(&0u32), reflexo::hash::hash128(&0u32));
    }
}
