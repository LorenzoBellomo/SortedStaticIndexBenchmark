use itertools::Itertools;

use crate::binary_search::SortedVec;
use std::simd::num::SimdUint;
use std::simd::prelude::SimdFloat;
use std::simd::prelude::SimdInt;
use std::simd::SimdCast;
use std::simd::{cmp::SimdOrd, LaneCount, Simd, SupportedLaneCount};

#[inline(always)]
fn elementwise_division<const N: usize>(num: Simd<u32, N>, den: Simd<u32, N>) -> Simd<u32, N>
where
    LaneCount<N>: SupportedLaneCount,
{
    let mut result: [u32; N] = [0; N];
    let num_ary = num.as_array();
    let den_ary = den.as_array();
    for i in 0..N {
        result[i] = num_ary[i] / den_ary[i];
    }
    Simd::<u32, N>::from_array(result)
}

#[inline(always)]
fn fp_based_division<const N: usize>(num: Simd<u32, N>, den: Simd<u32, N>) -> Simd<u32, N>
where
    LaneCount<N>: SupportedLaneCount,
{
    let num_ary: Simd<f64, N> = num.cast::<f64>();
    let den_ary: Simd<f64, N> = num.cast::<f64>();
    let res = num_ary / den_ary;
    res.cast::<u32>()
}
#[inline(always)]
fn interpolate<const N: usize>(
    ls: &[u32],
    rs: &[u32],
    l_vals: &[u32],
    r_vals: &[u32],
    qs: &[u32],
    i: usize,
) -> [u32; N]
where
    LaneCount<N>: SupportedLaneCount,
{
    let l_vec = Simd::<u32, N>::from_slice(&ls[i * N..(i + 1) * N]);
    let r_vec = Simd::<u32, N>::from_slice(&rs[i * N..(i + 1) * N]);
    let q_vec = Simd::<u32, N>::from_slice(&qs[i * N..(i + 1) * N]);
    let l_vals_vec = Simd::<u32, N>::from_slice(&l_vals[i * N..(i + 1) * N]);
    let r_vals_vec = Simd::<u32, N>::from_slice(&r_vals[i * N..(i + 1) * N]);
    let ones = Simd::<u32, N>::splat(1);
    let twos = Simd::<u32, N>::splat(2);
    // print l_vec, r_vec, q_vec, r_vals_vec, l_vals_vec
    // FIXME: this will overflow
    // HOT: division

    let mut m = l_vec
        + (r_vec - l_vec)
            * fp_based_division(q_vec - l_vals_vec + ones, r_vals_vec - l_vals_vec + twos);
    let low = l_vec + (r_vec - l_vec) / Simd::<u32, N>::splat(16);
    let high = l_vec + Simd::<u32, N>::splat(15) * (r_vec - l_vec) / Simd::<u32, N>::splat(16);
    // equivalent to clamp
    m = m.simd_min(high);
    m = m.simd_max(low);
    m.to_array()
}

