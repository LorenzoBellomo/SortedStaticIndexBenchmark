use itertools::Itertools;
use std::any::type_name;
use std::array::from_fn;
use std::iter::repeat;
use std::mem::{size_of, size_of_val};
use std::{fmt::Debug, simd::Simd};

use crate::node::{BTreeNode, MAX};
use crate::s_tree::TreeBase;
use crate::{prefetch_ptr, vec_on_hugepages, SearchIndex};

/// N total elements in a node.
/// B branching factor.
/// B-1 actual elements in a node.
/// COMPACT: instead of a single tree with the first few layers removed,
///          store many small packed trees.
/// L1: reduce first branching factor to level 1.
#[derive(Debug)]
pub struct PartitionedSTree<const B: usize, const N: usize, Tp> {
    tree: Vec<BTreeNode<N>>,
    offsets: Vec<usize>,
    prefix_map: Vec<u32>,
    /// Amount to shift values/queries to the right to get their part.
    shift: usize,
    /// blocks per part
    bpp: usize,
    /// Number of nodes in layer 1.
    /// Number of values in the root is l1-1.
    l1: usize,
    overlap: usize,
    _tp: std::marker::PhantomData<Tp>,
}

pub trait Marker: Sync {
    const COMPACT: bool;
    const L1: bool;
    const OL: bool;
    const MAP: bool;
}

// Marker traits for the type of tree.
pub struct Simple;
// Compact layout: each tree is packed separately.
pub struct Compact;
// Extension of simple layout; not compact.
pub struct L1;
// Extension of L1 layout; not compact.
pub struct Overlapping;
// Extension over Overlapping.
pub struct Map;

impl Marker for Simple {
    const COMPACT: bool = false;
    const L1: bool = false;
    const OL: bool = false;
    const MAP: bool = false;
}
impl Marker for Compact {
    const COMPACT: bool = true;
    const L1: bool = false;
    const OL: bool = false;
    const MAP: bool = false;
}
impl Marker for L1 {
    const COMPACT: bool = false;
    const L1: bool = true;
    const OL: bool = false;
    const MAP: bool = false;
}
impl Marker for Overlapping {
    const COMPACT: bool = false;
    const L1: bool = true;
    const OL: bool = true;
    const MAP: bool = false;
}
impl Marker for Map {
    const COMPACT: bool = false;
    const L1: bool = true;
    const OL: bool = true;
    const MAP: bool = true;
}

// Workaround because <Marker<COMPACT=false>> is not supported.
pub trait NotCompact {}
impl NotCompact for Simple {}
impl NotCompact for L1 {}
impl NotCompact for Overlapping {}
impl NotCompact for Map {}

pub type PartitionedSTree16 = PartitionedSTree<16, 16, Simple>;
pub type PartitionedSTree15 = PartitionedSTree<15, 16, Simple>;
pub type PartitionedSTree16C = PartitionedSTree<16, 16, Compact>;
pub type PartitionedSTree15C = PartitionedSTree<15, 16, Compact>;
pub type PartitionedSTree16L = PartitionedSTree<16, 16, L1>;
pub type PartitionedSTree15L = PartitionedSTree<15, 16, L1>;
pub type PartitionedSTree16O = PartitionedSTree<16, 16, Overlapping>;
pub type PartitionedSTree15O = PartitionedSTree<15, 16, Overlapping>;
pub type PartitionedSTree16M = PartitionedSTree<16, 16, Map>;

impl<const B: usize, const N: usize, Tp: Marker> SearchIndex for PartitionedSTree<B, N, Tp> {
    fn size(&self) -> usize {
        size_of_val(self.tree.as_slice()) + size_of_val(self.prefix_map.as_slice())
    }

    fn layers(&self) -> usize {
        self.offsets.len() + Tp::MAP as usize
    }
}

