use clap::Parser;
use log::{debug, info};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::{iter::repeat, path::PathBuf};
use suffix_array_searching::{
    experiments,
    sa_search::{self, *},
    util::*,
};

#[derive(Parser)]
struct Args {
    #[clap(short)]
    n: Option<usize>,
    #[clap(short, default_value_t = 10000000)]
    q: usize,
    #[clap(short, default_value_t = 20)]
    p: usize,

    #[clap(long)]
    path: Option<PathBuf>,

    #[clap(short, default_value_t = 0, action = clap::ArgAction::Count,)]
    verbose: u8,
}

fn main() {
    let args = Args::parse();

    stderrlog::new()
        .verbosity(4 + args.verbose as usize)
        .show_level(false)
        .init()
        .unwrap();

    // Get a fixed seeded rng.
    let rng = &mut ChaCha8Rng::seed_from_u64(31415);

    let mut t = if let Some(path) = args.path {
        info!("Reading {path:?}..");
        let mut t = read_fasta_file(&path);
        info!("Length {}", t.len());
        if let Some(n) = args.n {
            if n < t.len() {
                t.resize(n, 0);
            }
            info!("Cropped to {n}");
        }
        t
    } else {
        debug!("gen string..");
        random_string(args.n.unwrap_or(100_000_000), rng)
    };

    // Padding.
    t.extend(repeat(0).take(200));
    let t = &t[..t.len() - 200];

    debug!("gen queries..");
    let qs = random_queries(t, args.q, rng);

    // Run experiments from `experiments`.

    info!("build SA..");
    let start = std::time::Instant::now();
    let sa = experiments::SA::build(t);
    info!("build SA: {:.2?}", start.elapsed());

    info!("start bench..");

    eprintln!(
        "{:<20}  {:>8} {:>6} {:>6} {:>5}",
        "method", "total", "/query", "/loop", "#loops"
    );

    experiments::bench(&sa, &qs, "binary_basic", experiments::binary_search as _);
    experiments::bench(
        &sa,
        &qs,
        "binary_branchless",
        experiments::branchless_bin_search as _,
    );

    // Run experiments from `sa_search`.

    info!("build SA..");
    let start = std::time::Instant::now();
    let sa = sa_search::SaNaive::build(t);
    info!("build SA: {:.2?}", start.elapsed());

    info!("start bench..");

    sa_search::bench(&sa, &qs, "binary_c", binary_search_cmp as _);
    sa_search::bench(&sa, &qs, "branchy", branchy_search as _);
    sa_search::bench(&sa, &qs, "branchfree", branchfree_search as _);
    sa_search::bench(&sa, &qs, "interpolation", interpolation_search::<16> as _);

    bench_batch(&sa, &qs, "batch_4_c", binary_search_batch_c::<4> as _);
    bench_batch(&sa, &qs, "batch_8_c", binary_search_batch_c::<8> as _);
    bench_batch(&sa, &qs, "batch_16_c", binary_search_batch_c::<16> as _);
    bench_batch(&sa, &qs, "batch_32_c", binary_search_batch_c::<32> as _);
    bench_batch(&sa, &qs, "batch_64_c", binary_search_batch_c::<64> as _);

    bench_batch(&sa, &qs, "batch_4", binary_search_batch::<4> as _);
    bench_batch(&sa, &qs, "batch_8", binary_search_batch::<8> as _);
    bench_batch(&sa, &qs, "batch_16", binary_search_batch::<16> as _);
    bench_batch(&sa, &qs, "batch_32", binary_search_batch::<32> as _);
    bench_batch(&sa, &qs, "batch_64", binary_search_batch::<64> as _);

    bench_batch(&sa, &qs, "batch_16", binary_search_batch::<16> as _);
    bench_batch(&sa, &qs, "batch_16_c", binary_search_batch_c::<16> as _);
    bench_batch(
        &sa,
        &qs,
        "branchfree_16",
        branchfree_search_batch::<16> as _,
    );
    bench_batch(&sa, &qs, "branchfree_16_c", branchfree_batch_cmp::<16> as _);
}
