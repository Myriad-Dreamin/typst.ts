use std::cell::{RefCell, RefMut};

type QueryCell<Res, Err, QueryContext> = (Option<QueryContext>, Option<Result<Res, Err>>);

/// std::ops::DerefMut is disabled, since we can call compute_ref safely.
/// It means that multiple immutable references can be long lived.
pub struct QueryResult<'a, T>(RefMut<'a, T>);

impl<'a, T> std::ops::Deref for QueryResult<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Represent the result of an immutable query reference.
/// The compute function should be pure enough.
///
/// [`compute`]: Self::compute
/// [`compute_ref`]: Self::compute_ref
pub struct QueryRef<Res, Err, QueryContext = ()> {
    /// `None` means no value has been computed yet.
    cell: RefCell<QueryCell<Res, Err, QueryContext>>,
}

impl<T, E, QC> QueryRef<T, E, QC> {
    pub fn with_value(value: T) -> Self {
        Self {
            cell: RefCell::new((None, Some(Ok(value)))),
        }
    }

    pub fn with_context(ctx: QC) -> Self {
        Self {
            cell: RefCell::new((Some(ctx), None)),
        }
    }
}

impl<T, E: Clone, QC> QueryRef<T, E, QC> {
    /// Clone the error so that it can escape the borrowed reference to the ref cell.
    #[inline]
    fn clone_err(r: RefMut<'_, QueryCell<T, E, QC>>) -> E {
        let initialized_res = r.1.as_ref().unwrap();
        let checked_res = initialized_res.as_ref().map(|_| ());
        checked_res.unwrap_err().clone()
    }

    /// Get the reference to the query result, which asserts that the query result is initialized.
    #[inline]
    fn get_ref(&self) -> Result<&T, E> {
        let holding = unsafe { (*self.cell.as_ptr()).1.as_ref().unwrap_unchecked() };
        holding.as_ref().map_err(Clone::clone)
    }

    /// Compute and return a checked reference guard.
    #[inline]
    pub fn compute<F: FnOnce() -> Result<T, E>>(&self, f: F) -> Result<QueryResult<'_, T>, E> {
        self.compute_with_context(|_| f())
    }

    /// Compute with context and return a checked reference guard.
    #[inline]
    pub fn compute_with_context<F: FnOnce(QC) -> Result<T, E>>(
        &self,
        f: F,
    ) -> Result<QueryResult<'_, T>, E> {
        let borrowed = self.cell.borrow_mut();
        let result = RefMut::filter_map(
            borrowed,
            |(ref mut ctx, ref mut res): &mut QueryCell<T, E, QC>| -> Option<&mut T> {
                let get_or_init = || f(ctx.take().unwrap());
                res.get_or_insert_with(get_or_init).as_mut().ok()
            },
        );

        result.map(QueryResult).map_err(Self::clone_err)
    }

    /// Gets the reference to the (maybe uninitialized) result.
    ///
    /// Returns `None` if the cell is empty, or being initialized. This
    /// method never blocks.
    ///
    /// It is possible not hot, so that it is non-inlined
    pub fn get_uninitialized(&self) -> QueryResult<'_, Option<Result<T, E>>> {
        let borrowed = self.cell.borrow_mut();
        let result = RefMut::map(borrowed, |(_, ref mut res)| res);

        QueryResult(result)
    }

    /// Compute and return a unchecked reference guard.
    #[inline]
    pub fn compute_ref<F: FnOnce() -> Result<T, E>>(&self, f: F) -> Result<&T, E> {
        self.compute(f)?;

        // the query result is already initialized by f
        self.get_ref()
    }

    /// Compute with context and return a unchecked reference guard.
    #[inline]
    pub fn compute_with_context_ref<F: FnOnce(QC) -> Result<T, E>>(&self, f: F) -> Result<&T, E> {
        self.compute_with_context(f)?;
        self.get_ref()
    }
}

impl<T, E> Default for QueryRef<T, E> {
    fn default() -> Self {
        QueryRef {
            cell: RefCell::new((Some(()), None)),
        }
    }
}
