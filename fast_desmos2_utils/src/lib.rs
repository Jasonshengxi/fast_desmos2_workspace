pub mod iter;
mod options;
mod vecs;

// pub use iter::{into_exactly, take_n, try_take_n};
pub use options::{OptExt, ResExt};
pub use vecs::{IdVec, SparseVec};

pub fn leak<T>(value: T) -> &'static mut T {
    Box::leak(Box::new(value))
}
