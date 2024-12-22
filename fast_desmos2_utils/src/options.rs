pub trait OptExt {
    type T;
    fn unwrap_unreach(self) -> Self::T;
}

pub trait ResExt {
    type T;
    fn unwrap_unreach(self) -> Self::T;
    fn assert_ok(self) -> Self;
}

impl<T> OptExt for Option<T> {
    type T = T;

    #[track_caller]
    fn unwrap_unreach(self) -> Self::T {
        self.unwrap_or_else(|| unreachable!())
    }
}
impl<T, E> ResExt for Result<T, E> {
    type T = T;

    #[track_caller]
    fn unwrap_unreach(self) -> Self::T {
        self.unwrap_or_else(|_| unreachable!())
    }

    #[track_caller]
    fn assert_ok(self) -> Self {
        self.map_err(|_| unreachable!())
    }
}
