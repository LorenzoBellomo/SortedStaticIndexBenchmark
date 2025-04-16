use rand::Rng;
use std::path::Path;
use std::{arch::x86_64::_pext_u64, ops::Range};

pub type Seq = [u8];
pub type Sequence = Vec<u8>;

/// Generate a random text of length n.
pub fn random_string(n: usize, rng: &mut impl Rng) -> Sequence {
    let mut seq = Vec::with_capacity(n);
    for _ in 0..n {
        seq.push(rng.gen_range(0..4));
    }
    seq
}

/// Generate a random subset of queries.
pub fn random_queries<'t>(t: &'t Seq, n: usize, rng: &mut impl Rng) -> Vec<&'t Seq> {
    (0..n)
        .map(|_| {
            let i = rng.gen_range(0..t.len() - 200);
            let len = rng.gen_range(30..100);
            &t[i..i + len]
        })
        .collect()
}

#[allow(unused)]
pub trait Search<'t> {
    /// Build a suffix array on the given text.
    fn build(t: &'t Seq) -> Self;

    /// Find position of the smallest suffix >= q.
    fn search(&self, q: &Seq) -> usize;

    /// Iterate over all suffixes that have q as a prefix.
    fn search_prefix(&self, q: &Seq) -> impl Iterator<Item = usize> {
        unimplemented!();
        0..0
    }

    /// Iterate over all suffixes in the range.
    fn search_range(&self, r: Range<&Seq>) -> impl Iterator<Item = usize> {
        unimplemented!();
        0..0
    }
}

/// Prefetch the given cacheline into L1 cache.
pub fn prefetch_index<T>(s: &[T], index: usize) {
    let ptr = unsafe { s.as_ptr().add(index) };
    prefetch_ptr(ptr);
}

/// Prefetch the given cacheline into L1 cache.
pub fn prefetch_ptr<T>(ptr: *const T) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        std::arch::x86_64::_mm_prefetch(ptr as *const i8, std::arch::x86_64::_MM_HINT_T0);
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        std::arch::x86::_mm_prefetch(ptr as *const i8, std::arch::x86::_MM_HINT_T0);
    }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        // TODO: Put this behind a feature flag.
        // std::arch::aarch64::_prefetch(ptr as *const i8, std::arch::aarch64::_PREFETCH_LOCALITY3);
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86", target_arch = "aarch64")))]
    {
        // Do nothing.
    }
}

pub fn string_value<const K: usize>(q: &Seq) -> usize {
    // Read two u64 values starting at q. Use bit-extract instructions to extract the low 2 bits of each byte.
    let mask = 0x0303030303030303u64;
    if K == 8 {
        // Read u64 from q.
        unsafe {
            let a = std::ptr::read_unaligned(q.as_ptr() as *const u64);
            let a = a.swap_bytes();
            // let v1 = _pext_u64(a, mask) as usize;
            // assert_eq!(v0, v1, "\n{:?}\n{v0:0b}\n{v1:0b}", &q[..8]);
            return a.try_into().unwrap();
        }
    }
    if K == 16 {
        unsafe {
            let a = std::ptr::read_unaligned(q.as_ptr() as *const u64);
            let b = std::ptr::read_unaligned(q.as_ptr().add(8) as *const u64);
            let a = a.swap_bytes();
            let b = b.swap_bytes();
            let v1 = ((_pext_u64(a, mask) as usize) << 16) + _pext_u64(b, mask) as usize;
            // assert_eq!(v0, v1);
            return v1;
        }
    }
    if K == 24 {
        unsafe {
            let a = std::ptr::read_unaligned(q.as_ptr() as *const u64);
            let b = std::ptr::read_unaligned(q.as_ptr().add(8) as *const u64);
            let c = std::ptr::read_unaligned(q.as_ptr().add(16) as *const u64);
            let a = a.swap_bytes();
            let b = b.swap_bytes();
            let c = c.swap_bytes();
            let v1 = ((_pext_u64(a, mask) as usize) << 32)
                + ((_pext_u64(b, mask) as usize) << 16)
                + _pext_u64(c, mask) as usize;
            // assert_eq!(v0, v1);
            return v1;
        }
    }
    let v0 = q.iter().take(K).fold(0, |acc, &x| acc * 4 + x as usize);
    v0
}

type SA = Vec<usize>;

pub fn build_sa(text: &Seq) -> SA {
    libdivsufsort_rs::divsufsort(&text)
        .unwrap()
        .iter()
        .map(|x| *x as usize)
        .collect()
}

pub fn time<T>(t: &str, f: impl FnOnce() -> T) -> T {
    eprintln!("{t}: Starting");
    let start = std::time::Instant::now();
    let r = f();
    let elapsed = start.elapsed();
    eprintln!("{t}: Elapsed: {:?}", elapsed);
    r
}

pub fn read_human_genome_with_sa() -> (Sequence, SA) {
    let seq = read_human_genome();
    let sa = build_sa(&seq);
    (seq, sa)
}

pub fn read_fasta_file(path: &Path) -> Sequence {
    let mut map = [0; 256];
    map[b'A' as usize] = 0;
    map[b'C' as usize] = 1;
    map[b'G' as usize] = 2;
    map[b'T' as usize] = 3;

    map[b'a' as usize] = 0;
    map[b'c' as usize] = 1;
    map[b'g' as usize] = 2;
    map[b't' as usize] = 3;

    let mut reader = needletail::parse_fastx_file(path).unwrap();
    let mut out = vec![];
    while let Some(record) = reader.next() {
        out.extend(
            record
                .unwrap()
                .seq()
                .into_iter()
                .map(|c| unsafe { map.get_unchecked(*c as usize) }),
        );
        // break;
    }
    out
}

pub fn read_human_genome() -> Sequence {
    read_fasta_file(&Path::new("human-genome.fa"))
}
