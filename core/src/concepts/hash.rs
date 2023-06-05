use std::{
    any::Any,
    hash::{Hash, Hasher},
    ops::Deref,
};

use siphasher::sip128::{Hasher128, SipHasher13};

pub trait StaticHash128 {
    fn get_hash(&self) -> u128;
}

impl Hash for dyn StaticHash128 {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u128(self.get_hash());
    }
}

pub fn make_hash<T: Hash + 'static>(item: &T) -> u128 {
    // Also hash the TypeId because the type might be converted
    // through an unsized coercion.
    let mut state = SipHasher13::new();
    item.type_id().hash(&mut state);
    item.hash(&mut state);
    state.finish128().as_u128()
}

pub struct HashedTrait<T: ?Sized> {
    hash: u128,
    t: Box<T>,
}

impl<T: ?Sized> HashedTrait<T> {
    pub fn new(hash: u128, t: Box<T>) -> Self {
        Self { hash, t }
    }
}

impl<T: ?Sized> Deref for HashedTrait<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.t
    }
}

impl<T> Hash for HashedTrait<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u128(self.hash);
    }
}

impl<T: Hash + Default + 'static> Default for HashedTrait<T> {
    fn default() -> Self {
        let t = T::default();
        Self {
            hash: make_hash(&t),
            t: Box::new(t),
        }
    }
}

impl<T: ?Sized> StaticHash128 for HashedTrait<T> {
    fn get_hash(&self) -> u128 {
        self.hash
    }
}