impl<const B: usize, const N: usize, Tp: Marker> PartitionedSTree<B, N, Tp> {
    fn get_part_size(vals: &[u32], b: usize) -> (usize, usize, usize, usize, Option<usize>) {
        assert!(vals.is_sorted());
        assert!(*vals.last().unwrap() <= MAX);
        assert!(vals.len() > 0);

        let bits = 1 + vals.last().unwrap().ilog2() as usize;
        let mut shift = bits.saturating_sub(b);

        let mut parts = 1 << (bits - shift);

        // Compute bucket sizes.
        // For compact case, we need to store one sentinel of padding at the end of each part.
        let mut bucket_sizes = vec![if Tp::COMPACT { 1 } else { 0 }; parts];
        for &val in vals {
            let bucket = (val >> shift) as usize;
            bucket_sizes[bucket] += 1;
        }

        // When mapping, we play it safe and only assume a branching factor of 16 at the top level.
        let get_height =
            |x: usize| TreeBase::<B>::height(if Tp::MAP { (x * 17).div_ceil(16) } else { x });

        // Find largest bucket.
        let mut max_bucket = *bucket_sizes.iter().max().unwrap();
        // Number of layers for largest bucket.
        let mut height = get_height(max_bucket);
        eprintln!("{b}: Max bucket {max_bucket} => height {height}");

        // Try reducing the number of parts if the height stays the same.
        let mut b2 = b;
        loop {
            if b2 == 0 {
                break;
            }
            b2 -= 1;
            if b2 > bits {
                break;
            }
            let shift2 = bits.saturating_sub(b2);
            let parts2 = 1 << (bits - shift2);
            let mut bucket_sizes2 = vec![if Tp::COMPACT { 1 } else { 0 }; parts2];
            for &val in vals {
                let bucket = (val >> shift2) as usize;
                bucket_sizes2[bucket] += 1;
            }
            let max_bucket2 = *bucket_sizes2.iter().max().unwrap();
            let height2 = get_height(max_bucket2);
            eprintln!("{b2}: Max bucket {max_bucket2} => height {height2}");
            if height2 > height {
                // eprintln!("{bucket_sizes2:?}");
                break;
            }
            shift = shift2;
            parts = parts2;
            max_bucket = max_bucket2;
            bucket_sizes = bucket_sizes2;
            height = height2;
        }

        let overlap = if Tp::MAP {
            Some(0)
        } else if Tp::OL {
            let subtree_size = if height == 1 {
                1
            } else {
                B * (B + 1).pow(height as u32 - 2)
            };
            Self::max_overlap(&bucket_sizes, subtree_size)
        } else {
            None
        };

        eprintln!("shift {shift}");
        eprintln!("parts {parts}");
        eprintln!("max_bucket {max_bucket}");
        eprintln!("height {height}");
        eprintln!("overlap {overlap:?}");

        (shift, parts, max_bucket, height, overlap)
    }

    /// Each node can reach 17 subtrees.
    /// Without overlap, each node has its own 17 subtrees
    /// With overlap 0, each node has 1 shared with the previous one, and 16 new subtrees.
    /// With overlap 15, each node shares 15 subtrees with the previous one, and 1 new subtree.
    ///
    /// The question is what is the max overlap we can use.
    ///
    /// Returns none when each node really needs its own 17 subtrees.
    fn max_overlap(buckets: &[usize], subtree_size: usize) -> Option<usize> {
        if buckets.len() == 1 {
            if buckets[0] <= subtree_size {
                return Some(0);
            } else {
                return None;
            }
        }
        // remaining elements to be placed.
        // eprintln!("Bucket sizes: {:?}", buckets);
        eprintln!("Subtree size: {}", subtree_size);
        let capacity = 16 * subtree_size;
        'overlap: for overlap in (0..16).rev() {
            let mut x = 0;
            for b in buckets {
                x += b;
                if x > capacity {
                    eprintln!("overlap {overlap}: Current sum is {x}, capacity is {capacity}");
                    continue 'overlap;
                }
                x = x.saturating_sub((16 - overlap) * subtree_size);
            }
            eprintln!("Overlap: {}", overlap);
            return Some(overlap);
        }
        eprintln!("Overlap: None");
        None
    }
}

impl<const B: usize, const N: usize> PartitionedSTree<B, N, Compact> {
    pub fn new(vals: &[u32], b: usize) -> Self {
        Self::try_new(vals, b).unwrap()
    }

