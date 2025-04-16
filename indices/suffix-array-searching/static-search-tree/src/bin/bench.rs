use clap::Parser;
use itertools::Itertools;
use rdst::RadixSort;
use static_search_tree::{
    batched,
    binary_search::SortedVec,
    eytzinger::Eytzinger,
    full,
    node::BTreeNode,
    partitioned_s_tree::{
        PartitionedSTree16, PartitionedSTree16C, PartitionedSTree16L, PartitionedSTree16M,
        PartitionedSTree16O,
    },
    s_tree::{STree15, STree16},
    util::{gen_positive_queries, gen_queries, gen_vals},
    SearchIndex, SearchScheme,
};
use std::{
    hint::black_box,
    path::Path,
    sync::LazyLock,
    time::{Duration, Instant},
};
use suffix_array_searching::util::read_human_genome;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    from: Option<usize>,
    #[clap(short, long)]
    to: Option<usize>,
    #[clap(long)]
    release: bool,
    #[clap(short, long)]
    dense: bool,
    #[clap(short, long)]
    queries: Option<usize>,
    #[clap(long)]
    runs: Option<usize>,
    #[clap(long)]
    human: bool,
    #[clap(long)]
    range: bool,
    #[clap(long)]
    positive: bool,
}

static ARGS: LazyLock<Args> = LazyLock::new(|| Args::parse());

