#![allow(dead_code)]
use cmov::Cmov;
use std::intrinsics::select_unpredictable;
use std::simd::Simd;

use crate::{prefetch_index, vec_on_hugepages, SearchIndex};

pub struct SortedVec {
    pub(super) vals: Vec<u32>,
}

impl SortedVec {
    pub(super) fn get(&self, index: usize) -> u32 {
        unsafe { *self.vals.get_unchecked(index) }
    }
}

impl SearchIndex for SortedVec {
    fn new(vals: &[u32]) -> Self {
        assert!(vals.is_sorted());
        let mut vec = vec_on_hugepages(vals.len()).unwrap();
        vec.copy_from_slice(vals);
        SortedVec { vals: vec }
    }

    fn layers(&self) -> usize {
        self.vals.len().ilog2() as usize + 1
    }

    fn size(&self) -> usize {
        std::mem::size_of_val(self.vals.as_slice())
    }
}

impl SortedVec {
    /// Return the value of the first value >= query.
    pub fn binary_search(&self, q: u32) -> u32 {
        let mut l = 0;
        let mut r = self.vals.len();
        while l < r {
            let m = (l + r) / 2;
            if self.get(m) < q {
                l = m + 1;
            } else {
                r = m;
            }
        }
        self.get(l)
    }

    /// Return the value of the first value >= query.
    pub fn binary_search_std(&self, q: u32) -> u32 {
        let idx = self.vals.binary_search(&q).unwrap_or_else(|i| i);
        self.vals[idx]
    }

    /// branchless search using the cmov crate
    pub fn binary_search_branchless_cmov(&self, q: u32) -> u32 {
        let mut base: u64 = 0;
        let mut len: u64 = self.vals.len() as u64;
        while len > 1 {
            let half = len / 2;
            base.cmovnz(
                &(base + half),
                (self.get((base + half - 1) as usize) < q) as u8,
            );
            len = len - half;
        }
        self.get(base as usize)
    }

    /// branchless search
    pub fn binary_search_branchless(&self, q: u32) -> u32 {
        let mut base: u64 = 0;
        let mut len: u64 = self.vals.len() as u64;
        while len > 1 {
            let half = len / 2;
            let cmp = self.get((base + half - 1) as usize) < q;
            base = select_unpredictable(cmp, base + half, base);
            len = len - half;
        }
        self.get(base as usize)
    }

    /// branchless search with prefetching
    pub fn binary_search_branchless_prefetch(&self, q: u32) -> u32 {
        let mut base: u64 = 0;
        let mut len: u64 = self.vals.len() as u64;
        while len > 1 {
            let half = len / 2;
            prefetch_index(&self.vals, (base + half / 2 - 1) as usize);
            prefetch_index(&self.vals, (base + half + half / 2 - 1) as usize);
            let cmp = self.get((base + half - 1) as usize) < q;
            base = select_unpredictable(cmp, base + half, base);
            len = len - half;
        }
        self.get(base as usize)
    }

    // pub fn batch_impl_binary_search<const P: usize>(&self, qb: &[u32; P]) -> [u32; P] {
    //     let mut bases = [0; P];
    //     let mut len = self.vals.len();
    //     while len > 1 {
    //         let half = len / 2;
    //         for i in 0..P {
    //             bases[i] += (self.get(bases[i] + half - 1) < qb[i]) as usize * half;
    //         }
    //         len = len - half;
    //     }

    //     bases.map(|x| self.get(x))
    // }

    pub fn batch_impl_binary_search_branchless<const P: usize>(&self, qb: &[u32; P]) -> [u32; P] {
        let mut bases = [0u64; P];
        let mut len = self.vals.len() as u64;
        while len > 1 {
            let half = len / 2;
            for i in 0..P {
                let cmp = self.get((bases[i] + half - 1) as usize) < qb[i];
                bases[i] = select_unpredictable(cmp, bases[i] + half, bases[i]);
            }
            len = len - half;
        }

        bases.map(|x| self.get(x as usize))
    }

    pub fn batch_impl_binary_search_branchless_prefetch<const P: usize>(
        &self,
        qb: &[u32; P],
    ) -> [u32; P] {
        let mut bases = [0u64; P];
        let mut len = self.vals.len() as u64;
        while len > 1 {
            let half = len / 2;
            len = len - half;
            for i in 0..P {
                let cmp = self.get((bases[i] + half - 1) as usize) < qb[i];
                bases[i] = select_unpredictable(cmp, bases[i] + half, bases[i]);
                prefetch_index(&self.vals, (bases[i] + half / 2 - 1) as usize);
            }
        }

        bases.map(|x| self.get(x as usize))
    }
}
