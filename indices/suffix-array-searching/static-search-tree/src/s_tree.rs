use std::array::from_fn;
use std::iter::repeat;
use std::{fmt::Debug, iter::zip, simd::Simd};

use itertools::Itertools;

use crate::node::{BTreeNode, MAX};
use crate::{prefetch_index, prefetch_ptr, vec_on_hugepages, SearchIndex};

/// N total elements in a node.
/// B branching factor.
/// B-1 actual elements in a node.
#[repr(C)]
#[derive(Debug)]
pub struct STree<const B: usize, const N: usize> {
    tree: Vec<BTreeNode<N>>,
    offsets: Vec<usize>,
}

pub type STree16 = STree<16, 16>;
pub type STree15 = STree<15, 16>;

pub(crate) struct TreeBase<const B: usize>;

/// Wrap some helper functions.
impl<const B: usize> TreeBase<B> {
    pub fn blocks(n: usize) -> usize {
        n.div_ceil(B)
    }
    pub fn prev_keys(n: usize) -> usize {
        Self::blocks(n).div_ceil(B + 1) * B
    }
    pub fn height(n: usize) -> usize {
        if n <= B {
            1
        } else {
            Self::height(Self::prev_keys(n)) + 1
        }
    }
    pub fn layer_size(mut n: usize, h: usize, height: usize) -> usize {
        for _ in h..height - 1 {
            n = Self::prev_keys(n);
        }
        n
    }
}

impl<const B: usize, const N: usize> SearchIndex for STree<B, N> {
    fn new(vals: &[u32]) -> Self {
        Self::new_params(vals, false, false, false)
    }

    fn layers(&self) -> usize {
        self.offsets.len()
    }

    fn size(&self) -> usize {
        std::mem::size_of_val(self.tree.as_slice())
    }
}

impl<const B: usize, const N: usize> STree<B, N> {
    // Helper functions for unchecked indexing.
    fn node(&self, b: usize) -> &BTreeNode<N> {
        unsafe { &*self.tree.get_unchecked(b) }
    }

    fn get(&self, b: usize, i: usize) -> u32 {
        unsafe { *self.tree.get_unchecked(b).data.get_unchecked(i) }
    }

    // The `hugepages` parameter is only internal.
    pub fn new_params(
        vals: &[u32],
        left_max: bool,
        reverse_storage: bool,
        full_array: bool,
    ) -> Self {
        let hugepages = true;

        if full_array {
            assert!(
                !reverse_storage,
                "Full array only makes sense in forward layout."
            );
        }

        for &v in vals {
            assert!(v <= MAX);
        }

        let n = vals.len();

        assert!(n > 0);
        let height = TreeBase::<B>::height(n);
        assert!(height > 0);
        let layer_sizes = if full_array {
            (0..height).map(|h| (B + 1).pow(h as u32)).collect_vec()
        } else {
            (0..height)
                .map(|h| TreeBase::<B>::layer_size(n, h, height).div_ceil(B))
                .collect_vec()
        };
        assert!(layer_sizes[0] > 0);
        let n_blocks = layer_sizes.iter().sum::<usize>();

        let offsets = if !reverse_storage {
            layer_sizes
                .iter()
                .scan(0, |sum, sz| {
                    let offset = *sum;
                    *sum += sz;
                    Some(offset)
                })
                .collect_vec()
        } else {
            layer_sizes
                .iter()
                .scan(0, |sum, sz| {
                    *sum += sz;
                    Some(n_blocks - *sum)
                })
                .collect_vec()
        };

        let mut tree = if hugepages {
            vec_on_hugepages(n_blocks).unwrap()
        } else {
            vec![BTreeNode { data: [MAX; N] }; n_blocks]
        };
        // eprintln!("Len {} Capacity {}", tree.len(), tree.capacity());

        // Copy the input values to the last layer.
        let ol = offsets[height - 1];
        for (i, &val) in vals.iter().enumerate() {
            tree[ol + i / B].data[i % B] = val;
            // If B<N and there is some buffer space in each node, fill in the next larger element.
            if B < N && i % B == 0 && i > 0 {
                tree[ol + i / B - 1].data[B] = val;
            }
        }

        // Pad the last node in the initial layer with MAX.
        if n / B < layer_sizes[height - 1] {
            tree[ol + n / B].data[n % B..].fill(MAX);
        }

        // Initialize layers; based on Algorithmica.
        // https://en.algorithmica.org/hpc/data-structures/s-tree/#construction-1
        for h in (0..height - 1).rev() {
            let oh = offsets[h];
            // First initialize all nodes in the layer with MAX.
            tree[oh..oh + layer_sizes[h]].iter_mut().for_each(|node| {
                node.data.fill(MAX);
            });

            for i in 0..B * layer_sizes[h] {
                let mut k = i / B;
                let j = i % B;
                k = k * (B + 1) + j + 1;
                for _l in h..height - 2 {
                    k *= B + 1;
                }
                tree[oh + i / B].data[i % B] = if k * B < n {
                    if !left_max {
                        tree[ol + k].data[0]
                    } else {
                        tree[ol + k - 1].data[B - 1]
                    }
                } else {
                    MAX
                };
            }
        }
        assert!(offsets.len() > 0);
        Self { tree, offsets }
    }