fn main() {
    let mut results = vec![];

    let runs = ARGS.runs.unwrap_or(if ARGS.release { 3 } else { 1 });
    let sizes = sizes();
    for run in 0..runs {
        // Setup

        let mut vals = if !ARGS.human {
            gen_vals(*sizes.last().unwrap(), false)
        } else {
            let k = 16;
            let seq = read_human_genome();

            let mut key = 0;
            for i in 0..k - 1 {
                key = key << 2 | seq[i] as usize;
            }
            let mut vals = vec![];
            for i in k - 1..seq.len().min(sizes.last().unwrap() + k - 1) {
                key = key << 2 | seq[i] as usize;
                key &= (1 << (2 * k)) - 1;
                vals.push(key as u32 & i32::MAX as u32);
            }
            vals[0] = i32::MAX as u32;
            vals
        };

        let queries = ARGS.queries.unwrap_or(1_000_000).next_multiple_of(256 * 3);
        let qs = if ARGS.positive {
            &gen_positive_queries(queries, &vals)
        } else {
            &gen_queries(queries)
        };
        let range_queries = qs.iter().flat_map(|&q| [q, q + 1]).collect_vec();

        for &size in &sizes {
            let len = size / std::mem::size_of::<u32>();
            let vals = &mut vals[..len];
            vals.radix_sort_unstable();

            if ARGS.range {
                let exps = [
                    &batched(STree16::batch_final::<128>) as &dyn SearchScheme<_>,
                    &full(STree16::batch_interleave_all_128),
                ];
                let index = STree16::new_params(vals, true, false, false);
                run_exps(&mut results, size, &index, qs, run, &exps, "single");

                run_exps(
                    &mut results,
                    size,
                    &index,
                    &range_queries,
                    run,
                    &exps,
                    "range",
                );
                continue;
            }

            let bs = (4..=20).step_by(4).collect_vec();

            // SECTION 1.3: Binary search & Eytzinger
            run_exps(
                &mut results,
                size,
                &SortedVec::new(vals),
                qs,
                run,
                &[&SortedVec::binary_search_std],
                "",
            );

            run_exps(
                &mut results,
                size,
                &SortedVec::new(vals),
                qs,
                run,
                &[&SortedVec::binary_search],
                "",
            );

            run_exps(
                &mut results,
                size,
                &SortedVec::new(vals),
                qs,
                run,
                &[&SortedVec::binary_search_branchless],
                "",
            );

            run_exps(
                &mut results,
                size,
                &SortedVec::new(vals),
                qs,
                run,
                &[&SortedVec::binary_search_branchless_prefetch],
                "",
            );

            run_exps(
                &mut results,
                size,
                &SortedVec::new(vals),
                qs,
                run,
                &[&batched(
                    SortedVec::batch_impl_binary_search_branchless::<16>,
                )],
                "",
            );

            run_exps(
                &mut results,
                size,
                &SortedVec::new(vals),
                qs,
                run,
                &[&batched(
                    SortedVec::batch_impl_binary_search_branchless_prefetch::<16>,
                )],
                "",
            );

            // run_exps(
            //     &mut results,
            //     size,
            //     &Eytzinger::new_no_hugepages(vals),
            //     qs,
            //     run,
            //     &[&Eytzinger::search_prefetch::<4>],
            //     " No hugepages",
            // );

            // run_exps(
            //     &mut results,
            //     size,
            //     &Eytzinger::new(vals),
            //     qs,
            //     run,
            //     &[&Eytzinger::search_prefetch::<4>],
            //     "",
            // );

            // run_exps(
            //     &mut results,
            //     size,
            //     &SortedVec::new(vals),
            //     qs,
            //     run,
            //     &[
            //         &batched(SortedVec::batch_impl_binary_search_std::<2>),
            //         &batched(SortedVec::batch_impl_binary_search_std::<4>),
            //         &batched(SortedVec::batch_impl_binary_search_std::<8>),
            //         &batched(SortedVec::batch_impl_binary_search_std::<16>),
            //         &batched(SortedVec::batch_impl_binary_search_std::<32>),
            //     ],
            //     "batched_binsearch",
            // );

            run_exps(
                &mut results,
                size,
                &SortedVec::new(vals),
                qs,
                run,
                &[
                    &batched(SortedVec::batch_impl_binary_search_branchless_prefetch::<2>),
                    &batched(SortedVec::batch_impl_binary_search_branchless_prefetch::<4>),
                    &batched(SortedVec::batch_impl_binary_search_branchless_prefetch::<8>),
                    &batched(SortedVec::batch_impl_binary_search_branchless_prefetch::<16>),
                    &batched(SortedVec::batch_impl_binary_search_branchless_prefetch::<32>),
                    &batched(SortedVec::batch_impl_binary_search_branchless_prefetch::<64>),
                    &batched(SortedVec::batch_impl_binary_search_branchless_prefetch::<128>),
                ],
                "batched_binsearch",
            );

            // run_exps(
            //     &mut results,
            //     size,
            //     &Eytzinger::new(vals),
            //     qs,
            //     run,
            //     &[&batched(Eytzinger::batch_impl::<16>)],
            //     "batched_eytzinger",
            // );

            //     run_exps(
            //         &mut results,
            //         size,
            //         &STree16::new(vals),
            //         qs,
            //         run,
            //         &[
            //             // SECTION 2: Optimizing find
            //             &STree16::search_with_find(BTreeNode::find_linear) as _,
            //             &STree16::search_with_find(BTreeNode::find_linear_count) as _,
            //             &STree16::search_with_find(BTreeNode::find_ctz) as _,
            //             &STree16::search_with_find(BTreeNode::find_ctz_signed) as _,
            //             &STree16::search_with_find(BTreeNode::find_popcnt_portable) as _,
            //             &STree16::search_with_find(BTreeNode::find_popcnt) as _,
            //             // SECTION 3.1: Batching
            //             &batched(STree16::batch::<1>),
            //             &batched(STree16::batch::<2>),
            //             &batched(STree16::batch::<4>),
            //             &batched(STree16::batch::<8>),
            //             &batched(STree16::batch::<16>),
            //             &batched(STree16::batch::<32>),
            //             &batched(STree16::batch::<64>),
            //             &batched(STree16::batch::<128>),
            //             // SECTION 3.2: prefetch
            //             &batched(STree16::batch_prefetch::<128>),
            //             // SECTION 3.3: pointer arithmetic
            //             &batched(STree16::batch_splat::<128>),
            //             &batched(STree16::batch_ptr::<128>),
            //             &batched(STree16::batch_byte_ptr::<128>),
            //             &batched(STree16::batch_final::<128>),
            //             // SECTION 3.4: skip prefetch
            //             &batched(STree16::batch_skip_prefetch::<128, 1>),
            //             &batched(STree16::batch_skip_prefetch::<128, 2>),
            //             &batched(STree16::batch_skip_prefetch::<128, 3>),
            //             // SECTION 3.4: interleaving
            //             &full(STree16::batch_interleave_last::<64, 2>),
            //             &full(STree16::batch_interleave_last::<64, 3>),
            //             &full(STree16::batch_interleave_all_128),
            //         ],
            //         "",
            //     );

            //     // SECTION 4: layout

            //     // SECTION 4.1: left-max tree
            //     let exps = [
            //         &batched(STree16::batch_final::<128>) as &dyn SearchScheme<_>,
            //         &full(STree16::batch_interleave_all_128),
            //     ];
            //     run_exps(
            //         &mut results,
            //         size,
            //         &STree16::new_params(vals, true, false, false),
            //         qs,
            //         run,
            //         &exps,
            //         "LeftMax",
            //     );

            //     // SECTION 4.2: Memory layouts

            //     // reverse
            //     run_exps(
            //         &mut results,
            //         size,
            //         &STree16::new_params(vals, true, true, false),
            //         qs,
            //         run,
            //         &exps,
            //         "LeftMax+Reverse",
            //     );

            //     // full
            //     run_exps(
            //         &mut results,
            //         size,
            //         &STree16::new_params(vals, true, false, true),
            //         qs,
            //         run,
            //         &[
            //             &batched(STree16::batch_final::<128>),
            //             &full(STree16::batch_interleave_all_128),
            //             &batched(STree16::batch_final_full::<128>),
            //         ],
            //         "LeftMax+Full",
            //     );

            //     // SECTION 4.3: B=15

            //     let index = STree15::new_params(vals, true, false, false);
            //     run_exps(
            //         &mut results,
            //         size,
            //         &index,
            //         qs,
            //         run,
            //         &[
            //             &batched(STree15::batch_final::<128>),
            //             &full(STree15::batch_interleave_all_128),
            //         ],
            //         "LeftMax",
            //     );

            //     // SECTION 5: Prefix partitioning

            //     // SECTION 5.1: Full layout over all subtrees
            //     for &b in &bs {
            //         try_run_exps(
            //             &mut results,
            //             size,
            //             &PartitionedSTree16::try_new(vals, b),
            //             qs,
            //             run,
            //             &[
            //                 &batched(PartitionedSTree16::search::<128, false>),
            //                 &batched(PartitionedSTree16::search::<128, true>),
            //             ],
            //             &format!("{b}"),
            //         );
            //     }

            //     // SECTION 5.2: Packed subtrees
            //     for &b in &bs {
            //         try_run_exps(
            //             &mut results,
            //             size,
            //             &PartitionedSTree16C::try_new(vals, b),
            //             qs,
            //             run,
            //             &[
            //                 &batched(PartitionedSTree16C::search::<128, false>),
            //                 &batched(PartitionedSTree16C::search::<128, true>),
            //             ],
            //             &format!("{b}"),
            //         );
            //     }

            //     // SECTION 5.3: Reduced level-1 branching factor
            //     for &b in &bs {
            //         let index = PartitionedSTree16L::try_new(vals, b);
            //         try_run_exps(
            //             &mut results,
            //             size,
            //             &index,
            //             qs,
            //             run,
            //             &[&batched(PartitionedSTree16L::search::<128, false>)],
            //             &format!("{b}"),
            //         );
            //     }

            //     // SECTION 5.4: Overlapping subtrees
            //     for &b in &bs {
            //         try_run_exps(
            //             &mut results,
            //             size,
            //             &PartitionedSTree16O::try_new(vals, b),
            //             qs,
            //             run,
            //             &[&batched(PartitionedSTree16O::search::<128, false>)],
            //             &format!("{b}"),
            //         );
            //     }

            //     // SECTION 5.5: Prefix map
            //     for &b in &bs {
            //         try_run_exps(
            //             &mut results,
            //             size,
            //             &PartitionedSTree16M::try_new(vals, b),
            //             qs,
            //             run,
            //             &[&batched(PartitionedSTree16M::search::<128, true>)],
            //             &format!("{b}"),
            //         );
            //     }

            //     // SECTION 5.6: Prefix lookup + interleaving
            //     for &b in &bs {
            //         try_run_exps(
            //             &mut results,
            //             size,
            //             &PartitionedSTree16M::try_new(vals, b),
            //             qs,
            //             run,
            //             &[&full(PartitionedSTree16M::search_interleave_128)],
            //             &format!("{b}"),
            //         );
            //     }
        }

        save_results(
            &results,
            if ARGS.range {
                if ARGS.human {
                    "results-human-range"
                } else {
                    "results-range"
                }
            } else {
                if ARGS.human {
                    "results-human"
                } else {
                    "results"
                }
            },
        );
        eprintln!("Saved results after run {}", run + 1);
    }
}

