use crate::{prefetch_index, vec_on_hugepages, SearchIndex};
use cmov::Cmov;
use std::intrinsics::select_unpredictable;

fn search_result_to_index(idx: usize) -> usize {
    idx >> (idx.trailing_ones() + 1)
}

pub struct Eytzinger {
    vals: Vec<u32>,
    num_iters: usize,
}

impl Eytzinger {
    fn get(&self, index: usize) -> u32 {
        unsafe { *self.vals.get_unchecked(index) }
    }

    fn get_next_index_branchless(&self, idx: usize, q: u32) -> usize {
        let mut idx_u64 = 2 * idx as u64;
        let candidate = (2 * idx + 1) as u64;
        // the OR here is a hack; it is done to achieve the same result algorithmica does.
        // We have to do this because we're using unsigned integers and they are using signed, so they use -1 as their "value not found"
        // retval. Therefore, they can do their last check against -1 at position 0 in the vector, which always results in the comparison
        // being true.

        let in_bounds = idx < self.vals.len();
        let idx = if in_bounds { idx } else { 0 };
        idx_u64.cmovnz(&candidate, (q > self.get(idx) || !in_bounds) as u8);
        idx_u64 as usize
    }

    pub fn new_no_hugepages(vals: &[u32]) -> Self {
        Self::new_impl(vals, false)
    }

    fn new_impl(vals: &[u32], hugepages: bool) -> Self {
        // +1 for one-based indexing
        let len = vals.len() + 1;
        let mut e = Eytzinger {
            vals: if hugepages {
                vec_on_hugepages(len).unwrap()
            } else {
                vec![0; len]
            },
            num_iters: len.ilog2() as usize,
        };
        e.vals[0] = u32::MAX;

        /// A recursive function to actually perform the Eytzinger transformation
        /// NOTE: This is not in-place.
        fn recurse(e: &mut Eytzinger, a: &[u32], i: &mut usize, k: usize) {
            if k <= a.len() {
                recurse(e, a, i, 2 * k);
                e.vals[k] = a[*i];
                *i += 1;
                recurse(e, a, i, 2 * k + 1);
            }
        }

        recurse(&mut e, &vals, &mut 0, 1);
        e
    }
}

impl SearchIndex for Eytzinger {
    fn new(vals: &[u32]) -> Self {
        Self::new_impl(vals, true)
    }

    fn layers(&self) -> usize {
        self.vals.len().ilog2() as usize + 1
    }

    fn size(&self) -> usize {
        std::mem::size_of_val(self.vals.as_slice())
    }
}

impl Eytzinger {
    pub fn search(&self, q: u32) -> u32 {
        let mut idx = 1;
        while idx < self.vals.len() {
            idx = 2 * idx + (q > self.get(idx)) as usize;
        }
        idx = search_result_to_index(idx);
        self.get(idx)
    }

    pub fn search_branchless(&self, q: u32) -> u32 {
        let mut idx = 1;
        // do a constant number of iterations
        for _ in 0..self.num_iters {
            let jump_to = (q > self.get(idx)) as usize;
            idx = 2 * idx + jump_to;
        }

        // let cmp_idx = if idx < self.vals.len() { idx } else { 0 };
        idx = self.get_next_index_branchless(idx, q);
        idx = search_result_to_index(idx);
        self.get(idx)
    }

    /// L: number of levels ahead to prefetch.
    pub fn search_prefetch<const L: usize>(&self, q: u32) -> u32 {
        let mut idx = 1;
        while (1 << L) * idx < self.vals.len() {
            idx = 2 * idx + (q > self.get(idx)) as usize;
            prefetch_index(&self.vals, (1 << L) * idx);
        }
        while idx < self.vals.len() {
            idx = 2 * idx + (q > self.get(idx)) as usize;
        }
        idx = search_result_to_index(idx);
        self.get(idx)
    }

    pub fn search_branchless_prefetch<const L: usize>(&self, q: u32) -> u32 {
        let mut idx = 1;
        let prefetch_until = self.num_iters as isize - L as isize;
        for _ in 0..prefetch_until {
            let jump_to = (q > self.get(idx)) as usize;
            idx = 2 * idx + jump_to;
            // the extra prefetch is apparently very slow here; why?
            prefetch_index(&self.vals, (1 << L) * idx);
        }

        for _ in prefetch_until..(self.num_iters as isize) {
            let jump_to = (q > self.get(idx)) as usize;
            idx = 2 * idx + jump_to;
        }

        idx = self.get_next_index_branchless(idx, q);
        idx = search_result_to_index(idx);
        self.get(idx)
    }

    pub fn batch_impl<const P: usize>(&self, qb: &[u32; P]) -> [u32; P] {
        let mut k = [1; P]; // current indices

        for _ in 0..self.num_iters {
            for i in 0..P {
                let jump_to = (self.get(k[i]) < qb[i]) as usize;
                k[i] = 2 * k[i] + jump_to;
            }
        }
        for i in 0..P {
            k[i] = self.get_next_index_branchless(k[i], qb[i]);
            k[i] = search_result_to_index(k[i]);
        }

        k.map(|x| self.get(x))
    }

    pub fn batch_impl_prefetched<const P: usize, const L: usize>(&self, qb: &[u32; P]) -> [u32; P] {
        let mut k = [1; P]; // current indices
        let prefetch_until = self.num_iters as isize - L as isize;

        for _ in 0..prefetch_until {
            for i in 0..P {
                let jump_to = (self.get(k[i]) < qb[i]) as usize;
                k[i] = 2 * k[i] + jump_to;
                prefetch_index(&self.vals, (1 << L) * k[i]);
            }
        }

        for _ in prefetch_until..(self.num_iters as isize) {
            for i in 0..P {
                let jump_to = (self.get(k[i]) < qb[i]) as usize;
                k[i] = 2 * k[i] + jump_to;
            }
        }

        for i in 0..P {
            k[i] = self.get_next_index_branchless(k[i], qb[i]);
            k[i] = search_result_to_index(k[i]);
        }
        // println!("{:?}", k);
        k.map(|x| self.get(x))
    }
}

#[cfg(test)]
mod tests {

    use crate::{binary_search::SortedVec, SearchIndex};

    use super::*;

    #[test]
    fn eytzinger_vs_binsearch() {
        let input = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let q = 5;
        let ey_res = Eytzinger::new(&input).search(q);
        let bin_res = SortedVec::new(&input).binary_search(q);
        println!("{ey_res}, {bin_res}");
    }

    #[test]
    fn eytzinger_test_pow2_min_1() {
        let input = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        #[rustfmt::skip]
        let corr_output = vec![u32::MAX, 8, 4, 12, 2, 6, 10, 14, 1, 3, 5, 7, 9, 11, 13, 15];
        let e = Eytzinger::new(&input);
        assert_eq!(e.vals, corr_output);
    }

    #[test]
    fn eytzinger_test_non_pow2() {
        let input = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let corr_output = vec![u32::MAX, 6, 3, 8, 1, 5, 7, 9, 0, 2, 4];
        let e = Eytzinger::new(&input);
        assert_eq!(e.vals, corr_output);
    }

    #[test]
    fn eyetzinger_search_test() {
        let input = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let q: u32 = 3;
        let ey_res = Eytzinger::new(&input).search(q);
        assert_eq!(ey_res, 3);
    }

    #[test]
    fn eyetzinger_search_oob() {
        let input = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let q: u32 = 12;
        let ey_res = Eytzinger::new(&input).search(q);
        assert_eq!(ey_res, u32::MAX);
    }
}
