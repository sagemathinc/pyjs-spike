//! Parallel map when the `parallel` feature is on, sequential otherwise.

#[cfg(feature = "parallel")]
use rayon::prelude::*;

pub fn map_range<T: Send, F: Fn(usize) -> T + Sync + Send>(n: usize, f: F) -> Vec<T> {
    #[cfg(feature = "parallel")]
    return (0..n).into_par_iter().map(f).collect();
    #[cfg(not(feature = "parallel"))]
    return (0..n).map(f).collect();
}

pub fn map_slice<A: Sync, T: Send, F: Fn(&A) -> T + Sync + Send>(xs: &[A], f: F) -> Vec<T> {
    #[cfg(feature = "parallel")]
    return xs.par_iter().map(f).collect();
    #[cfg(not(feature = "parallel"))]
    return xs.iter().map(f).collect();
}

pub fn for_each_mut<A: Send, F: Fn(usize, &mut A) + Sync + Send>(xs: &mut [A], f: F) {
    #[cfg(feature = "parallel")]
    xs.par_iter_mut().enumerate().for_each(|(i, x)| f(i, x));
    #[cfg(not(feature = "parallel"))]
    xs.iter_mut().enumerate().for_each(|(i, x)| f(i, x));
}