    fn search_with_find_impl(&self, q: u32, find: impl Fn(&BTreeNode<N>, u32) -> usize) -> u32 {
        let mut k = 0;
        for [o, _o2] in self.offsets.array_windows() {
            let jump_to = find(self.node(o + k), q);
            k = k * (B + 1) + jump_to;
        }

        let o = self.offsets.last().unwrap();
        let idx = find(self.node(o + k), q);
        self.get(o + k + idx / N, idx % N)
    }

    pub const fn search_with_find(
        find: impl Fn(&BTreeNode<N>, u32) -> usize + Copy,
    ) -> impl Fn(&STree<B, N>, u32) -> u32 {
        move |index, q| index.search_with_find_impl(q, find)
    }

    pub fn search(&self, q: u32) -> u32 {
        let mut k = 0;
        for [o, _o2] in self.offsets.array_windows() {
            let jump_to = self.node(o + k).find(q);
            k = k * (B + 1) + jump_to;
        }

        let o = self.offsets.last().unwrap();
        let idx = self.node(o + k).find(q);
        self.get(o + k + idx / N, idx % N)
    }

    fn batch_impl<const P: usize, const PF: bool>(&self, qb: &[u32; P]) -> [u32; P] {
        let mut k = [0; P];
        for [o, o2] in self.offsets.array_windows() {
            for i in 0..P {
                let jump_to = self.node(o + k[i]).find(qb[i]);
                k[i] = k[i] * (B + 1) + jump_to;

                if PF {
                    prefetch_index(&self.tree, o2 + k[i]);
                }
            }
        }

        let o = self.offsets.last().unwrap();
        from_fn(|i| {
            let idx = self.node(o + k[i]).find(qb[i]);
            self.get(o + k[i] + idx / N, idx % N)
        })
    }

    pub fn batch<const P: usize>(&self, qb: &[u32; P]) -> [u32; P] {
        self.batch_impl::<P, false>(qb)
    }
    pub fn batch_prefetch<const P: usize>(&self, qb: &[u32; P]) -> [u32; P] {
        self.batch_impl::<P, true>(qb)
    }

    pub fn batch_splat<const P: usize>(&self, qb: &[u32; P]) -> [u32; P] {
        let mut k = [0; P];
        let q_simd = qb.map(|q| Simd::<u32, 8>::splat(q));
        for [o, o2] in self.offsets.array_windows() {
            for i in 0..P {
                let jump_to = self.node(o + k[i]).find_splat(q_simd[i]);
                k[i] = k[i] * (B + 1) + jump_to;
                prefetch_index(&self.tree, o2 + k[i]);
            }
        }

        let o = self.offsets.last().unwrap();
        from_fn(|i| {
            let idx = self.node(o + k[i]).find_splat(q_simd[i]);
            self.get(o + k[i] + idx / N, idx % N)
        })
    }

