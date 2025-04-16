use clap::Parser;
use rand::Rng;
use suffix_array_searching::util::read_human_genome;

#[derive(Parser)]
struct Args {
    k: usize,
    random: bool,
}

fn main() {
    let seq = read_human_genome();

    let args = Args::parse();
    let k = args.k;
    let random = args.random;

    let mut counts = vec![0u32; 1 << (2 * k)];
    let mut countsr = vec![0u32; 1 << (2 * k)];
    let mut key = 0;

    let mut rng = rand::thread_rng();

    for i in 0..k - 1 {
        key = key << 2 | seq[i] as usize;
    }
    for i in k - 1..seq.len() {
        key = key << 2 | seq[i] as usize;
        key &= (1 << (2 * k)) - 1;
        counts[key] += 1;
        if random {
            countsr[rng.gen_range(0..counts.len())] += 1;
        }
    }

    // histogram of sizes
    let mut hist = vec![0; 32];
    let mut sum = vec![0; 32];
    for &c in &counts {
        if c > 0 {
            let x = c.ilog2() as usize;
            hist[x] += 1;
            sum[x] += c;
        }
    }

    let mut histr = vec![0; 32];
    let mut sumr = vec![0; 32];
    for &c in &countsr {
        if c > 0 {
            let x = c.ilog2() as usize;
            histr[x] += 1;
            sumr[x] += c;
        }
    }

    let kmers = seq.len() - k + 1;
    let mut acc = 0;

    eprintln!(
        "size {:>10} {:>10} {:>10} {:>10} {:>10}",
        "#buckets", "#kmers", "acc #kmers", "acc #kmers %", "acc #kmers %",
    );
    for i in 0..hist.len() {
        let h = hist[i];
        let s = sum[i];
        acc += s;
        eprintln!(
            "2^{i:>2} {h:>10} {s:>10} {acc:>10} {:>10.4} {:>10.4}",
            acc as f64 / kmers as f64,
            1. - acc as f64 / kmers as f64
        );
    }

    eprintln!("min: {}", counts.iter().min().unwrap());
    eprintln!(
        "avg: {}",
        counts.iter().sum::<u32>() as usize / counts.len()
    );
    eprintln!("max: {}", counts.iter().max().unwrap());

    if random {
        eprintln!("RANDOM");

        eprintln!(
            "size {:>10} {:>10} {:>10} {:>10} {:>10}",
            "#buckets", "#kmers", "acc #kmers", "acc #kmers %", "acc #kmers %",
        );
        for i in 0..histr.len() {
            let h = histr[i];
            let s = sumr[i];
            acc += s;
            eprintln!(
                "2^{i:>2} {h:>10} {s:>10} {acc:>10} {:>10.4} {:>10.4}",
                acc as f64 / kmers as f64,
                1. - acc as f64 / kmers as f64
            );
        }

        eprintln!("min: {}", countsr.iter().min().unwrap());
        eprintln!(
            "avg: {}",
            countsr.iter().sum::<u32>() as usize / counts.len()
        );
        eprintln!("max: {}", countsr.iter().max().unwrap());

        // let out = serde_json::to_string(&counts).unwrap();
        // eprintln!("{}", out);
    }
}