    /// Partition on the first `b` bits of each key before building the tree.
    /// Any bits beyond the maximum value are skipped.
    /// - uses hugepages.
    /// - uses forward layout.
    /// - uses 'rev' bucket order.
    /// - uses the full array.
    pub fn try_new(vals: &[u32], b: usize) -> Option<Self> {
        let n = vals.len();

        let (shift, parts, max_bucket, height, _overlap) = Self::get_part_size(vals, b);

        let layer_sizes;
        let offsets;
        let mut tree;
        let bpp;
        // In this case, L1 is meaningless!
        assert!(height > 0);
        // All layers are full, for indexing purposes.
        // TODO: Layer sizes given by max_bucket_size.
        layer_sizes = (0..height)
            .map(|h| TreeBase::<B>::layer_size(max_bucket, h, height).div_ceil(B))
            .collect_vec();
        assert!(layer_sizes[0] == 1);
        bpp = layer_sizes.iter().sum::<usize>();
        let n_blocks = parts * bpp;

        // Offsets within a part.
        offsets = layer_sizes
            .iter()
            .scan(0, |sum, sz| {
                let offset = *sum;
                *sum += sz;
                Some(offset)
            })
            .collect_vec();

        if n_blocks * size_of::<BTreeNode<N>>() > (32 << 30) {
            // Too much overhead.
            return None;
        }

        tree = vec_on_hugepages::<BTreeNode<N>>(n_blocks)?;

        // First initialize all nodes in the layer with MAX.
        for node in &mut tree {
            node.data.fill(MAX);
        }

        // Initialize the last layer.
        let mut prev_part = 0;
        let mut idx = 0;

        let h = height - 1;
        let ol = offsets[h]; // last-level offset
        for &val in vals {
            let part = (val >> shift) as usize;

            // For each completed part (typically just 1), append the current val.
            while prev_part < part {
                if idx / B < layer_sizes[h] {
                    tree[prev_part * bpp + ol + idx / B].data[idx % B..].fill(val);
                }
                prev_part += 1;
                idx = 0;
            }

            tree[part * bpp + ol + idx / B].data[idx % B] = val;
            // If B<N and there is some buffer space in each node,
            // put us also in the last element of the previous node.
            if B < N && idx % B == 0 && idx > 0 {
                tree[part * bpp + ol + idx / B - 1].data[B] = val;
            }
            idx += 1;
        }

        // Initialize the inner layers.
        for h in (0..height - 1).rev() {
            let oh = offsets[h];

            for part in 0..parts {
                for i in 0..B * layer_sizes[h] {
                    let mut k = i / B;
                    let j = i % B;
                    k = k * (B + 1) + j + 1;
                    for _l in h..height - 2 {
                        k *= B + 1;
                    }
                    tree[part * bpp + oh + i / B].data[i % B] = if k * B < max_bucket {
                        tree[part * bpp + ol + k - 1].data[B - 1]
                    } else {
                        MAX
                    };
                }
            }
        }
        assert!(offsets.len() > 0);

        eprintln!(
            "PartitionedSTree: n={} b={} shift={} parts={} height={} layer_sizes={:?} offsets={:?} compact=true max_bucket={max_bucket}",
            n, b, shift, parts, height, layer_sizes, offsets
        );
        // for (i, node) in tree.iter().enumerate() {
        //     eprintln!("{i:>2} {:?}", node);
        // }

        Some(Self {
            tree,
            offsets,
            prefix_map: vec![],
            shift,
            bpp,
            l1: 0,
            overlap: 0,
            _tp: std::marker::PhantomData,
        })
    }
}

impl<const B: usize, const N: usize, Tp: Marker + NotCompact> PartitionedSTree<B, N, Tp> {
    pub fn new(vals: &[u32], b: usize) -> Self {
        Self::try_new(vals, b).unwrap()
    }