    pub fn batch_ptr<const P: usize>(&self, qb: &[u32; P]) -> [u32; P] {
        let mut k = [0; P];
        let q_simd = qb.map(|q| Simd::<u32, 8>::splat(q));

        let offsets = self
            .offsets
            .iter()
            .map(|o| unsafe { self.tree.as_ptr().add(*o) })
            .collect_vec();

        for [o, o2] in offsets.array_windows() {
            for i in 0..P {
                let jump_to = unsafe { *o.add(k[i]) }.find_splat(q_simd[i]);
                k[i] = k[i] * (B + 1) + jump_to;
                prefetch_ptr(unsafe { o2.add(k[i]) });
            }
        }

        let o = offsets.last().unwrap();
        from_fn(|i| {
            let idx = unsafe { *o.add(k[i]) }.find_splat(q_simd[i]);
            unsafe { *(*o.add(k[i] + idx / N)).data.get_unchecked(idx % N) }
        })
    }

    pub fn batch_byte_ptr<const P: usize>(&self, qb: &[u32; P]) -> [u32; P] {
        let mut k = [0; P];
        let q_simd = qb.map(|q| Simd::<u32, 8>::splat(q));

        let offsets = self
            .offsets
            .iter()
            .map(|o| unsafe { self.tree.as_ptr().add(*o) })
            .collect_vec();

        for [o, o2] in offsets.array_windows() {
            for i in 0..P {
                let jump_to = unsafe { *o.byte_add(k[i]) }.find_splat(q_simd[i]);
                k[i] = k[i] * (B + 1) + jump_to * 64;
                prefetch_ptr(unsafe { o2.byte_add(k[i]) });
            }
        }

        let o = offsets.last().unwrap();
        from_fn(|i| {
            let idx = unsafe { *o.byte_add(k[i]) }.find_splat(q_simd[i]);
            unsafe { (o.byte_add(k[i]) as *const u32).add(idx).read() }
        })
    }

    pub fn batch_final<const P: usize>(&self, qb: &[u32; P]) -> [u32; P] {
        let mut k = [0; P];
        let q_simd = qb.map(|q| Simd::<u32, 8>::splat(q));

        let offsets = self
            .offsets
            .iter()
            .map(|o| unsafe { self.tree.as_ptr().add(*o) })
            .collect_vec();

        for [o, o2] in offsets.array_windows() {
            for i in 0..P {
                let jump_to = unsafe { *o.byte_add(k[i]) }.find_splat64(q_simd[i]);
                k[i] = k[i] * (B + 1) + jump_to;
                prefetch_ptr(unsafe { o2.byte_add(k[i]) });
            }
        }

        let o = offsets.last().unwrap();
        from_fn(|i| {
            let idx = unsafe { *o.byte_add(k[i]) }.find_splat(q_simd[i]);
            unsafe { (o.byte_add(k[i]) as *const u32).add(idx).read() }
        })
    }

    pub fn batch_skip_prefetch<const P: usize, const SKIP: usize>(
        &self,
        qb: &[u32; P],
    ) -> [u32; P] {
        let mut k = [0; P];
        let q_simd = qb.map(|q| Simd::<u32, 8>::splat(q));

        let offsets = self
            .offsets
            .iter()
            .map(|o| unsafe { self.tree.as_ptr().add(*o) })
            .collect_vec();

        let skip = SKIP.min(offsets.len() - 1);

        for o in &offsets[..skip] {
            // let o2 = unsafe { *offsets.get_unchecked(h - 1) };
            for i in 0..P {
                let jump_to = unsafe { *o.byte_add(k[i]) }.find_splat64(q_simd[i]);
                k[i] = k[i] * (B + 1) + jump_to;
                // prefetch_ptr(unsafe { o2.byte_add(k[i]) });
            }
        }

        for [o, o2] in offsets[skip..].array_windows() {
            for i in 0..P {
                let jump_to = unsafe { *o.byte_add(k[i]) }.find_splat64(q_simd[i]);
                k[i] = k[i] * (B + 1) + jump_to;
                prefetch_ptr(unsafe { o2.byte_add(k[i]) });
            }
        }

        let o = offsets.last().unwrap();
        from_fn(|i| {
            let idx = unsafe { *o.byte_add(k[i]) }.find_splat(q_simd[i]);
            unsafe { (o.byte_add(k[i]) as *const u32).add(idx).read() }
        })
    }

