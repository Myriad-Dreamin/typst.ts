pub use reflexo::hash::*;

/// This function maintain hash function corresponding to Typst
/// Typst changed the hash function from `siphasher::sip128::SipHasher` to
/// `siphasher::sip128::SipHasher13` since commit <https://github.com/typst/typst/commit/d0afba959d18d1c2c646b99e6ddd864b1a91deb2>
/// Commit log:
/// This seems to significantly improves performance. Inspired by
/// rust-lang/rust#107925
///
/// Update: Use Typst's new util function `reflexo::typst_shim::utils::hash128`
#[inline]
pub fn typst_affinite_hash<T: std::hash::Hash>(t: &T) -> u128 {
    reflexo::typst_shim::utils::hash128(t)
}