impl SortedVec {
    /// Return the value of the first value >= query.
    pub fn interpolation_search(&self, q: u32) -> u32 {
        let mut l: usize = 0;
        // FIXME: is this inclusive?
        let mut r: usize = self.vals.len() - 1;
        let mut l_val: usize = self.get(l).try_into().unwrap();
        let mut r_val: usize = self.get(r).try_into().unwrap();
        let q_val = q.try_into().unwrap();
        if q_val <= l_val {
            return self.get(l);
        }
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
            // The +1 and +2 ensure l<m<r.
            // HOT: The division is slow.
            // OMFG is there really not a better way to do this? Multiple integer types are a nightmare
            // try doing the division in u32
            let mut m: usize = l + (r - l) * (q_val - l_val + 1) / (r_val - l_val + 2);
            let low = l + (r - l) / 16;
            let high = l + 15 * (r - l) / 16;
            m = m.clamp(low, high);
            let m_val: usize = self.get(m).try_into().unwrap();
            if m_val < q_val {
                l = m + 1;
                l_val = m_val;
            } else {
                r = m;
                r_val = m_val;
            }
        }
        self.get(l)
    }

    pub fn interp_search_batched_simd<const P: usize>(&self, qs: &[u32; P]) -> [u32; P] {
        const LANE_COUNT: usize = 4;
        let mut ls: [isize; P] = [0isize; P];
        let mut rs: [isize; P] = [(self.vals.len() - 1).try_into().unwrap(); P];
        let mut l_vals: [isize; P] = ls.map(|i| self.get(i as usize).try_into().unwrap());
        let mut r_vals: [isize; P] = rs.map(|i| self.get(i as usize).try_into().unwrap());
        let qs: [isize; P] = qs
            .iter()
            .map(|&x| x as isize)
            .collect_vec()
            .try_into()
            .unwrap();
        let mut retvals = [0u32; P];
        // an accumulator value going from 0 to P
        let mut done = 0;
        assert!(P % LANE_COUNT == 0);
        let ones = Simd::<isize, LANE_COUNT>::splat(1);
        let twos = Simd::<isize, LANE_COUNT>::splat(2);
        // println!("Iteration! {:?} {:?}", ls, rs);
        done = 0;
        // this is subject to change, maybe we want completely size-agnostic simd
        let simd_iters = P / LANE_COUNT;

        while done < P {
            done = 0;
            for i in 0..simd_iters {
                let q_vec = Simd::<isize, LANE_COUNT>::from_slice(
                    &qs[i * LANE_COUNT..(i + 1) * LANE_COUNT],
                );
                let l_vec = Simd::<isize, LANE_COUNT>::from_slice(
                    &ls[i * LANE_COUNT..(i + 1) * LANE_COUNT],
                );
                let r_vec = Simd::<isize, LANE_COUNT>::from_slice(
                    &rs[i * LANE_COUNT..(i + 1) * LANE_COUNT],
                );

                let l_vals_vec = Simd::<isize, LANE_COUNT>::from_slice(
                    &l_vals[i * LANE_COUNT..(i + 1) * LANE_COUNT],
                );
                let r_vals_vec = Simd::<isize, LANE_COUNT>::from_slice(
                    &r_vals[i * LANE_COUNT..(i + 1) * LANE_COUNT],
                );
                // TODO: do something about the division here
                let mut m = l_vec
                    + (r_vec - l_vec) * (q_vec - l_vals_vec + ones)
                        / (r_vals_vec - l_vals_vec + twos);
                let low = l_vec + (r_vec - l_vec) / Simd::<isize, LANE_COUNT>::splat(16);
                let high = l_vec
                    + Simd::<isize, LANE_COUNT>::splat(15) * (r_vec - l_vec)
                        / Simd::<isize, LANE_COUNT>::splat(16);
                // equivalent to clamp
                m = m.simd_min(high);
                m = m.simd_max(low);
                let m_arr = m.as_array();
                let mut m_val = [0isize; LANE_COUNT];
                for j in 0..LANE_COUNT {
                    if ls[i * LANE_COUNT + j] < rs[i * LANE_COUNT + j] {
                        // so this takes time if it is done for every one of the LANE_COUNT elements
                        // essentially random accesses in memory - could this be done better with sorting of queries?
                        m_val[j] = self.get(m_arr[j] as usize).try_into().unwrap();
                        if m_val[j] < qs[i * LANE_COUNT + j] {
                            ls[i * LANE_COUNT + j] = m_arr[j] + 1;
                            l_vals[i * LANE_COUNT + j] = m_val[j];
                        } else {
                            rs[i * LANE_COUNT + j] = m_arr[j];
                            r_vals[i * LANE_COUNT + j] = m_val[j];
                        }
                    } else {
                        // this is likely cached and fast
                        retvals[i * LANE_COUNT + j] = self.get(ls[i * LANE_COUNT + j] as usize);
                        done += 1;
                    }
                }
            }
        }

        retvals
    }

    pub fn interp_search_batched<const P: usize>(&self, qs: &[u32; P]) -> [u32; P] {
        let mut ls = [0usize; P];
        let mut rs = [self.vals.len() - 1; P];
        let mut l_vals: [usize; P] = ls.map(|i| self.get(i).try_into().unwrap());
        let mut r_vals: [usize; P] = rs.map(|i| self.get(i).try_into().unwrap());
        let mut retvals = [0u32; P];
        let mut done = [false; P];
        let mut done_count = 0;

        // trick to avoid negative values
        for i in 0..P {
            let q_val: usize = qs[i].try_into().unwrap();
            if q_val <= l_vals[i] {
                retvals[i] = self.get(ls[i]);
                done_count += 1;
                done[i] = true;
            }
        }

        // this loop should not be difficult to make a fixed number of iterations, or just not have to check
        // the condition every time
        while done_count < P {
            for i in 0..P {
                // this might be a hard-to-predict branch
                if done[i] {
                    continue;
                }

                let q_val = qs[i].try_into().unwrap();
                let l = ls[i];
                let r = rs[i];
                let l_val = l_vals[i];
                let r_val = r_vals[i];

                if l >= r {
                    retvals[i] = self.get(l);
                    done_count += 1;
                    done[i] = true;
                    continue;
                }

                let mut m: usize = l + (r - l) * (q_val - l_val + 1) / (r_val - l_val + 2);
                let low = l + (r - l) / 16;
                let high = l + 15 * (r - l) / 16;
                m = m.clamp(low, high);
                let m_val = self.get(m).try_into().unwrap();
                if m_val < q_val {
                    ls[i] = m + 1;
                    l_vals[i] = m_val;
                } else {
                    rs[i] = m;
                    r_vals[i] = m_val;
                }
            }
        }

        retvals
    }
}

#[cfg(test)]
mod tests {
    use crate::{binary_search::SortedVec, SearchIndex};

    #[test]
    fn interppolation_vs_binsearch() {
        let input = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let q = 5;
        let i_res = SortedVec::new(&input).interpolation_search(q);
        let bin_res = SortedVec::new(&input).binary_search(q);
        println!("{i_res}, {bin_res}");
        assert!(i_res == bin_res);
    }

    #[test]
    fn normal_vs_batched() {
        let input = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let qs = [0, 1, 2, 3, 4, 5, 6, 9];
        let batch_res = SortedVec::new(&input).interp_search_batched::<8>(&qs);
        let i_res = qs.map(|q| SortedVec::new(&input).interpolation_search(q));
        println!("{:?}, {:?}", i_res, batch_res);
        assert!(i_res == batch_res);
    }

    #[test]
    fn interpolation_batched_simd() {
        let input = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let qs = [0, 1, 2, 3, 4, 5, 6, 9];
        let bin_res = SortedVec::new(&input).interp_search_batched::<8>(&qs);
        let i_res = SortedVec::new(&input).interp_search_batched_simd::<8>(&qs);
        println!("{:?}, {:?}", i_res, bin_res);
        assert!(i_res == bin_res);
    }
}