    pub fn batch_final_full<const P: usize>(&self, qb: &[u32; P]) -> [u32; P] {
        let mut k = [0; P];
        let q_simd = qb.map(|q| Simd::<u32, 8>::splat(q));

        let o = self.tree.as_ptr();

        for _l in 0..self.offsets.len() - 1 {
            for i in 0..P {
                let jump_to = unsafe { *o.byte_add(k[i]) }.find_splat64(q_simd[i]);
                k[i] = k[i] * (B + 1) + jump_to + 64;
                prefetch_ptr(unsafe { o.byte_add(k[i]) });
            }
        }

        from_fn(|i| {
            let idx = unsafe { *o.byte_add(k[i]) }.find_splat(q_simd[i]);
            unsafe { (o.byte_add(k[i]) as *const u32).add(idx).read() }
        })
    }

    pub fn batch_interleave_half<const P: usize>(&self, qs: &[u32]) -> Vec<u32> {
        if self.offsets.len() % 2 != 0 {
            return vec![];
        }
        assert!(self.offsets.len() % 2 == 0);

        let offsets = self
            .offsets
            .iter()
            .map(|o| unsafe { self.tree.as_ptr().add(*o) })
            .collect_vec();

        let mut k1 = [0; P];
        let mut k2 = [0; P];

        // temporary: assert h is even

        // 1. setup first P queries
        // 2. do 2nd half of first P queries, and first half of next; repeat
        // 3. last half of last P queries.

        let hh = self.offsets.len() / 2;

        let mut chunks = qs.array_chunks::<P>();
        assert!(
            chunks.remainder().is_empty(),
            "Interleave does not process trailing bits"
        );
        let mut out = Vec::with_capacity(qs.len());

        let c1 = chunks.next().unwrap();
        let mut q_simd1 = [Simd::<u32, 8>::splat(0); P];
        let mut q_simd2 = c1.map(|q| Simd::<u32, 8>::splat(q));

        let hs_first = 0..hh;
        let hs_second = hh..offsets.len() - 1;

        // 1
        for h1 in hs_first.clone() {
            let o = unsafe { *offsets.get_unchecked(h1) };
            let o2 = unsafe { *offsets.get_unchecked(h1 + 1) };
            for i in 0..P {
                let jump_to = unsafe { *o.byte_add(k2[i]) }.find_splat64(q_simd2[i]);
                k2[i] = k2[i] * (B + 1) + jump_to;
                prefetch_ptr(unsafe { o2.byte_add(k2[i]) });
            }
        }

        assert!(qs.len() % (2 * P) == 0);

        // 2
        for [c1, c2] in chunks.array_chunks() {
            // swap
            // (q_simd1, q_simd2) = (q_simd2, q_simd1);
            // (k1, k2) = (k2, k1);

            // C1

            q_simd1 = c1.map(|q| Simd::<u32, 8>::splat(q));
            k1.fill(0);

            // First hh levels of c1.
            // Last hh levels of c2, with the last level special.

            for (h1, h2) in zip(hs_first.clone(), hs_second.clone()) {
                // eprintln!("h1: {}, h2: {}", h1, h2);
                let o1 = unsafe { *offsets.get_unchecked(h1) };
                let o12 = unsafe { *offsets.get_unchecked(h1 + 1) };
                let o2 = unsafe { *offsets.get_unchecked(h2) };
                let o22 = unsafe { *offsets.get_unchecked(h2 + 1) };
                for i in 0..P {
                    // 1
                    let jump_to = unsafe { *o1.byte_add(k1[i]) }.find_splat64(q_simd1[i]);
                    k1[i] = k1[i] * (B + 1) + jump_to;
                    prefetch_ptr(unsafe { o12.byte_add(k1[i]) });

                    // 2
                    let jump_to = unsafe { *o2.byte_add(k2[i]) }.find_splat64(q_simd2[i]);
                    k2[i] = k2[i] * (B + 1) + jump_to;
                    prefetch_ptr(unsafe { o22.byte_add(k2[i]) });
                }
            }

            // eprintln!("h1: {}, h2: {}", hh, 0);

            let o1 = unsafe { *offsets.get_unchecked(hh - 1) };
            let o12 = unsafe { *offsets.get_unchecked(hh) };

            // last iteration is special, where h2 = 0.
            let o = offsets.last().unwrap();
            let ans: [u32; P] = from_fn(|i| {
                let jump_to = unsafe { *o1.byte_add(k1[i]) }.find_splat64(q_simd1[i]);
                k1[i] = k1[i] * (B + 1) + jump_to;
                prefetch_ptr(unsafe { o12.byte_add(k1[i]) });

                let idx = unsafe { *o.byte_add(k2[i]) }.find_splat(q_simd2[i]);
                unsafe { (o.byte_add(k2[i]) as *const u32).add(idx).read() }
            });
            out.extend_from_slice(&ans);

            // C2

            q_simd2 = c2.map(|q| Simd::<u32, 8>::splat(q));
            k2.fill(0);

            // First hh levels of c2.
            // Last hh levels of c1, with the last level special.

            for (h1, h2) in zip(hs_first.clone(), hs_second.clone()) {
                // eprintln!("h1: {}, h2: {}", h1, h2);
                let o1 = unsafe { *offsets.get_unchecked(h1) };
                let o12 = unsafe { *offsets.get_unchecked(h1 + 1) };
                let o2 = unsafe { *offsets.get_unchecked(h2) };
                let o22 = unsafe { *offsets.get_unchecked(h2 + 1) };
                for i in 0..P {
                    // 1
                    let jump_to = unsafe { *o1.byte_add(k2[i]) }.find_splat64(q_simd2[i]);
                    k2[i] = k2[i] * (B + 1) + jump_to;
                    prefetch_ptr(unsafe { o12.byte_add(k2[i]) });

                    // 2
                    let jump_to = unsafe { *o2.byte_add(k1[i]) }.find_splat64(q_simd1[i]);
                    k1[i] = k1[i] * (B + 1) + jump_to;
                    prefetch_ptr(unsafe { o22.byte_add(k1[i]) });
                }
            }

            // eprintln!("h1: {}, h2: {}", hh, 0);

            let o1 = unsafe { *offsets.get_unchecked(hh - 1) };
            let o12 = unsafe { *offsets.get_unchecked(hh) };

            // last iteration is special, where h2 = 0.
            let o = offsets.last().unwrap();
            let ans: [u32; P] = from_fn(|i| {
                let jump_to = unsafe { *o1.byte_add(k2[i]) }.find_splat64(q_simd2[i]);
                k2[i] = k2[i] * (B + 1) + jump_to;
                prefetch_ptr(unsafe { o12.byte_add(k2[i]) });

                let idx = unsafe { *o.byte_add(k1[i]) }.find_splat(q_simd1[i]);
                unsafe { (o.byte_add(k1[i]) as *const u32).add(idx).read() }
            });
            out.extend_from_slice(&ans);
        }

        // 3
        for h2 in hs_second {
            // eprintln!("h2: {}", h2);
            let o = unsafe { *offsets.get_unchecked(h2) };
            let o2 = unsafe { *offsets.get_unchecked(h2 + 1) };
            for i in 0..P {
                let jump_to = unsafe { *o.byte_add(k2[i]) }.find_splat64(q_simd1[i]);
                k1[i] = k1[i] * (B + 1) + jump_to;
                prefetch_ptr(unsafe { o2.byte_add(k1[i]) });
            }
        }
        // h=0
        let o = offsets.last().unwrap();
        let ans: [u32; P] = from_fn(|i| {
            let idx = unsafe { *o.byte_add(k2[i]) }.find_splat(q_simd2[i]);
            unsafe { (o.byte_add(k2[i]) as *const u32).add(idx).read() }
        });
        out.extend_from_slice(&ans);
        out
    }

