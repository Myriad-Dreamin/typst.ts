use std::cell::{RefCell, RefMut};

/// Represent the result of an immutable query reference.
/// The compute function should be pure enough.
///
/// [`compute`]: Self::compute
/// [`compute_ref`]: Self::compute_ref
pub struct QueryRef<T, E> {
    /// `None` means no value has been computed yet.
    result: RefCell<Option<Result<T, E>>>,
}

impl<T, E: Clone> QueryRef<T, E> {
    /// Compute and return a checked reference guard.
    pub fn compute<F: FnOnce() -> Result<T, E>>(&self, f: F) -> Result<QueryResult<'_, T>, E> {
        RefMut::filter_map(
            self.result.borrow_mut(),
            |r: &mut Option<Result<T, E>>| -> Option<&mut T> {
                r.get_or_insert_with(f).as_mut().ok()
            },
        )
        .map_err(|r| {
            r.as_ref()
                .unwrap()
                .as_ref()
                .map(|_| ())
                .unwrap_err()
                .clone()
        })
        .map(QueryResult)
    }

    /// Compute and return a unchecked reference guard.
    pub fn compute_ref<F: FnOnce() -> Result<T, E>>(&self, f: F) -> Result<&T, E> {
        self.compute(f)?;

        // the value is already initialized by f
        let holding = unsafe { (*self.result.as_ptr()).as_ref().unwrap_unchecked() };
        holding.as_ref().map_err(Clone::clone)
    }
}

/// std::ops::DerefMut is disabled, since we can call compute_ref safely.
/// It means that multiple immutable references can be long lived.
pub struct QueryResult<'a, T>(RefMut<'a, T>);

impl<'a, T> std::ops::Deref for QueryResult<'a, T> {
    type Target = RefMut<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, E> Default for QueryRef<T, E> {
    fn default() -> Self {
        QueryRef {
            result: RefCell::new(None),
        }
    }
}
