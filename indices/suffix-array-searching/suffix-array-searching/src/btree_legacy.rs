#![allow(unused)]

use std::{arch::x86_64::_mm_prefetch, ops::Range};
pub struct Augment<'a> {
    text: &'a [u8],
    sa: &'a SA,
    /// Eytzinger layout tree of subset of the SA.
    tree: Vec<Node>,
    tree_levels: usize,
}

const CACHE_LINE_SIZE: usize = 64;
const PREFIX: usize = 8;

pub struct Node {
    // lcp: usize,
    idx: usize,
    prefix: [u8; PREFIX],
}

impl<'a> Augment<'a> {
    pub fn new(text: &'a [u8], sa: &'a SA) -> Self {
        let mut tree = vec![];
        // Push a default node so we start the Eytzinger layout at index 1.
        tree.push(Node {
            idx: 0,
            // lcp: 0,
            prefix: [0; PREFIX],
        });

        let n = text.len();
        assert_eq!(n, sa.len());
        let total_levels = n.next_power_of_two().ilog2() as usize;
        let tree_levels = total_levels - 7;
        eprintln!("Total levels: {:?}", total_levels);
        eprintln!("Tree levels:  {:?}", tree_levels);

        for i in 1..(1 << tree_levels) {
            let range = Self::get_node_range(&tree, i, n);
            // TODO: Move `mid` around to make sure the prefix comparison gives the right result.
            let mid = (range.start + range.end) / 2;
            let prefix = *text[sa[mid]..].split_first_chunk().unwrap().0;
            tree.push(Node {
                idx: mid,
                // lcp: 0,
                prefix,
            });
        }

        Self {
            text,
            sa,
            tree,
            tree_levels,
        }
    }

    fn size(&self) -> usize {
        std::mem::size_of_val(self.tree.as_slice())
    }

    /// ```txt
    ///       3
    ///     /   \
    ///    6     7
    ///   / \   / \
    ///  12 13 14 15
    /// 1100  1110
    ///    1101  1111
    /// ```
    ///
    /// The range of 13 is from 6=110 to 3=11.
    /// The end of the range is given by the parent of the most recent left-move.
    /// Left-moves correspond to 0, so we remove everything up to and including the last 0.
    fn get_node_range(tree: &Vec<Node>, i: usize, n: usize) -> Range<usize> {
        let get_idx = |i: usize| tree[i >> (1 + i.trailing_ones())].idx;
        let idx_start = get_idx(i - 1);
        let mut idx_end = get_idx(i);
        if idx_end == 0 {
            idx_end = n;
        }
        let range = idx_start..idx_end;
        range
    }

    pub fn query_bs(&self, q: &[u8]) -> usize {
        self.sa
            .binary_search_by(|&i| {
                let suffix = &self.text[i..];
                suffix.cmp(q)
            })
            .map_or_else(|x| x, |x| x)
    }

    pub fn query_bs_range(&self, q: &[u8], range: Range<usize>) -> usize {
        range.start
            + self.sa[range]
                .binary_search_by(|&i| {
                    let suffix = &self.text[i..];
                    suffix.cmp(q)
                })
                .map_or_else(|x| x, |x| x)
    }

    pub fn query_btree(&self, q: &[u8]) -> usize {
        let mut i = 1;
        let mut level = 0;

        const LOOKAHEAD: usize = CACHE_LINE_SIZE / std::mem::size_of::<Node>();
        const LOOKAHEAD_LEVELS: usize = LOOKAHEAD.trailing_zeros() as usize;

        for level in 0..self.tree_levels {
            if level < self.tree_levels - LOOKAHEAD_LEVELS {
                let prefix_i = i * LOOKAHEAD;
                unsafe {
                    _mm_prefetch::<{ std::arch::x86_64::_MM_HINT_T0 }>(
                        self.tree.as_ptr().offset(prefix_i as isize) as *const _,
                    )
                };
            }
            let node = &self.tree[i];
            let q_part = *q[0..].split_first_chunk().unwrap().0;
            i = if q_part < node.prefix {
                2 * i
            } else {
                2 * i + 1
            };
        }
        self.query_bs_range(q, Self::get_node_range(&self.tree, i, self.text.len()))
    }
}

// TODO
// - prefix lookup
// - plain binary search for the end
// - bitpacking

#[cfg(test)]
mod test {
    use super::*;

    fn time_queries(aug: &Augment, queries: &[Vec<u8>]) {
        // time("BS   ", || {
        //     for q in queries {
        //         let _ = aug.query_bs(q);
        //     }
        // });
        time("BTREE", || {
            for q in queries {
                let _ = aug.query_btree(q);
            }
        });
    }

    #[test]
    fn query_random() {
        // 3. build augmented structure
        // 4. query random strings

        let (text, sa) = read();
        let aug = time("Augmenting", || Augment::new(&text, &sa));
        eprintln!("Size: {:?}", aug.size());

        let query_len = 31;
        let mut rng = rand::thread_rng();
        let num_queries = 1000000;
        let mut queries = vec![];
        for _ in 0..num_queries {
            let q = (0..query_len)
                .map(|_| b"ACTG"[rand::random::<u8>() as usize % 4])
                .collect::<Vec<u8>>();
            queries.push(q);
        }

        time_queries(&aug, &queries);
    }

    #[test]
    fn query_substrings() {
        // 3. build augmented structure
        // 4. query random strings

        let (text, sa) = read();
        let aug = time("Augmenting", || Augment::new(&text, &sa));
        eprintln!("Size: {:?}", aug.size());

        let query_len = 31;
        let mut rng = rand::thread_rng();
        let num_queries = 1000000;
        let mut queries = vec![];
        for _ in 0..num_queries {
            let q =
                text[rand::random::<usize>() % (text.len() - query_len)..][..query_len].to_vec();
            queries.push(q);
        }

        time_queries(&aug, &queries);
    }
}
