#![cfg(test)]

use itertools::Itertools;

use crate::binary_search::SortedVec;
use crate::btree::{BTree, BTree16};
use crate::eytzinger::Eytzinger;
use crate::node::BTreeNode;
use crate::partitioned_s_tree::{
    PartitionedSTree16, PartitionedSTree16C, PartitionedSTree16L, PartitionedSTree16M,
    PartitionedSTree16O,
};
use crate::s_tree::{STree, STree15, STree16};
use crate::SearchIndex;
use crate::{batched, full, util::*, SearchScheme};
use std::any::type_name;
use std::iter::zip;

pub struct SearchSchemes {
    bs: Vec<&'static dyn SearchScheme<SortedVec>>,
    eyt: Vec<&'static dyn SearchScheme<Eytzinger>>,
    bt: Vec<&'static dyn SearchScheme<BTree16>>,
    bp: Vec<&'static dyn SearchScheme<STree16>>,
    bp15: Vec<&'static dyn SearchScheme<STree15>>,
    psp: Vec<&'static dyn SearchScheme<PartitionedSTree16>>,
    pspc: Vec<&'static dyn SearchScheme<PartitionedSTree16C>>,
    pspl: Vec<&'static dyn SearchScheme<PartitionedSTree16L>>,
    pspo: Vec<&'static dyn SearchScheme<PartitionedSTree16O>>,
    pspm: Vec<&'static dyn SearchScheme<PartitionedSTree16M>>,
}

fn get_search_schemes() -> SearchSchemes {
    let bs = vec![
        &SortedVec::binary_search as &dyn SearchScheme<_>,
        &SortedVec::binary_search_std,
        &SortedVec::binary_search_branchless,
        &SortedVec::binary_search_branchless_prefetch,
        &SortedVec::binary_search_branchless_cmov,
        &SortedVec::interpolation_search,
    ];
    let eyt = vec![
        &Eytzinger::search as &dyn SearchScheme<_>,
        &Eytzinger::search_branchless,
        &Eytzinger::search_prefetch::<2>,
        &Eytzinger::search_prefetch::<3>,
        &Eytzinger::search_prefetch::<4>,
        &Eytzinger::search_branchless_prefetch::<2>,
        &Eytzinger::search_branchless_prefetch::<3>,
        &Eytzinger::search_branchless_prefetch::<4>,
    ];
    let bt = vec![
        &BTree::search as &dyn SearchScheme<_>,
        &BTree::search_loop,
        &BTree::search_simd,
    ];
    let bp = const {
        [
            &STree::search as &dyn SearchScheme<_>,
            &STree::search_with_find(BTreeNode::find_split) as _,
            &STree::search_with_find(BTreeNode::find_ctz_signed) as _,
            &STree::search_with_find(BTreeNode::find_popcnt_portable) as _,
            &batched(STree::batch::<4>),
            &batched(STree::batch::<8>),
            &batched(STree::batch::<16>),
            &batched(STree::batch::<32>),
            &batched(STree::batch::<64>),
            &batched(STree::batch::<128>),
            &batched(STree::batch_prefetch::<128>),
            &batched(STree::batch_splat::<128>),
            &batched(STree::batch_ptr::<128>),
            &batched(STree::batch_byte_ptr::<128>),
            &batched(STree::batch_final::<128>),
            &batched(STree::batch_skip_prefetch::<128, 1>),
            &batched(STree::batch_skip_prefetch::<128, 2>),
            // &full(STree::batch_interleave_half::<64>),
            &full(STree::batch_interleave_last::<64, 1>),
            &full(STree::batch_interleave_last::<64, 2>),
            &full(STree::batch_interleave_last::<64, 3>),
            &full(STree::batch_interleave_last::<64, 4>),
            &full(STree16::batch_interleave_all::<128, 1, 128>),
            &full(STree16::batch_interleave_all::<64, 2, 128>),
            &full(STree16::batch_interleave_all::<32, 3, 96>),
            &full(STree16::batch_interleave_all::<32, 4, 128>),
            &full(STree16::batch_interleave_all::<16, 5, 80>),
            &full(STree16::batch_interleave_all::<16, 6, 96>),
            &full(STree16::batch_interleave_all::<16, 7, 112>),
            &full(STree16::batch_interleave_all::<16, 8, 128>),
            &full(STree16::batch_interleave_all_128),
        ]
    }
    .to_vec();

    let bp15 = const {
        [
            &STree::search as &dyn SearchScheme<_>,
            &batched(STree::batch::<128>),
            &batched(STree::batch_prefetch::<128>),
            &batched(STree::batch_splat::<128>),
            &batched(STree::batch_byte_ptr::<128>),
            &batched(STree::batch_final::<128>),
            &batched(STree::batch_skip_prefetch::<128, 1>),
            &batched(STree::batch_skip_prefetch::<128, 2>),
            // &full(STree::batch_interleave_half::<64>),
            &full(STree::batch_interleave_all_128),
        ]
    }
    .to_vec();

    let psp = const { [&batched(PartitionedSTree16::search::<128, true>) as &dyn SearchScheme<_>] }
        .to_vec();
    let pspc =
        const { [&batched(PartitionedSTree16C::search::<128, true>) as &dyn SearchScheme<_>] }
            .to_vec();
    let pspl =
        const { [&batched(PartitionedSTree16L::search::<128, true>) as &dyn SearchScheme<_>] }
            .to_vec();
    let pspo =
        const { [&batched(PartitionedSTree16O::search::<128, true>) as &dyn SearchScheme<_>] }
            .to_vec();
    let pspm = const {
        [
            &batched(PartitionedSTree16M::search::<128, true>) as &dyn SearchScheme<_>,
            &full(PartitionedSTree16M::search_interleave_128) as &dyn SearchScheme<_>,
        ]
    }
    .to_vec();

    SearchSchemes {
        bs,
        eyt,
        bt,
        bp,
        bp15,
        psp,
        pspc,
        pspl,
        pspo,
        pspm,
    }
}