    /// Partition on the first `b` bits of each key before building the tree.
    /// Any bits beyond the maximum value are skipped.
    /// - uses hugepages.
    /// - uses forward layout.
    /// - uses 'rev' bucket order.
    /// - uses the full array.
    pub fn try_new(vals: &[u32], b: usize) -> Option<Self> {
        let n = vals.len();

        let (shift, parts, max_bucket, height, overlap) = Self::get_part_size(vals, b);

        let layer_sizes;
        let offsets;
        let mut tree;
        let mut l1 = 0;
        assert!(height > 0);
        // All layers are full, for indexing purposes.
        // Unless, L1 is set. Then, the first layer below the root (ie the number of entries in the root) can be smaller.
        layer_sizes = if Tp::MAP {
            let mut layer_sizes = (0..height)
                .map(|h| TreeBase::<B>::layer_size(n, h, height).div_ceil(B))
                .collect_vec();
            if height > 1 {
                layer_sizes[0] = TreeBase::<B>::layer_size(n, 1, height)
                    .div_ceil(B)
                    .div_ceil(B);
            }

            layer_sizes
        } else if !Tp::L1 {
            (0..height).map(|h| (B + 1).pow(h as u32)).collect_vec()
        } else {
            // let normal_layer_sizes = (0..height)
            //     .map(|h| Self::layer_size(max_bucket, h, height))
            //     .collect_vec();
            // eprintln!("normal_layer_sizes={:?}", normal_layer_sizes);
            // let normal_layer_sizes = (0..height)
            //     .map(|h| Self::layer_size(max_bucket, h, height).div_ceil(B))
            //     .collect_vec();
            // eprintln!("normal_layer_sizes={:?}", normal_layer_sizes);

            // Number of nodes in level 1 is number of values in level 0 + 1.
            l1 = if Tp::OL {
                assert_eq!(N, 16);
                overlap.map_or(N + 1, |o| N - o)
            } else {
                TreeBase::<B>::layer_size(max_bucket, 1, height).div_ceil(B)
            };
            eprintln!("size: {n} l1 {l1}");
            (0..height)
                .map(|h| ((B + 1).pow(h as u32) * l1).div_ceil(B + 1))
                .collect_vec()
        };
        eprintln!("layer_sizes={:?}", layer_sizes);

        let n_blocks;
        let mut extra_parts = 0;
        if !Tp::MAP {
            assert!(
                layer_sizes[0] == 1,
                "layer_sizes={:?} has unexpected root size?",
                layer_sizes
            );

            // FIXME: We can be more precise and add only single-node subtrees, instead of l1 copies at a time.
            extra_parts = if l1 == 0 {
                assert_eq!(overlap.unwrap_or(0), 0);
                0
            } else {
                overlap.unwrap_or(0).div_ceil(l1)
            };
            let mut layer_blocks = layer_sizes
                .iter()
                .map(|x| x * (parts + extra_parts))
                .collect_vec();
            if let Some(o) = overlap {
                layer_blocks[0] = (parts * (16 - o) + o).div_ceil(16);
            };
            eprintln!("layer_blocks={:?}", layer_blocks);

            n_blocks = layer_blocks.iter().sum::<usize>();
            eprintln!("n_blocks={}", n_blocks);

            offsets = layer_blocks
                .iter()
                .scan(0, |sum, sz| {
                    let offset = *sum;
                    *sum += sz;
                    Some(offset)
                })
                .collect_vec();
            eprintln!("offsets={:?}", offsets);
        } else {
            n_blocks = layer_sizes.iter().sum::<usize>();
            offsets = layer_sizes
                .iter()
                .scan(0, |sum, sz| {
                    let offset = *sum;
                    *sum += sz;
                    Some(offset)
                })
                .collect_vec();
            eprintln!("offsets={:?}", offsets);
        }

        if n_blocks * size_of::<BTreeNode<N>>() > (32 << 30) {
            // Too much overhead.
            return None;
        }

        tree = vec_on_hugepages::<BTreeNode<N>>(n_blocks)?;

        // First initialize all nodes in the layer with MAX.
        // TODO: Maybe we can omit this and avoid mapping some of the pages of the tree?
        for node in &mut tree {
            node.data.fill(MAX);
        }

        // Initialize the last layer.
        let ol = offsets[height - 1];
        let mut prev_part = 0;
        let mut idx = 0;

        let subtree_size = if height == 1 {
            1
        } else {
            B * (B + 1).pow(height as u32 - 2)
        };
        eprintln!("subtree size {subtree_size}");
        let mut part_size = l1 * subtree_size;
        eprintln!("part size {part_size}");

        if !Tp::OL {
            assert_eq!(overlap, None);
            // FIXME?
            part_size = B * layer_sizes[height - 1];
            // assert_eq!(
            //     part_size,
            //     B * layer_sizes[height - 1],
            //     "l1: {l1} subtree_size {subtree_size}"
            // );
        }

        eprintln!("Build base layer");
        for &val in vals {
            if !Tp::MAP {
                let part = (val >> shift) as usize;

                // For each completed part (typically just 1), append the current val.
                while prev_part < part {
                    prev_part += 1;
                    while idx < prev_part * part_size {
                        // eprintln!("write {val} to {idx}");
                        tree[ol + idx / B].data[idx % B] = val;
                        idx += 1;
                    }
                }
            }

            // eprintln!("write {val} to {idx}");
            tree[ol + idx / B].data[idx % B] = val;
            // If B<N and there is some buffer space in each node,
            // put us also in the last element of the previous node.
            // FIXME: DO WE NEED THIS? HERE AND ELSEWHERE.
            if B < N && idx % B == 0 && idx > 0 {
                tree[ol + idx / B - 1].data[B] = val;
            }
            idx += 1;
        }

        eprintln!("Fill tree");
        // Initialize the inner layers.
        for h in (0..height - 1).rev() {
            let oh = offsets[h];

            if h == 0 {
                if let Some(o) = overlap {
                    // In case of overlap, layer 0 is special.
                    let o0 = offsets[0];
                    // FIXME: (parts+extra_parts)*l1?
                    let range = if Tp::MAP {
                        0..layer_sizes[1] - 1
                    } else {
                        0..parts * l1 + o
                    };
                    for i in range {
                        let j = (i + 1) * subtree_size - 1;
                        // eprintln!("root: copy from {j} to {i}");
                        tree[o0 + i / B].data[i % B] = tree[ol + j / B].data[j % B];
                    }

                    break;
                }
            }

            let l = layer_sizes[h];
            let ll = layer_sizes[height - 1];
            if Tp::MAP {
                for i in 0..B * layer_sizes[h] {
                    let mut k = i / B;
                    let j = i % B;
                    k = k * (B + 1) + j + 1;
                    for _l in h..height - 2 {
                        k *= B + 1;
                    }
                    tree[oh + i / B].data[i % B] = if k * B < n {
                        tree[ol + k - 1].data[B - 1]
                    } else {
                        MAX
                    };
                }
            } else {
                for p in 0..parts + extra_parts {
                    for i in 0..B * layer_sizes[h] {
                        let mut k = i / B;
                        let j = i % B;
                        k = k * (B + 1) + j + 1;
                        for _l in h..height - 2 {
                            k *= B + 1;
                        }
                        tree[oh + l * p + i / B].data[i % B] = if k * B < max_bucket {
                            tree[ol + ll * p + k - 1].data[B - 1]
                        } else {
                            MAX
                        };
                    }
                }
            }
        }
        assert!(offsets.len() > 0);

        eprintln!("Prefix map..");
        let mut prefix_map;
        if !Tp::MAP {
            prefix_map = vec![];
        } else {
            if size_of_val(&tree) + parts * size_of::<u32>() > 4 * n * size_of::<u32>() {
                // Too much overhead.
                return None;
            }

            prefix_map = vec![0; parts];
            let max_idx = layer_sizes[0] * B - B;
            assert!(max_idx < u32::MAX as usize);
            assert_eq!(B, 16);
            // Iterate over layer 0, and find where each prefix starts.
            let mut p = 0;
            let o0 = offsets[0];
            for i in 0..layer_sizes[0] * B {
                let val = tree[o0 + i / B].data[i % B];
                let pi = (val >> shift) as usize;
                while p < pi {
                    p += 1;
                    prefix_map[p] = i.min(max_idx) as u32;
                }
            }
            while p + 1 < parts {
                p += 1;
                prefix_map[p] = max_idx as u32;
            }
            // eprintln!("Prefix map: {prefix_map:?}");
        }

        eprintln!(
            "PartitionedSTree {}: n={} b={} shift={} parts={} height={} layer_sizes={:?} offsets={:?} compact=false max_bucket={max_bucket}",
            type_name::<Tp>(),
            n, b, shift, parts, height, layer_sizes, offsets
        );
        eprintln!("overlap {:?}", overlap);
        eprintln!("subtree_size {}", subtree_size);
        eprintln!("l1 {}", l1);
        // for (i, node) in tree.iter().enumerate() {
        //     eprintln!("{i:>2} {:?}", node.data);
        // }

        Some(Self {
            tree,
            offsets,
            shift,
            prefix_map,
            bpp: 0,
            l1: if Tp::OL {
                // either 16 or 17
                l1.max(16)
            } else {
                l1
            },
            overlap: overlap.unwrap_or(0),
            _tp: std::marker::PhantomData,
        })
    }
}

