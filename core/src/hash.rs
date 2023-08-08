use std::{any::Any, collections::HashMap, hash::Hash};

use base64::Engine;
use siphasher::sip128::{Hasher128, SipHasher13};

#[cfg(feature = "rkyv")]
use rkyv::{Archive, Deserialize as rDeser, Serialize as rSer};

/// See <https://github.com/rust-lang/rust/blob/master/compiler/rustc_hir/src/stable_hash_impls.rs#L22>
/// The fingerprint conflicts should be very rare and should be handled by the
/// compiler.
///
/// > That being said, given a high quality hash function, the collision
/// > probabilities in question are very small. For example, for a big crate
/// > like `rustc_middle` (with ~50000 `LocalDefId`s as of the time of writing)
/// > there is a probability of roughly 1 in 14,750,000,000 of a crate-internal
/// > collision occurring. For a big crate graph with 1000 crates in it, there
/// > is a probability of 1 in 36,890,000,000,000 of a `StableCrateId`
/// > collision.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct Fingerprint(u64, u64);

impl Fingerprint {
    pub fn from_pair(lo: u64, hi: u64) -> Self {
        Self(lo, hi)
    }

    pub fn from_u128(hash: u128) -> Self {
        Self(hash as u64, (hash >> 64) as u64)
    }

    pub fn to_u128(self) -> u128 {
        ((self.1 as u128) << 64) | self.0 as u128
    }

    /// Create a xml id from the given prefix and the fingerprint of this
    /// reference. Note that the entire html document shares namespace for
    /// ids.
    #[comemo::memoize]
    pub fn as_svg_id(self, prefix: &'static str) -> String {
        let fingerprint_hi =
            base64::engine::general_purpose::STANDARD_NO_PAD.encode(self.0.to_le_bytes());
        if self.1 == 0 {
            return [prefix, &fingerprint_hi].join("");
        }

        // possible the id in the lower 64 bits.
        let fingerprint_lo = {
            let id = self.1.to_le_bytes();
            // truncate zero
            let rev_zero = id.iter().rev().skip_while(|&&b| b == 0).count();
            let id = &id[..rev_zero];
            base64::engine::general_purpose::STANDARD_NO_PAD.encode(id)
        };
        [prefix, &fingerprint_hi, &fingerprint_lo].join("")
    }
}

/// A fingerprint hasher that extends the [`std::hash::Hasher`] trait.
pub trait FingerprintHasher: std::hash::Hasher {
    /// Finish the fingerprint and return the fingerprint and the data.
    /// The data is used to resolve the conflict.
    fn finish_fingerprint(&self) -> (Fingerprint, Vec<u8>);
}

/// A fingerprint hasher that uses the [`SipHasher13`] algorithm.
struct FingerprintSipHasher {
    /// The underlying data passed to the hasher.
    data: Vec<u8>,
}

impl std::hash::Hasher for FingerprintSipHasher {
    fn write(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    fn finish(&self) -> u64 {
        let mut inner = SipHasher13::new();
        self.data.hash(&mut inner);
        inner.finish()
    }
}

impl FingerprintHasher for FingerprintSipHasher {
    fn finish_fingerprint(&self) -> (Fingerprint, Vec<u8>) {
        let buffer = self.data.clone();
        let mut inner = SipHasher13::new();
        buffer.hash(&mut inner);
        let hash = inner.finish128();
        (Fingerprint(hash.h1, hash.h2), buffer)
    }
}

/// A fingerprint builder that produces unique fingerprint for each item.
/// It resolves the conflict by checking the underlying data.
/// See [`Fingerprint`] for more information.
#[derive(Default)]
pub struct FingerprintBuilder {
    /// The conflict checker mapping fingerprints to their underlying data.
    conflict_checker: HashMap<Fingerprint, Vec<u8>>,
}

impl FingerprintBuilder {
    pub fn resolve<T: Hash + 'static>(&mut self, item: &T) -> Fingerprint {
        let mut s = FingerprintSipHasher { data: Vec::new() };
        item.type_id().hash(&mut s);
        item.hash(&mut s);
        let (fingerprint, featured_data) = s.finish_fingerprint();
        if let Some(prev_featured_data) = self.conflict_checker.get(&fingerprint) {
            if prev_featured_data != &featured_data {
                // todo: soft error
                panic!("Fingerprint conflict detected!");
            }

            return fingerprint;
        }

        self.conflict_checker.insert(fingerprint, featured_data);
        fingerprint
    }
}

/// This function maintain hash function corresponding to Typst
/// Typst changed the hash function from [`siphasher::sip128::SipHasher`] to
///   [`SipHasher13`] since commit
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

/// This function provides a hash function for items, which also includes a type
/// id as part of the hash. Note: This function is not stable across different
/// versions of typst-ts, so it is preferred to be always used in memory.
/// Currently, this function use [`SipHasher13`] as the underlying hash
/// algorithm.
pub fn item_hash128<T: Hash + 'static>(item: &T) -> u128 {
    // Also hash the TypeId because the type might be converted
    // through an unsized coercion.
    let mut state = SipHasher13::new();
    item.type_id().hash(&mut state);
    item.hash(&mut state);
    state.finish128().as_u128()
}

/// Calculate a 128-bit siphash of a value.
/// Currently, this function use [`SipHasher13`] as the underlying hash
/// algorithm.
#[inline]
pub fn hash128<T: std::hash::Hash>(t: &T) -> u128 {
    typst::util::hash128(t)
}