    pub fn batch_interleave_last<const P: usize, const X: usize>(&self, qs: &[u32]) -> Vec<u32> {
        if self.offsets.len() < 2 + X - 1 {
            return vec![];
        }

        let offsets = self
            .offsets
            .iter()
            .map(|o| unsafe { self.tree.as_ptr().add(*o) })
            .collect_vec();

        let mut k1 = &mut [0; P];
        let mut k2 = &mut [0; P];

        // temporary: assert h is even

        // 1. setup first P queries
        // 2. do 2nd half of first P queries, and first half of next; repeat
        // 3. last half of last P queries.

        let mut chunks = qs.array_chunks::<P>();
        assert!(
            chunks.remainder().is_empty(),
            "Interleave does not process trailing bits"
        );
        let mut out = Vec::with_capacity(qs.len());

        let c1 = chunks.next().unwrap();
        let mut q_simd1 = &mut [Simd::<u32, 8>::splat(0); P];
        let mut q_simd2 = &mut [Simd::<u32, 8>::splat(0); P];
        *q_simd1 = c1.map(|q| Simd::<u32, 8>::splat(q));

        let hs_first = 0..X - 1;
        let hs_mid = X - 1..offsets.len() - 2;
        let hs_second = offsets.len() - 2;

        // 1
        for h1 in hs_first.clone() {
            let o = unsafe { *offsets.get_unchecked(h1) };
            let o2 = unsafe { *offsets.get_unchecked(h1 + 1) };
            for i in 0..P {
                let jump_to = unsafe { *o.byte_add(k1[i]) }.find_splat64(q_simd1[i]);
                k1[i] = k1[i] * (B + 1) + jump_to;
                prefetch_ptr(unsafe { o2.byte_add(k1[i]) });
            }
        }

        // 2
        for c1 in chunks {
            // swap
            (q_simd1, q_simd2) = (q_simd2, q_simd1);
            (k1, k2) = (k2, k1);

            *q_simd1 = c1.map(|q| Simd::<u32, 8>::splat(q));
            k1.fill(0);

            // First hh levels of c1.
            // Last hh levels of c2, with the last level special.

            for h1 in hs_mid.clone() {
                let o1 = unsafe { *offsets.get_unchecked(h1) };
                let o12 = unsafe { *offsets.get_unchecked(h1 + 1) };
                for i in 0..P {
                    let jump_to = unsafe { *o1.byte_add(k2[i]) }.find_splat64(q_simd2[i]);
                    k2[i] = k2[i] * (B + 1) + jump_to;
                    prefetch_ptr(unsafe { o12.byte_add(k2[i]) });
                }
            }

            let h2 = hs_second;
            // eprintln!("h1: {}, h2: {}", h1, h2);
            let o2 = unsafe { *offsets.get_unchecked(h2) };
            let o22 = unsafe { *offsets.get_unchecked(h2 + 1) };

            let ols: [_; X] = from_fn(|i| unsafe { *offsets.get_unchecked(i) });
            for i in 0..P {
                for j in 0..X - 1 {
                    let jump_to = unsafe { *ols[j].byte_add(k1[i]) }.find_splat64(q_simd1[i]);
                    k1[i] = k1[i] * (B + 1) + jump_to;
                }
                // prefetch_ptr(unsafe { ols[X - 1].byte_add(k1[i]) });

                // 2
                let jump_to = unsafe { *o2.byte_add(k2[i]) }.find_splat64(q_simd2[i]);
                k2[i] = k2[i] * (B + 1) + jump_to;
                prefetch_ptr(unsafe { o22.byte_add(k2[i]) });
            }

            // eprintln!("h1: {}, h2: {}", hh, 0);

            // last iteration is special, where h2 = 0.
            let o = offsets.last().unwrap();
            let ans: [u32; P] = from_fn(|i| {
                let idx = unsafe { *o.byte_add(k2[i]) }.find_splat(q_simd2[i]);
                unsafe { (o.byte_add(k2[i]) as *const u32).add(idx).read() }
            });
            out.extend_from_slice(&ans);
        }

        for h1 in hs_mid.clone() {
            let o1 = unsafe { *offsets.get_unchecked(h1) };
            let o12 = unsafe { *offsets.get_unchecked(h1 + 1) };
            for i in 0..P {
                let jump_to = unsafe { *o1.byte_add(k1[i]) }.find_splat64(q_simd1[i]);
                k1[i] = k1[i] * (B + 1) + jump_to;
                prefetch_ptr(unsafe { o12.byte_add(k1[i]) });
            }
        }

        // 3
        let h2 = hs_second;
        {
            // eprintln!("h2: {}", h2);
            let o = unsafe { *offsets.get_unchecked(h2) };
            let o2 = unsafe { *offsets.get_unchecked(h2 + 1) };
            for i in 0..P {
                let jump_to = unsafe { *o.byte_add(k1[i]) }.find_splat64(q_simd1[i]);
                k1[i] = k1[i] * (B + 1) + jump_to;
                prefetch_ptr(unsafe { o2.byte_add(k1[i]) });
            }
        }
        // h=0
        let o = offsets.last().unwrap();
        let ans: [u32; P] = from_fn(|i| {
            let idx = unsafe { *o.byte_add(k1[i]) }.find_splat(q_simd1[i]);
            unsafe { (o.byte_add(k1[i]) as *const u32).add(idx).read() }
        });
        out.extend_from_slice(&ans);
        out
    }