/// Return an iterator over sizes to iterate over.
/// Starts at 32B and goes up to ~1GB.
pub fn sizes() -> Vec<usize> {
    let mut v = vec![];
    let from = ARGS.from.unwrap_or(5);
    let release = ARGS.release;
    let dense = release || ARGS.dense;
    let to = ARGS.to.unwrap_or(if release { 30 } else { 28 });
    for b in from..to {
        let base = 1 << b;
        v.push(base);
        if dense {
            v.push(base * 5 / 4);
            v.push(base * 3 / 2);
            v.push(base * 7 / 4);
        }
    }
    v.push(1 << to);
    v
}

pub fn save_results(results: &Vec<Result>, name: &str) {
    let dir = Path::new("results").to_owned();
    std::fs::create_dir_all(&dir).unwrap();
    let f = if ARGS.release {
        dir.join(&format!("{name}-release"))
    } else {
        dir.join(name)
    };
    let f = f.with_extension("json");
    let f = std::fs::File::create(f).unwrap();
    serde_json::to_writer(f, &results).unwrap();
}

fn run_exps<I: SearchIndex>(
    results: &mut Vec<Result>,
    size: usize,
    index: &I,
    qs: &Vec<u32>,
    run: usize,
    exps: &[&(dyn SearchScheme<I>)],
    name: &str,
) {
    for &exp in exps {
        results.push(Result::new(name, size, index, qs, exp, run, 1));
        results.push(Result::new(name, size, index, qs, exp, run, 6));
    }
}
fn try_run_exps<I: SearchIndex>(
    results: &mut Vec<Result>,
    size: usize,
    index: &Option<I>,
    qs: &Vec<u32>,
    run: usize,
    exps: &[&(dyn SearchScheme<I>)],
    name: &str,
) {
    if let Some(index) = index {
        run_exps(results, size, index, qs, run, exps, name);
    } else {
        for &exp in exps {
            results.push(Result::skip(name, size, qs, exp, run, 1));
        }
    }
}

