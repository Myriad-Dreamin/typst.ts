macro_rules! static_assert {
    ($e:expr) => {
        #[allow(clippy::assertions_on_constants)]
        const _: () = assert!($e);
    };
}

pub(crate) use static_assert;
