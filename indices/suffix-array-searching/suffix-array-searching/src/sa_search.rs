use std::{
    ops::{Index, Range},
    simd::{cmp::SimdPartialEq, Simd},
    slice::from_raw_parts,
};

use crate::util::*;
use itertools::Itertools;
use log::info;

pub struct SaNaive<'a> {
    t: &'a Seq,
    sa: Vec<u32>,
    /// Table on the first p bits.
    p: usize,
    /// For each prefix, the index in the suffix array of the first element larger than it.
    /// Has length 2^p + 1.
    table: Vec<u32>,
}

impl Index<usize> for SaNaive<'_> {
    type Output = u32;

    fn index(&self, i: usize) -> &Self::Output {
        unsafe { self.sa.get_unchecked(i) }
    }
}

impl<'t> SaNaive<'t> {
    pub fn build(t: &'t Seq) -> Self {
        let p = 0;
        let mut sa = vec![0; t.len() + 100000];
        sais::sais64::parallel::sais(t, &mut sa, None, 5).unwrap();
        sa.resize(t.len(), 0);
        let sa: Vec<u32> = sa.into_iter().map(|x| x as u32).collect();
        for (&x, &y) in sa.iter().tuple_windows() {
            assert!(t[x as usize..] < t[y as usize..]);
        }

        info!("Building table..");
        assert!(p <= 32);
        let mut s = Self {
            t,
            sa,
            p,
            table: vec![],
        };
        s.fill_prefix_table();
        let sa_size = std::mem::size_of_val(s.sa.as_slice()) / 1024 / 1024;
        let table_size = std::mem::size_of_val(s.table.as_slice()) / 1024 / 1024;
        info!("Suffix array size: {sa_size:>13}MB");
        info!(
            "Prefix table size: {table_size:>13}MB = {:.3} * SA size  (p={p})",
            table_size as f32 / sa_size as f32
        );
        s
    }

    fn fill_prefix_table(&mut self) {
        self.table = vec![0; (1 << self.p) + 1];
        self.table[1 << self.p] = self.sa.len() as u32;
        let mut last_prefix = 0;
        // TODO: We can also binary search for each prefix.
        for i in 0..self.sa.len() {
            let prefix = self.prefix(&self.t[self.sa[i] as usize..]);
            if prefix != last_prefix {
                assert!(prefix > last_prefix);
                while last_prefix < prefix {
                    last_prefix += 1;
                    self.table[last_prefix] = i as u32;
                }
            }
        }
    }

    pub fn suffix(&self, i: usize) -> &Seq {
        self.suffix_at(self[i])
    }
    pub fn suffix_at(&self, i: u32) -> &Seq {
        unsafe { self.t.get_unchecked(i as usize..) }
    }

    pub fn prefix(&self, _q: &Seq) -> usize {
        return 0;
    }

    pub fn prefix_range(&self, q: &Seq, cnt: &mut usize) -> Range<usize> {
        if self.p > 0 {
            *cnt += 1;
        }
        let prefix = self.prefix(q);
        let start = self.table[prefix as usize] as usize;
        let end = self.table[prefix as usize + 1] as usize;
        start..end
    }
}

pub fn binary_search(sa: &SaNaive, q: &Seq, cnt: &mut usize) -> usize {
    let range = sa.prefix_range(q, cnt);
    let mut l = range.start;
    let mut r = range.end;
    while l < r {
        let m = (l + r) / 2;
        *cnt += 1;
        if sa.suffix(m) < q {
            l = m + 1;
        } else {
            r = m;
        }
    }
    sa[l] as usize
}

//   *........*...................*............*...
//   00000000 1111111111111111111 222222222222 3333
//   00111223 0000111111111222223 012222233333 1123
//                       *
//           *
//                  *

pub fn binary_search_cmp(sa: &SaNaive, q: &Seq, cnt: &mut usize) -> usize {
    let range = sa.prefix_range(q, cnt);
    let mut l = range.start;
    let mut r = range.end;
    while l < r {
        let m = (l + r) / 2;
        *cnt += 1;
        let t = sa.suffix(m);
        if cmp(t, q) {
            l = m + 1;
        } else {
            r = m;
        }
    }
    sa[l] as usize
}

pub fn branchy_search(sa: &SaNaive, q: &Seq, cnt: &mut usize) -> usize {
    let range = sa.prefix_range(q, cnt);
    let mut l = range.start;
    let mut r = range.end;
    while l < r {
        let m = (l + r) / 2;
        *cnt += 1;
        let t = sa.suffix(m);
        if t < q {
            l = m + 1;
        } else if t > q {
            r = m;
        } else {
            return m;
        }
    }
    sa[l] as usize
}

