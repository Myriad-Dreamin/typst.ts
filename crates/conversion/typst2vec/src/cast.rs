pub trait FromTypst<T>: Sized {
    fn from_typst(value: T) -> Self;
}

pub trait IntoTypst<T>: Sized {
    fn into_typst(self) -> T;
}

impl<T, U> IntoTypst<U> for T
where
    U: FromTypst<T>,
{
    fn into_typst(self) -> U {
        U::from_typst(self)
    }
}

pub trait TryFromTypst<T>: Sized {
    type Error;

    fn try_from_typst(value: T) -> Result<Self, Self::Error>;
}

pub trait TryIntoTypst<T>: Sized {
    type Error;

    fn try_into_typst(self) -> Result<T, Self::Error>;
}

impl<T, U> TryIntoTypst<U> for T
where
    U: TryFromTypst<T>,
{
    type Error = U::Error;

    fn try_into_typst(self) -> Result<U, Self::Error> {
        U::try_from_typst(self)
    }
}
