#![feature(
    array_chunks,
    iter_array_chunks,
    portable_simd,
    array_windows,
    type_changing_struct_update,
    core_intrinsics
)]

pub mod binary_search;
pub mod btree;
pub mod eytzinger;
pub mod interp_search;
pub mod node;
pub mod partitioned_s_tree;
pub mod s_tree;
mod test;
pub mod util;

use std::marker::PhantomData;

use util::*;

#[ctor::ctor]
fn init_color_backtrace() {
    // color_backtrace::install();
}

/// Construct the data structure from a sorted vector.
pub trait SearchIndex: Sized + Sync {
    fn new(_vals: &[u32]) -> Self {
        unimplemented!()
    }

    /// Size of the index in bytes.
    fn size(&self) -> usize;

    /// Number of layers in the tree, corresponding to the number of memory accesses needed for each query.
    fn layers(&self) -> usize;

    // Convenience methods to forward to a search scheme.
    fn query_one(&self, q: u32, scheme: &(impl SearchScheme<Self> + ?Sized)) -> u32 {
        scheme.query_one(&self, q)
    }
    fn query(&self, qs: &[u32], scheme: &(impl SearchScheme<Self> + ?Sized)) -> Vec<u32> {
        scheme.query(&self, qs)
    }
}

/// Add a search scheme to an index.
pub trait SearchScheme<INDEX>: Sync {
    fn query_one(&self, index: &INDEX, q: u32) -> u32 {
        self.query(index, &vec![q])[0]
    }
    fn query(&self, index: &INDEX, qs: &[u32]) -> Vec<u32> {
        qs.iter().map(|&q| self.query_one(index, q)).collect()
    }
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl<I, F: Fn(&I, u32) -> u32 + Sync> SearchScheme<I> for F {
    fn query_one(&self, index: &I, q: u32) -> u32 {
        self(index, q)
    }
}

// Wrapper for batching queries.

pub struct Batched<const P: usize, I, F: for<'a> Fn(&'a I, &[u32; P]) -> [u32; P] + Sync>(
    F,
    PhantomData<fn(&I)>,
);

pub const fn batched<const P: usize, I, F: for<'a> Fn(&'a I, &[u32; P]) -> [u32; P] + Sync>(
    f: F,
) -> Batched<P, I, F> {
    Batched(f, PhantomData)
}

impl<const P: usize, I, F: for<'a> Fn(&'a I, &[u32; P]) -> [u32; P] + Sync> SearchScheme<I>
    for Batched<P, I, F>
{
    fn query(&self, index: &I, qs: &[u32]) -> Vec<u32> {
        let it = qs.array_chunks();
        assert!(
            it.remainder().is_empty(),
            "For now, batched queries cannot handle leftovers"
        );
        it.flat_map(|qb| (self.0)(index, qb)).collect()
    }
}

// Wrapper for full queries.

pub struct Full<I, F: for<'a> Fn(&'a I, &[u32]) -> Vec<u32> + Sync>(F, PhantomData<fn(&I)>);

pub const fn full<I, F: for<'a> Fn(&'a I, &[u32]) -> Vec<u32> + Sync>(f: F) -> Full<I, F> {
    Full(f, PhantomData)
}

impl<I, F: for<'a> Fn(&'a I, &[u32]) -> Vec<u32> + Sync> SearchScheme<I> for Full<I, F> {
    fn query(&self, index: &I, qs: &[u32]) -> Vec<u32> {
        (self.0)(index, qs)
    }
}
