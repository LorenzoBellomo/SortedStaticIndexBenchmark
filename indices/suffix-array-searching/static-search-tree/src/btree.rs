use crate::{
    node::{BTreeNode, MAX},
    SearchIndex,
};
use std::fmt::Debug;

#[derive(Debug)]
pub struct BTree<const B: usize, const N: usize> {
    tree: Vec<BTreeNode<N>>,
    pub cnt: usize,
}

pub type BTree16 = BTree<16, 16>;

impl<const B: usize, const N: usize> BTree<B, N> {
    fn go_to(&self, k: usize, j: usize) -> usize {
        k * (B + 1) + j + 1
    }

    fn get(&self, b: usize, i: usize) -> u32 {
        unsafe { *self.tree.get_unchecked(b).data.get_unchecked(i) }
    }
}

impl<const B: usize, const N: usize> SearchIndex for BTree<B, N> {
    fn new(vals: &[u32]) -> Self {
        // always have at least one node
        let n_blocks = vals.len().div_ceil(B);
        let mut btree = Self {
            tree: vec![BTreeNode::default(); n_blocks],
            cnt: 0,
        };
        let mut i: usize = 0;
        let k = 0;
        // SIMD operations fail for values outside this range.
        for &v in vals {
            assert!(v <= MAX);
        }

        // recursive function to create a btree
        // a is the original sorted array
        // k is the number of the block
        // i is the position in the original array
        fn recurse<const B: usize, const N: usize>(
            btree: &mut BTree<B, N>,
            a: &[u32],
            i: &mut usize,
            k: usize,
        ) {
            let num_blocks = (a.len() + B - 1) / B;
            if k < num_blocks {
                for j in 0..B {
                    recurse(btree, a, i, btree.go_to(k, j));
                    btree.tree[k].data[j] = a.get(*i).unwrap_or(&MAX).clone();
                    *i += 1;
                }
                recurse(btree, a, i, btree.go_to(k, B));
            }
        }

        recurse(&mut btree, &vals, &mut i, k);
        btree
    }

    fn layers(&self) -> usize {
        todo!()
    }

    fn size(&self) -> usize {
        std::mem::size_of_val(self.tree.as_slice())
    }
}

impl<const B: usize, const N: usize> BTree<B, N> {
    // basic searching with no vectorized magic inside the nodes
    pub fn search(&self, q: u32) -> u32 {
        // completely naive
        let mut k = 0;
        let btree_blocks = self.tree.len();
        let mut ans = MAX;
        while k < btree_blocks {
            let mut jump_to = 0;
            for j in 0..B {
                let compare_to = self.get(k, j);
                if q <= compare_to {
                    break;
                }
                jump_to += 1;
            }
            if jump_to < B {
                ans = self.get(k, jump_to);
            }
            k = self.go_to(k, jump_to);
        }
        ans
    }

    pub fn search_loop(&self, q: u32) -> u32 {
        // completely naive
        let mut k = 0;
        let btree_blocks = self.tree.len();
        let mut ans = MAX;
        while k < btree_blocks {
            let mut jump_to = 0;
            for j in 0..B {
                let compare_to = self.get(k, j);
                jump_to += (q > compare_to) as usize;
            }
            if jump_to < B {
                ans = self.get(k, jump_to);
            }
            k = self.go_to(k, jump_to);
        }
        ans
    }

    pub fn search_simd(&self, q: u32) -> u32 {
        // completely naive
        let mut k = 0;
        let btree_blocks = self.tree.len();
        let mut ans = MAX;
        while k < btree_blocks {
            let jump_to = self.tree[k].find(q);
            if jump_to < B {
                ans = self.get(k, jump_to);
            }
            k = self.go_to(k, jump_to);
        }
        ans
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{binary_search::SortedVec, SearchIndex};

    #[test]
    fn test_b_tree_k_2() {
        let vals = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let btree = BTree::<2, 2>::new(&vals);
        println!("{:?}", btree);
    }

    #[test]
    fn test_b_tree_k_3() {
        let vals = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        // let correct_output = vec![4, 8, 12, 1, 2, 3, 5, 6, 7, 9, 10, 11, 13, 14, 15];
        let computed_out = BTree::<3, 3>::new(&vals);
        println!("{:?}", computed_out);
    }

    #[test]
    fn test_btree_search_bottom_layer() {
        let mut vals: Vec<u32> = (1..2000).collect();
        vals.push(MAX);
        let q = 452;
        let btree = BTree::<16, 16>::new(&vals);
        let btree_res = btree.search(q);

        let binsearch_res = SortedVec::new(&vals).binary_search(q);
        println!("{btree_res}, {binsearch_res}");
        assert!(btree_res == binsearch_res);
    }

    #[test]
    fn test_btree_search_top_node() {
        let mut vals: Vec<u32> = (1..2000).collect();
        vals.push(MAX);
        let q = 289;
        let btree = BTree::<16, 16>::new(&vals);
        let btree_res = btree.search(q);

        let binsearch_res = SortedVec::new(&vals).binary_search(q);
        println!("{btree_res}, {binsearch_res}");
        assert!(btree_res == binsearch_res);
    }

    #[test]
    fn test_simd_cmp() {
        let mut vals: Vec<u32> = (1..16).collect();
        vals.push(MAX);
        let btree = BTree::<16, 16>::new(&vals);
        let idx = btree.tree[0].find(1);
        println!("{}", idx);
        assert!(idx == 0);
    }

    #[test]
    fn test_btree_simd_bottom_layer() {
        let mut vals: Vec<u32> = (1..2000).collect();
        vals.push(MAX);
        let q = 452;
        let btree = BTree::<16, 16>::new(&vals);
        let btree_res = btree.search_simd(q);

        let binsearch_res = SortedVec::new(&vals).binary_search(q);
        println!("{btree_res}, {binsearch_res}");
        assert!(btree_res == binsearch_res);
    }

    #[test]
    fn test_btree_simd_top_node() {
        let mut vals: Vec<u32> = (1..2000).collect();
        vals.push(MAX);
        let q = 289;
        let btree = BTree::<16, 16>::new(&vals);
        let btree_res = btree.search_simd(q);

        let binsearch_res = SortedVec::new(&vals).binary_search(q);
        println!("{btree_res}, {binsearch_res}");
        assert!(btree_res == binsearch_res);
    }
}
