use std::ops::Index;
use itertools::Itertools;


type Seq = [u8];

pub struct SA<'a> {
    t: &'a Seq,
    sa: Vec<u32>
}

impl Index<usize> for SA<'_> {
    type Output = u32;

    fn index(&self, i: usize) -> &Self::Output {
        unsafe { self.sa.get_unchecked(i) }
    }


}

impl SA<'_> {
    pub fn build(t: &Seq) -> SA {
        // taken over from 
        let mut sa: Vec<i64> = vec![0; t.len() + 100000];
        sais::sais64::parallel::sais(t, &mut sa, None, 5).unwrap();
        sa.resize(t.len(), 0);
        let sa: Vec<u32> = sa.into_iter().map(|x| x as u32).collect();
        // what does this sanity check do?
        for (&x, &y) in sa.iter().tuple_windows() {
            assert!(t[x as usize..] < t[y as usize..]);
        }

        SA {
            t,
            sa,
        }
    }

    pub fn suffix(&self, i: usize) -> &Seq {
        self.suffix_at(self[i])
    }

    pub fn suffix_at(&self, i: u32) -> &Seq {
        unsafe { self.t.get_unchecked(i as usize..) }
    }

}

// completely basic binsearch
pub fn binary_search(sa: &SA, q: &Seq, cnt: &mut usize) -> usize {
    let mut l = 0;
    let mut r = sa.sa.len();
    while l < r {
        *cnt += 1;
        let m = (l + r) / 2;
        if sa.suffix(m) < q {
            l = m + 1;
        } else {
            r = m;
        }
    }
    sa[l] as usize
}


pub fn branchless_bin_search(sa: &SA, q: &Seq, cnt: &mut usize) -> usize {
    let mut base = 0;
    let mut len = sa.sa.len();
    while len > 1 {
        let half = len / 2;
        *cnt += 1;
        base += ((sa.suffix(base + half) < q) as usize) * half;
        len = len - half;
    }
    base
}

type F1 = fn(&SA, &[u8], &mut usize) -> usize;

pub fn bench(sa: &SA, queries: &[&Seq], name: &str, f: F1) {
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