#[test]
fn test_search() {
    let fs = get_search_schemes();

    const TEST_START_POW2: usize = 6;
    const TEST_END_POW2: usize = 26;
    const TEST_QUERIES: usize = 1000;

    let sizes = (TEST_START_POW2..=TEST_END_POW2)
        .map(|p| 1 << p)
        .flat_map(|x| [x, x * 5 / 4, x * 6 / 4, x * 7 / 4])
        .collect_vec();

    for size in sizes {
        let vals = gen_vals(size, true);
        eprintln!("LEN: {}", vals.len());
        let qs = &gen_queries(TEST_QUERIES.next_multiple_of(128));

        let results = &mut vec![];

        // Helper to extract type `I` and build the index.
        fn map<I: SearchIndex>(
            schemes: &Vec<&dyn SearchScheme<I>>,
            vals: &[u32],
            qs: &[u32],
            results: &mut Vec<u32>,
        ) {
            eprintln!("Building index for {:?}", type_name::<I>());
            map_idx(schemes, &I::new(vals), qs, results);
        }

        fn map_idx<I>(
            schemes: &Vec<&(dyn SearchScheme<I>)>,
            index: &I,
            qs: &[u32],
            results: &mut Vec<u32>,
        ) {
            for &scheme in schemes {
                eprintln!("Testing scheme {:?}", scheme.name());
                let new_results = scheme.query(index, qs);
                if new_results.is_empty() {
                    continue;
                }
                if results.is_empty() {
                    *results = new_results;
                } else {
                    if *results != new_results {
                        for (i, (&x, &y)) in zip(results.iter(), new_results.iter()).enumerate() {
                            if x != y {
                                eprintln!("{i}: Expected {x:>10} got {y:>10}");
                            }
                        }
                        assert_eq!(results, &new_results);
                    }
                }
            }
        }

        map(&fs.bs, &vals, qs, results);
        map(&fs.eyt, &vals, qs, results);
        map(&fs.bt, &vals, qs, results);
        map(&fs.bp, &vals, qs, results);
        map(&fs.bp15, &vals, qs, results);
        eprintln!(
            "Building index for {:?} (true, false, false)",
            type_name::<STree16>()
        );
        map_idx(
            &fs.bp,
            &STree16::new_params(&vals, true, false, false),
            qs,
            results,
        );
        eprintln!(
            "Building index for {:?} (true, false, true)",
            type_name::<STree16>()
        );
        map_idx(
            &fs.bp,
            &STree16::new_params(&vals, true, false, true),
            qs,
            results,
        );

        eprintln!("PARTS");
        map_idx(&fs.psp, &PartitionedSTree16::new(&vals, 0), qs, results);
        map_idx(&fs.psp, &PartitionedSTree16::new(&vals, 4), qs, results);
        map_idx(&fs.psp, &PartitionedSTree16::new(&vals, 8), qs, results);
        map_idx(&fs.psp, &PartitionedSTree16::new(&vals, 16), qs, results);
        map_idx(&fs.psp, &PartitionedSTree16::new(&vals, 20), qs, results);
        eprintln!("PARTS COMPACT");
        map_idx(&fs.pspc, &PartitionedSTree16C::new(&vals, 0), qs, results);
        map_idx(&fs.pspc, &PartitionedSTree16C::new(&vals, 4), qs, results);
        map_idx(&fs.pspc, &PartitionedSTree16C::new(&vals, 8), qs, results);
        map_idx(&fs.pspc, &PartitionedSTree16C::new(&vals, 16), qs, results);
        map_idx(&fs.pspc, &PartitionedSTree16C::new(&vals, 20), qs, results);

        eprintln!("PARTS L1");
        map_idx(&fs.pspl, &PartitionedSTree16L::new(&vals, 0), qs, results);
        map_idx(&fs.pspl, &PartitionedSTree16L::new(&vals, 4), qs, results);
        map_idx(&fs.pspl, &PartitionedSTree16L::new(&vals, 8), qs, results);
        map_idx(&fs.pspl, &PartitionedSTree16L::new(&vals, 16), qs, results);
        map_idx(&fs.pspl, &PartitionedSTree16L::new(&vals, 20), qs, results);

        eprintln!("PARTS OVERLAPPING");
        map_idx(&fs.pspo, &PartitionedSTree16O::new(&vals, 0), qs, results);
        map_idx(&fs.pspo, &PartitionedSTree16O::new(&vals, 4), qs, results);
        map_idx(&fs.pspo, &PartitionedSTree16O::new(&vals, 8), qs, results);
        map_idx(&fs.pspo, &PartitionedSTree16O::new(&vals, 16), qs, results);
        map_idx(&fs.pspo, &PartitionedSTree16O::new(&vals, 20), qs, results);

        eprintln!("PARTS MAP");
        map_idx(&fs.pspm, &PartitionedSTree16M::new(&vals, 0), qs, results);
        map_idx(&fs.pspm, &PartitionedSTree16M::new(&vals, 4), qs, results);
        map_idx(&fs.pspm, &PartitionedSTree16M::new(&vals, 8), qs, results);
        map_idx(&fs.pspm, &PartitionedSTree16M::new(&vals, 16), qs, results);
        map_idx(&fs.pspm, &PartitionedSTree16M::new(&vals, 20), qs, results);
    }
}
