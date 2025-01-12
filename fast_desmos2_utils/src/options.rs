pub trait OptExt {
    type T;
    fn unwrap_unreach(self) -> Self::T;
}

pub trait ResExt {
    type T;
    type E;

    fn unwrap_unreach(self) -> Self::T;
    fn assert_ok(self) -> Self;
}

impl<T> OptExt for Option<T> {
    type T = T;

    #[track_caller]
    fn unwrap_unreach(self) -> Self::T {
        #[track_caller]
        fn unreachable<U>() -> U {
            unreachable!()
        }
        self.unwrap_or_else(unreachable)
    }
}
impl<T, E> ResExt for Result<T, E> {
    type T = T;
    type E = E;

    #[track_caller]
    fn unwrap_unreach(self) -> Self::T {
        self.unwrap_or_else(unreachable)
    }

    #[track_caller]
    fn assert_ok(self) -> Self {
        self.map_err(unreachable)
    }
}

#[track_caller]
fn unreachable<T, U>(_: T) -> U {
    unreachable!()
}