    pub fn batch_interleave_all_128(&self, qs: &[u32]) -> Vec<u32> {
        match self.offsets.len() {
            1 => self.batch_interleave_all::<128, 1, 128>(qs),
            2 => self.batch_interleave_all::<64, 2, 128>(qs),
            3 => self.batch_interleave_all::<32, 3, 96>(qs),
            4 => self.batch_interleave_all::<32, 4, 128>(qs),
            5 => self.batch_interleave_all::<16, 5, 80>(qs),
            6 => self.batch_interleave_all::<16, 6, 96>(qs),
            7 => self.batch_interleave_all::<16, 7, 112>(qs),
            8 => self.batch_interleave_all::<16, 8, 128>(qs),
            _ => panic!("Unsupported tree height {}", self.offsets.len()),
        }
    }

    pub fn batch_interleave_all<const P: usize, const L: usize, const PL: usize>(
        &self,
        qs: &[u32],
    ) -> Vec<u32>
    where
        [(); PL]:,
    {
        if self.offsets.len() != L {
            return vec![];
        }
        assert_eq!(self.offsets.len(), L);

        let offsets = self
            .offsets
            .iter()
            .map(|o| unsafe { self.tree.as_ptr().add(*o) })
            .collect_vec();

        let offsets: &[_; L] = offsets.split_first_chunk().unwrap().0;

        // temporary: assert h is even

        // 1. setup first P queries
        // 2. do 2nd half of first P queries, and first half of next; repeat
        // 3. last half of last P queries.

        let zeros = [0; P];
        let buf = repeat(&zeros).take(L);
        let chunks = qs.array_chunks::<P>();
        assert!(
            chunks.remainder().is_empty(),
            "Interleave does not process trailing bits"
        );
        let chunks = chunks.chain(buf);

        let mut out = Vec::with_capacity(qs.len());

        let mut q_simd = [Simd::<u32, 8>::splat(0); PL];
        let mut k = [0; PL];

        let mut ans = [0; P];
        let ol = offsets.last().unwrap();

        // 2
        let mut first_i = (-(L as isize) as usize).wrapping_add(1);
        for (c, c1) in chunks.enumerate() {
            let mut i = first_i;
            first_i = first_i.wrapping_sub(1);
            if first_i == -(L as isize) as usize {
                first_i = 0;
            }
            let mut j = 0;

            // First incomplete iteration.
            {
                for l in 0..L - 1 {
                    // i>=0, but i is unsigned so we check from the other end.
                    if i < L * P {
                        let jump_to = unsafe { *offsets[l].byte_add(k[i]) }.find_splat64(q_simd[i]);
                        k[i] = k[i] * (B + 1) + jump_to;
                        prefetch_ptr(unsafe { offsets[l + 1].byte_add(k[i]) });
                    }
                    i = i.wrapping_add(1);
                }

                if i < PL {
                    ans[j] = {
                        let idx = unsafe { *ol.byte_add(k[i]) }.find_splat(q_simd[i]);
                        unsafe { (ol.byte_add(k[i]) as *const u32).add(idx).read() }
                    };
                    k[i] = 0;
                    q_simd[i] = Simd::splat(c1[j]);
                    j += 1;
                }

                i = i.wrapping_add(1);

                assert!(i < P);
            }

            // Middle

            loop {
                for l in 0..L - 1 {
                    let jump_to = unsafe { *offsets[l].byte_add(k[i]) }.find_splat64(q_simd[i]);
                    k[i] = k[i] * (B + 1) + jump_to;
                    prefetch_ptr(unsafe { offsets[l + 1].byte_add(k[i]) });
                    i += 1;
                }

                ans[j] = {
                    let idx = unsafe { *ol.byte_add(k[i]) }.find_splat(q_simd[i]);
                    unsafe { (ol.byte_add(k[i]) as *const u32).add(idx).read() }
                };
                k[i] = 0;
                q_simd[i] = Simd::splat(c1[j]);

                i += 1;
                j += 1;

                if i > PL - L {
                    break;
                }
            }

            // Last incomplete iteration.
            {
                for l in 0..L - 1 {
                    if i < L * P {
                        let jump_to = unsafe { *offsets[l].byte_add(k[i]) }.find_splat64(q_simd[i]);
                        k[i] = k[i] * (B + 1) + jump_to;
                        prefetch_ptr(unsafe { offsets[l + 1].byte_add(k[i]) });
                    }
                    i += 1;
                }

                if i < L * P {
                    ans[j] = {
                        let idx = unsafe { *ol.byte_add(k[i]) }.find_splat(q_simd[i]);
                        unsafe { (ol.byte_add(k[i]) as *const u32).add(idx).read() }
                    };
                }
                i += 1;
                // j += 1;

                assert!(i >= PL);
            }

            if c >= L {
                out.extend_from_slice(&ans);
            }
        }
        assert!(out.len() > 0, "qs {}", qs.len());
        out
    }