#[derive(serde::Serialize)]
pub struct Result {
    /// Index name
    pub params: String,
    /// SearchScheme name
    pub scheme: String,
    /// Input size in bytes.
    pub size: usize,
    /// Datastructure size in bytes.
    pub index_size: usize,
    /// Number of queries.
    pub queries: usize,
    /// Number of parallel threads
    pub threads: usize,
    /// Id
    pub run: usize,
    /// Total duration of the experiment.
    pub duration: Duration,
    /// Latency (or reverse throughput) of each operation, in nanoseconds.
    pub latency: f64,
    /// Number of layers in the tree.
    pub layers: usize,
    /// Number of clock cycles per operation.
    pub cycles: f64,
    /// CPU frequency in Hz.
    pub freq: f64,
}

impl Result {
    pub fn new<I: SearchIndex>(
        name: &str,
        size: usize,
        index: &I,
        qs: &[u32],
        scheme: &dyn SearchScheme<I>,
        run: usize,
        threads: usize,
    ) -> Result {
        // println!("n = {size:>8} {} / {} RUNNING", scheme.name(), name);
        let chunk_size = qs.len().div_ceil(threads);

        let start = Instant::now();

        rayon::scope(|scope| {
            for idx in 0..threads {
                let index = &index;
                // let scheme = scheme;
                scope.spawn(move |_| {
                    let start_idx = idx * chunk_size;
                    let end = ((idx + 1) * chunk_size).min(qs.len());
                    let qs_thread = &qs[start_idx..end];
                    black_box(scheme.query(index, qs_thread));
                });
            }
        });

        let duration = start.elapsed();
        let queries = qs.len();
        let freq = get_cpu_freq().unwrap();
        let latency = duration.as_nanos() as f64 / queries as f64;
        let cycles = latency / 1000000000. * freq;

        let sz = size::Size::from_bytes(size);
        let sz = format!("{}", sz);

        println!("n = {sz:>8}, s/it: {latency:>6.2?}ns cycles/it: {cycles:>7.2} freq: {freq:>10}",);
        Result {
            params: name.to_string(),
            scheme: scheme.name().to_string(),
            size,
            index_size: index.size(),
            queries,
            threads,
            layers: index.layers(),
            run,
            duration,
            latency,
            cycles,
            freq,
        }
    }

    pub fn skip<I: SearchIndex>(
        name: &str,
        size: usize,
        qs: &[u32],
        scheme: &dyn SearchScheme<I>,
        run: usize,
        threads: usize,
    ) -> Result {
        Result {
            params: name.to_string(),
            scheme: scheme.name().to_string(),
            size,
            index_size: usize::MAX,
            queries: qs.len(),
            threads,
            run,
            layers: 0,
            duration: Duration::ZERO,
            latency: 0.,
            cycles: 0.,
            freq: 0.,
        }
    }
}

/// Return the current CPU frequency in Hz.
fn get_cpu_freq() -> Option<f64> {
    let cur_cpu = get_cpu()?;
    let path = format!("/sys/devices/system/cpu/cpu{cur_cpu}/cpufreq/scaling_cur_freq");
    let path = &Path::new(&path);
    if !path.exists() {
        return None;
    }

    let val = std::fs::read_to_string(path).ok()?;
    Some(val.trim().parse::<f64>().ok()? * 1000.)
}

fn get_cpu() -> Option<i32> {
    #[cfg(not(target_os = "macos"))]
    {
        Some(unsafe { libc::sched_getcpu() })
    }
    #[cfg(target_os = "macos")]
    {
        None
    }
}