pub fn binary_search_batch<const B: usize>(
    sa: &SaNaive,
    q: [&Seq; B],
    cnt: &mut usize,
) -> [usize; B] {
    let mut l = [0; B];
    let mut r = [sa.sa.len(); B];
    let mut max_len = 0;
    for i in 0..B {
        let range = sa.prefix_range(q[i], cnt);
        l[i] = range.start;
        r[i] = range.end;
        max_len = max_len.max(r[i] - l[i]);
    }
    let iterations = max_len.ilog2() + 1;
    for _ in 0..iterations {
        let mut m = [0; B];
        let mut idx = [0; B];
        let mut t = [&sa.t[..0]; B];
        for i in 0..B {
            m[i] = (l[i] + r[i]) / 2;
            *cnt += 1;
            prefetch_index(&sa.sa, m[i]);
        }
        for i in 0..B {
            idx[i] = sa[m[i]];
            prefetch_index(&sa.t, idx[i] as usize);
            prefetch_index(&sa.t, idx[i] as usize + 15);
        }
        for i in 0..B {
            t[i] = &sa.suffix_at(idx[i]);
            if t[i] < q[i] {
                l[i] = m[i] + 1;
            } else {
                r[i] = m[i];
            }
        }
    }
    l.map(|l| sa.sa[l] as usize)
}

pub fn binary_search_batch_c<const B: usize>(
    sa: &SaNaive,
    q: [&Seq; B],
    cnt: &mut usize,
) -> [usize; B] {
    let mut l = [0; B];
    let mut r = [sa.sa.len(); B];
    let mut max_len = 0;
    for i in 0..B {
        let range = sa.prefix_range(q[i], cnt);
        l[i] = range.start;
        r[i] = range.end;
        max_len = max_len.max(r[i] - l[i]);
    }
    let iterations = max_len.ilog2() + 1;
    // let o = sa.p / 2;
    let o = 0;
    for _ in 0..iterations {
        let mut m = [0; B];
        let mut idx = [0; B];
        let mut t = [&sa.t[..0]; B];
        for i in 0..B {
            m[i] = (l[i] + r[i]) / 2;
            *cnt += 1;
            prefetch_index(&sa.sa, m[i]);
        }
        for i in 0..B {
            idx[i] = sa[m[i]];
            prefetch_index(&sa.t, idx[i] as usize + o);
            prefetch_index(&sa.t, idx[i] as usize + o + 15);
        }
        for i in 0..B {
            t[i] = &sa.suffix_at(idx[i]);
            if unsafe { cmp(&t[i].get_unchecked(o..), &q[i].get_unchecked(o..)) } {
                l[i] = m[i] + 1;
            } else {
                r[i] = m[i];
            }
        }
    }
    l.map(|l| sa.sa[l] as usize)
}

pub fn branchfree_search(sa: &SaNaive, q: &Seq, cnt: &mut usize) -> usize {
    let range = sa.prefix_range(q, cnt);
    let mut l = range.start;
    let mut n = range.end - range.start;
    while n > 0 {
        let half = (n + 1) / 2;
        *cnt += 1;
        l = if sa.suffix(l + half) < q { l + half } else { l };
        n -= half;
    }
    sa.sa[l] as usize
}

pub fn branchfree_search_batch<const B: usize>(
    sa: &SaNaive,
    q: [&Seq; B],
    cnt: &mut usize,
) -> [usize; B] {
    let mut l = [0; B];
    let mut n = [0; B];
    let mut max_len = 0;
    for i in 0..B {
        let range = sa.prefix_range(q[i], cnt);
        l[i] = range.start;
        n[i] = range.end - range.start;
        max_len = max_len.max(n[i]);
    }
    let iterations = max_len.ilog2() + 1;
    for _ in 0..iterations {
        let mut m = [0; B];
        let mut idx = [0; B];
        let mut t = [&sa.t[..0]; B];
        for i in 0..B {
            let half = (n[i] + 1) / 2;
            n[i] -= half;
            m[i] = l[i] + half;
            *cnt += 1;
            prefetch_index(&sa.sa, m[i]);
        }
        for i in 0..B {
            idx[i] = sa[m[i]];
            prefetch_index(&sa.t, idx[i] as usize);
            prefetch_index(&sa.t, idx[i] as usize + 15);
        }
        for i in 0..B {
            t[i] = &sa.suffix_at(idx[i]);
            l[i] = if t[i] < q[i] { m[i] } else { l[i] }
        }
    }
    l.map(|l| sa.sa[l] as usize)
}

pub fn branchfree_batch_cmp<const B: usize>(
    sa: &SaNaive,
    q: [&Seq; B],
    cnt: &mut usize,
) -> [usize; B] {
    let mut l = [0; B];
    let mut n = [0; B];
    let mut max_len = 0;
    for i in 0..B {
        let range = sa.prefix_range(q[i], cnt);
        l[i] = range.start;
        n[i] = range.end - range.start;
        max_len = max_len.max(n[i]);
    }
    let iterations = max_len.ilog2() + 1;
    // we can skip the first o characters.
    let o = 0;
    // let o = sa.p / 2;
    for _ in 0..iterations {
        let mut m = [0; B];
        let mut idx = [0; B];
        let mut t = [&sa.t[..0]; B];
        for i in 0..B {
            let half = (n[i] + 1) / 2;
            n[i] -= half;
            m[i] = l[i] + half;
            *cnt += 1;
            prefetch_index(&sa.sa, m[i]);
        }
        for i in 0..B {
            idx[i] = sa[m[i]];
            prefetch_index(&sa.t, idx[i] as usize + o);
            prefetch_index(&sa.t, idx[i] as usize + o + 15);
        }
        for i in 0..B {
            t[i] = &sa.suffix_at(idx[i]);

            l[i] = if unsafe { cmp(&t[i].get_unchecked(o..), &q[i].get_unchecked(o..)) } {
                m[i]
            } else {
                l[i]
            }
        }
    }
    l.map(|l| sa.sa[l] as usize)
}