    /// Copy queries to a new output vector.
    #[inline(never)]
    pub fn baseline(&self, qs: &[u32]) -> Vec<u32> {
        qs.iter().map(|x| 2 * x).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{binary_search::SortedVec, SearchIndex};

    #[test]
    fn test_b_tree_k_2() {
        let vals = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let bptree = STree::<2, 2>::new(&vals);
        println!("{:?}", bptree);
    }

    #[test]
    fn test_b_tree_k_3() {
        let vals = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        // let correct_output = vec![4, 8, 12, 1, 2, 3, 5, 6, 7, 9, 10, 11, 13, 14, 15];
        let computed_out = STree::<3, 3>::new(&vals);
        println!("{:?}", computed_out);
    }

    #[test]
    fn test_bptree_search_bottom_layer() {
        let mut vals: Vec<u32> = (1..2000).collect();
        vals.push(MAX);
        let q = 452;
        let bptree = STree::<16, 16>::new(&vals);
        let bptree_res = bptree.search(q);

        let bin_res = SortedVec::new(&vals).binary_search(q);
        println!("{bptree_res}, {bin_res}");
        assert!(bptree_res == bin_res);
    }

    #[test]
    fn test_bptree_search_top_node() {
        let mut vals: Vec<u32> = (1..2000).collect();
        vals.push(MAX);
        let q = 289;
        let bptree = STree::<16, 16>::new(&vals);
        let bptree_res = bptree.search(q);

        let bin_res = SortedVec::new(&vals).binary_search(q);
        println!("{bptree_res}, {bin_res}");
        assert!(bptree_res == bin_res);
    }

    #[test]
    fn test_simd_cmp() {
        let mut vals: Vec<u32> = (1..16).collect();
        vals.push(MAX);
        let bptree = STree::<16, 16>::new(&vals);
        let idx = bptree.tree[0].find(1);
        println!("{}", idx);
        assert_eq!(idx, 0);
    }
}

/// FFI Functions
#[no_mangle]
pub extern "C" fn stree16_new(vals: *const u32, len: usize) -> *mut STree16 {
    let vals = unsafe { std::slice::from_raw_parts(vals, len) };
    let tree = STree16::new(vals);
    Box::into_raw(Box::new(tree))
}

#[no_mangle]
pub extern "C" fn stree16_search(tree: *const STree16, query: u32) -> u32 {
    let tree = unsafe { &*tree };
    tree.search(query)
}

#[no_mangle]
pub extern "C" fn stree16_free(tree: *mut STree16) {
    if !tree.is_null() {
        unsafe { Box::from_raw(tree) };
    }
}

#[no_mangle]
pub extern "C" fn stree16_size(tree: *mut STree16) -> usize {
    let tree = unsafe { &*tree };
    tree.size()
}
