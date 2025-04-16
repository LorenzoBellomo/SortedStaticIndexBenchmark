use s_tree::{STree, STree16};
use static_search_tree::*;
use util::*;

fn main() {
    const TEST_START_POW2: usize = 31;
    const TEST_END_POW2: usize = 31;
    const TEST_QUERIES: usize = 10000000;

    eprintln!("Gen queries..");
    let queries = &gen_queries(TEST_QUERIES);
    eprintln!("Gen queries DONE");

    for pow2 in TEST_START_POW2..TEST_END_POW2 + 1 {
        eprintln!("Testing size: {}", pow2);
        let size = 2usize.pow(pow2 as u32);
        eprintln!("Gen vals..");
        let vals = gen_vals(size, true);
        eprintln!("Gen vals DONE");

        eprintln!("Building B+Tree..");
        let bp = &mut STree16::new_params(&vals, true, true, false);
        eprintln!("Building B+Tree DONE");

        for _ in 0..1000 {
            let scheme = &full(STree::batch_interleave_all_128);
            bench_scheme(bp, scheme, queries);
        }
    }
}