/// Partitions, full
/// First layer 0 of all parts, then layer 1 of all parts, ...
/// Inefficient, because layers much have their 'full' size and grown by B+1 each level.
impl<const B: usize, const N: usize> PartitionedSTree<B, N, Simple> {
    pub fn search<const P: usize, const PF: bool>(&self, qb: &[u32; P]) -> [u32; P] {
        let offsets = self
            .offsets
            .iter()
            .map(|o| unsafe { self.tree.as_ptr().add(*o) })
            .collect_vec();

        // Initial parts, and prefetch them.
        let o0 = offsets[0];
        let mut k: [usize; P] = qb.map(|q| {
            let k = (q as usize >> self.shift) * 64;
            if PF {
                prefetch_ptr(unsafe { o0.byte_add(k) });
            }
            k
        });
        let q_simd = qb.map(|q| Simd::<u32, 8>::splat(q));

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
}

/// Partitions, compact.
/// Layout: tree for part 1, tree for part 2, ...
/// More efficient because each part is stored compactly.
/// Slightly slower though, because we must explicitly track to which part each query belongs.
impl<const B: usize, const N: usize> PartitionedSTree<B, N, Compact> {
    pub fn search<const P: usize, const PF: bool>(&self, qb: &[u32; P]) -> [u32; P] {
        let offsets = self
            .offsets
            .iter()
            .map(|o| unsafe { self.tree.as_ptr().add(*o) })
            .collect_vec();

        // Initial parts, and prefetch them.
        let o0 = offsets[0];
        let mut k: [usize; P] = [0; P];
        let parts: [usize; P] = qb.map(|q| {
            // byte offset of the part.
            let p = (q as usize >> self.shift) * self.bpp * 64;
            if PF {
                prefetch_ptr(unsafe { o0.byte_add(p) });
            }
            p
        });

        let q_simd = qb.map(|q| Simd::<u32, 8>::splat(q));

        for [o, o2] in offsets.array_windows() {
            for i in 0..P {
                let jump_to = unsafe { *o.byte_add(parts[i] + k[i]) }.find_splat64(q_simd[i]);
                k[i] = k[i] * (B + 1) + jump_to;
                prefetch_ptr(unsafe { o2.byte_add(parts[i] + k[i]) });
            }
        }

        let o = offsets.last().unwrap();
        from_fn(|i| {
            let idx = unsafe { *o.byte_add(parts[i] + k[i]) }.find_splat(q_simd[i]);

            unsafe { (o.byte_add(parts[i] + k[i]) as *const u32).add(idx).read() }
        })
    }
}

/// Partitions, full with level1 compression.
/// First layer 0 of all parts, then layer 1 of all parts, ...
/// The number of children of each level0 node (root) is only `self.l1`, instead of the full `B+1`, saving some memory.
impl<const B: usize, const N: usize> PartitionedSTree<B, N, L1> {
    pub fn search<const P: usize, const PF: bool>(&self, qb: &[u32; P]) -> [u32; P] {
        let offsets = self
            .offsets
            .iter()
            .map(|o| unsafe { self.tree.as_ptr().add(*o) })
            .collect_vec();

        // Initial parts, and prefetch them.
        let o0 = offsets[0];
        let mut k: [usize; P] = qb.map(|q| {
            let k = (q as usize >> self.shift) * 64;
            if PF {
                prefetch_ptr(unsafe { o0.byte_add(k) });
            }
            k
        });
        let q_simd = qb.map(|q| Simd::<u32, 8>::splat(q));

        if let Some([o, o2]) = offsets.array_windows().next() {
            for i in 0..P {
                let jump_to = unsafe { *o.byte_add(k[i]) }.find_splat64(q_simd[i]);
                //            vvvvvvv
                k[i] = k[i] * self.l1 + jump_to;
                prefetch_ptr(unsafe { o2.byte_add(k[i]) });
            }
        }

        for [o, o2] in offsets[1..].array_windows() {
            for i in 0..P {
                let jump_to = unsafe { *o.byte_add(k[i]) }.find_splat64(q_simd[i]);
                //            vvvvvvv
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
}

/// Partitions, with overlapping parts.
/// If overlap is 0, this is close to the L1 case.
/// If overlap is 15, each partition roughly adds a single subtree.
/// If overlap is 16, there is only a single root node.
impl<const B: usize, const N: usize> PartitionedSTree<B, N, Overlapping> {
    pub fn search<const P: usize, const PF: bool>(&self, qb: &[u32; P]) -> [u32; P] {
        let offsets = self
            .offsets
            .iter()
            .map(|o| unsafe { self.tree.as_ptr().add(*o) })
            .collect_vec();

        // Initial parts, and prefetch them.
        let o0 = offsets[0];
        let mut k: [usize; P] = qb.map(|q| {
            let k = (q as usize >> self.shift) * 4 * (16 - self.overlap);
            if PF {
                prefetch_ptr(unsafe { o0.byte_add(k) });
            }
            k
        });
        let q_simd = qb.map(|q| Simd::<u32, 8>::splat(q));

        if let Some([o, o2]) = offsets.array_windows().next() {
            for i in 0..P {
                // First level read is intentionally unaligned.
                let jump_to = unsafe { o.byte_add(k[i]).read_unaligned() }.find_splat64(q_simd[i]);
                //            vvvvvvv
                k[i] = k[i] * self.l1 + jump_to;
                prefetch_ptr(unsafe { o2.byte_add(k[i]) });
            }
        }

        for [o, o2] in offsets[1..].array_windows() {
            for i in 0..P {
                let jump_to = unsafe { *o.byte_add(k[i]) }.find_splat64(q_simd[i]);
                //            vvvvvvv
                k[i] = k[i] * (B + 1) + jump_to;
                prefetch_ptr(unsafe { o2.byte_add(k[i]) });
            }
        }

        let o = offsets.last().unwrap();
        from_fn(|i| {
            // Read can be unaligned when the height of the tree is 1.
            let idx = unsafe { o.byte_add(k[i]).read_unaligned() }.find_splat(q_simd[i]);

            unsafe { (o.byte_add(k[i]) as *const u32).add(idx).read() }
        })
    }
}

impl<const B: usize, const N: usize> PartitionedSTree<B, N, Map> {
    // NOTE: Call this with PF (prefetch) = true, to avoid slow auto-vectorization of the first loop.
    pub fn search<const P: usize, const PF: bool>(&self, qb: &[u32; P]) -> [u32; P] {
        let offsets = self
            .offsets
            .iter()
            .map(|o| unsafe { self.tree.as_ptr().add(*o) })
            .collect_vec();

        // Initial parts, and prefetch them.
        let o0 = offsets[0];
        let mut k: [usize; P] = qb.map(|q| {
            let k =
                4 * unsafe { *self.prefix_map.get_unchecked(q as usize >> self.shift) as usize };
            if PF {
                prefetch_ptr(unsafe { o0.byte_add(k) });
            }
            k
        });
        let q_simd = qb.map(|q| Simd::<u32, 8>::splat(q));

        if let Some([o, o2]) = offsets.array_windows().next() {
            for i in 0..P {
                // First level read is intentionally unaligned.
                let jump_to = unsafe { o.byte_add(k[i]).read_unaligned() }.find_splat64(q_simd[i]);
                //            vvvvvvv
                k[i] = k[i] * 16 + jump_to;
                prefetch_ptr(unsafe { o2.byte_add(k[i]) });
            }
        }

        for [o, o2] in offsets[1..].array_windows() {
            for i in 0..P {
                let jump_to = unsafe { *o.byte_add(k[i]) }.find_splat64(q_simd[i]);
                //            vvvvvvv
                k[i] = k[i] * (B + 1) + jump_to;
                prefetch_ptr(unsafe { o2.byte_add(k[i]) });
            }
        }

        let o = offsets.last().unwrap();
        from_fn(|i| {
            // Read can be unaligned when the height of the tree is 1.
            let idx = unsafe { o.byte_add(k[i]).read_unaligned() }.find_splat(q_simd[i]);

            unsafe { (o.byte_add(k[i]) as *const u32).add(idx).read() }
        })
    }

    pub fn search_interleave_128(&self, qs: &[u32]) -> Vec<u32> {
        /// Prefetching doesn't really help.
        const PF: bool = false;
        match self.offsets.len() {
            1 => self.search_interleave::<128, 1, 128, PF>(qs),
            2 => self.search_interleave::<64, 2, 128, PF>(qs),
            3 => self.search_interleave::<32, 3, 96, PF>(qs),
            4 => self.search_interleave::<32, 4, 128, PF>(qs),
            5 => self.search_interleave::<16, 5, 80, PF>(qs),
            6 => self.search_interleave::<16, 6, 96, PF>(qs),
            7 => self.search_interleave::<16, 7, 112, PF>(qs),
            8 => self.search_interleave::<16, 8, 128, PF>(qs),
            _ => panic!("Unsupported tree height {}", self.offsets.len()),
        }
    }

    pub fn search_interleave<const P: usize, const L: usize, const PL: usize, const PF: bool>(
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
                // first layer has branching factor 16
                for l in (0..L - 1).take(1) {
                    // i>=0, but i is unsigned so we check from the other end.
                    if i < L * P {
                        let jump_to = unsafe { offsets[l].byte_add(k[i]).read_unaligned() }
                            .find_splat64(q_simd[i]);
                        k[i] = k[i] * 16 + jump_to;
                        prefetch_ptr(unsafe { offsets[l + 1].byte_add(k[i]) });
                    }
                    i = i.wrapping_add(1);
                }

                for l in (0..L - 1).skip(1) {
                    if i < L * P {
                        let jump_to = unsafe { *offsets[l].byte_add(k[i]) }.find_splat64(q_simd[i]);
                        k[i] = k[i] * (B + 1) + jump_to;
                        prefetch_ptr(unsafe { offsets[l + 1].byte_add(k[i]) });
                    }
                    i = i.wrapping_add(1);
                }

                if i < L * P {
                    ans[j] = {
                        let idx =
                            unsafe { ol.byte_add(k[i]).read_unaligned() }.find_splat(q_simd[i]);
                        unsafe { (ol.byte_add(k[i]) as *const u32).add(idx).read() }
                    };
                    // Find prefix position of new query.
                    q_simd[i] = Simd::splat(c1[j]);
                    k[i] = 4 * unsafe {
                        *self.prefix_map.get_unchecked(c1[j] as usize >> self.shift) as usize
                    };
                    if PF {
                        prefetch_ptr(unsafe { offsets[0].byte_add(k[i]) });
                    }
                    j += 1;
                }

                i = i.wrapping_add(1);

                assert!(i < P);
            }

            // Middle

            loop {
                // first layer has branching factor 16
                for l in (0..L - 1).take(1) {
                    let jump_to = unsafe { offsets[l].byte_add(k[i]).read_unaligned() }
                        .find_splat64(q_simd[i]);
                    k[i] = k[i] * 16 + jump_to;
                    prefetch_ptr(unsafe { offsets[l + 1].byte_add(k[i]) });
                    i += 1;
                }

                for l in (0..L - 1).skip(1) {
                    let jump_to = unsafe { *offsets[l].byte_add(k[i]) }.find_splat64(q_simd[i]);
                    k[i] = k[i] * (B + 1) + jump_to;
                    prefetch_ptr(unsafe { offsets[l + 1].byte_add(k[i]) });
                    i += 1;
                }

                ans[j] = {
                    let idx = unsafe { ol.byte_add(k[i]).read_unaligned() }.find_splat(q_simd[i]);
                    unsafe { (ol.byte_add(k[i]) as *const u32).add(idx).read() }
                };
                // Find prefix position of new query.
                q_simd[i] = Simd::splat(c1[j]);
                k[i] = 4 * unsafe {
                    *self.prefix_map.get_unchecked(c1[j] as usize >> self.shift) as usize
                };
                if PF {
                    prefetch_ptr(unsafe { offsets[0].byte_add(k[i]) });
                }

                i += 1;
                j += 1;

                if i > PL - L {
                    break;
                }
            }

            // Last incomplete iteration.
            {
                // first layer has branching factor 16
                for l in (0..L - 1).take(1) {
                    if i < L * P {
                        let jump_to = unsafe { offsets[l].byte_add(k[i]).read_unaligned() }
                            .find_splat64(q_simd[i]);
                        k[i] = k[i] * 16 + jump_to;
                        prefetch_ptr(unsafe { offsets[l + 1].byte_add(k[i]) });
                    }
                    i += 1;
                }

                for l in (0..L - 1).skip(1) {
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
}