/// Compare query to text.
/// Assumes:
/// - q.len() < t.len()
/// - both t and q can be safely indexed past-the-end for ~32 characters.
/// TODO: Keep track of the longest common prefix between query and left/right bounds,
/// so that the equal part can be skipped.
pub fn cmp(t: &[u8], q: &[u8]) -> bool {
    let mut len = q.len();
    let mut t = t.as_ptr();
    let mut q = q.as_ptr();
    const B: usize = 16;
    loop {
        unsafe {
            let t_head: [u8; B] = from_raw_parts(t, B).try_into().unwrap();
            let simd_t: Simd<u8, B> = Simd::from_array(t_head);
            let q_head: [u8; B] = from_raw_parts(q, B).try_into().unwrap();
            let simd_q: Simd<u8, B> = Simd::from_array(q_head);
            let eq = simd_t.simd_eq(simd_q).to_bitmask() as u32;
            let cnt = if cfg!(target_endian = "little") {
                eq.trailing_ones()
            } else {
                eq.leading_ones()
            } as usize;
            if cnt < B && cnt < len {
                return *t.add(cnt) < *q.add(cnt);
            }
            if len < B {
                return false;
            }
            t = t.add(B);
            q = q.add(B);
            len -= B;
        }
    }
}

pub fn interpolation_search<const K: usize>(sa: &SaNaive, q: &Seq, cnt: &mut usize) -> usize {
    let range = sa.prefix_range(q, cnt);
    let mut l = range.start;
    let mut r = range.end;
    let mut l_val = string_value::<K>(sa.suffix(l));
    let mut r_val = if r < sa.sa.len() {
        string_value::<K>(sa.suffix(r))
    } else {
        4usize.pow(K as u32)
    };
    let q_val = string_value::<K>(q);
    assert!(
        r_val.checked_mul(r).is_some(),
        "Too large K causes integer overflow."
    );
    // n = 10^9
    // lg n = 30
    // lg lg n = 5
    //
    // 1111111111111111111111112 9999
    // *.  --------------------     *
    //  *.                          *
    //   *                          *
    //    *                         *
    //                         *    *
    //
    while l < r {
        *cnt += 1;
        // The +1 and +2 ensure l<m<r.
        // HOT: The division is slow.
        let mut m = l + ((r - l) * (q_val - l_val + 1)) / (r_val - l_val + 2);
        let low = l + (r - l) / 16;
        let high = l + 15 * (r - l) / 16;
        m = m.clamp(low, high);
        let t = sa.suffix(m);
        let m_val = string_value::<K>(t);
        if t < q {
            l = m + 1;
            l_val = m_val;
        } else {
            r = m;
            r_val = m_val;
        }
    }
    sa.sa[l] as usize
}

pub fn bench(sa: &SaNaive, queries: &[&Seq], name: &str, f: F1) {
    let start = std::time::Instant::now();
    let mut cnt = 0;
    for &q in queries {
        f(sa, q, &mut cnt);
    }
    let elapsed = start.elapsed();
    let per_query = elapsed / queries.len() as u32;
    let per_suffix = elapsed / cnt as u32;
    let cnt_per_query = cnt as f32 / queries.len() as f32;
    eprintln!(
        "{name:<20}: {elapsed:>8.2?} {per_query:>6.0?} {per_suffix:>6.0?} {cnt_per_query:>5.2?}"
    );
}

pub fn bench_batch<const B: usize>(sa: &SaNaive, queries: &[&Seq], name: &str, f: F<B>) {
    let start = std::time::Instant::now();
    let mut cnt = 0;
    for &q in queries.array_chunks::<B>() {
        f(sa, q, &mut cnt);
    }
    let elapsed = start.elapsed();
    let per_query = elapsed / queries.len() as u32;
    let per_suffix = elapsed / cnt as u32;
    let cnt_per_query = cnt as f32 / queries.len() as f32;
    eprintln!(
        "{name:<20}: {elapsed:>8.2?} {per_query:>6.0?} {per_suffix:>6.0?} {cnt_per_query:>5.2?}"
    );
}

type F1 = fn(&SaNaive, &[u8], &mut usize) -> usize;
type F<const B: usize> = fn(&SaNaive, [&[u8]; B], &mut usize) -> [usize; B];

#[ctor::ctor]
fn init_color_backtrace() {
    color_backtrace::install();
}
