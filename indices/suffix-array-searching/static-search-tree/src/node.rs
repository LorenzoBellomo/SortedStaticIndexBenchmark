use std::arch::x86_64::{_mm256_movemask_epi8, _mm256_packs_epi32};
use std::mem::transmute;
use std::simd::{prelude::*, LaneCount, SupportedLaneCount};

pub(super) const MAX: u32 = i32::MAX as u32;

#[repr(align(64))]
#[derive(Clone, Copy, Debug)]
pub struct BTreeNode<const N: usize> {
    pub(super) data: [u32; N],
}

impl<const N: usize> Default for BTreeNode<N> {
    fn default() -> BTreeNode<N> {
        BTreeNode { data: [0; N] }
    }
}

impl<const N: usize> BTreeNode<N> {
    /// Return the index of the first element >=q.
    pub fn find(&self, q: u32) -> usize {
        self.find_popcnt(q)
    }

    pub fn find_linear(&self, q: u32) -> usize {
        for i in 0..N {
            if self.data[i] >= q {
                return i;
            }
        }
        N
    }

    pub fn find_linear_count(&self, q: u32) -> usize {
        let mut count = 0;
        for i in 0..N {
            if self.data[i] < q {
                count += 1;
            }
        }
        count
    }

    /// This first does a single comparison to choose the left or right half of the array,
    /// and then uses SIMD on that half.
    /// This may reduce the pressure on SIMD registers.
    pub fn find_split(&self, q: u32) -> usize {
        let idx;
        if q <= self.data[N / 2] {
            idx = 0;
        } else {
            idx = N / 2;
        }
        let half_simd = Simd::<u32, 8>::from_slice(&self.data[idx..idx + N / 2]);
        let q_simd = Simd::splat(q);
        let mask = q_simd.simd_le(half_simd);
        idx + mask.first_set().unwrap_or(8)
    }

    pub fn find_ctz(&self, q: u32) -> usize
    where
        LaneCount<N>: SupportedLaneCount,
    {
        let data: Simd<u32, N> = Simd::from_slice(&self.data[0..N]);
        let q = Simd::splat(q);
        let mask = q.simd_le(data);
        mask.first_set().unwrap_or(N)
    }

    pub fn find_ctz_signed(&self, q: u32) -> usize
    where
        LaneCount<N>: SupportedLaneCount,
    {
        let data: Simd<i32, N> = Simd::from_slice(unsafe { transmute(&self.data[0..N]) });
        let q = Simd::splat(q as i32);
        let mask = q.simd_le(data);
        mask.first_set().unwrap_or(N)
    }

    pub fn find_popcnt_portable(&self, q: u32) -> usize
    where
        LaneCount<N>: SupportedLaneCount,
    {
        let data: Simd<i32, N> = Simd::from_slice(unsafe { transmute(&self.data[0..N]) });
        let q = Simd::splat(q as i32);
        let mask = q.simd_gt(data);
        mask.to_bitmask().count_ones() as usize
    }

    /// Return the index of the first element >=q.
    /// Assumes that all elements fit in an i32, since SIMD doesn't have
    /// unsigned comparisons.
    pub fn find_popcnt(&self, q: u32) -> usize {
        let low: Simd<u32, 8> = Simd::from_slice(&self.data[0..N / 2]);
        let high: Simd<u32, 8> = Simd::from_slice(&self.data[N / 2..N]);
        let q_simd = Simd::<_, 8>::splat(q);
        // Merge the two masks, and convert to a single shuffled(!) mask.
        // But that's OK since popcount doesn't care about order.
        // TODO: Can we do this using portable SIMD?
        unsafe {
            use std::mem::transmute as t;
            let q_simd: Simd<i32, 8> = t(q_simd);
            let mask_low = q_simd.simd_gt(t(low));
            let mask_high = q_simd.simd_gt(t(high));
            let merged = _mm256_packs_epi32(t(mask_low), t(mask_high));
            let mask = _mm256_movemask_epi8(merged);
            mask.count_ones() as usize / 2
        }
    }

    pub fn find_splat(&self, q_simd: Simd<u32, 8>) -> usize {
        let low: Simd<u32, 8> = Simd::from_slice(&self.data[0..N / 2]);
        let high: Simd<u32, 8> = Simd::from_slice(&self.data[N / 2..N]);
        unsafe {
            use std::mem::transmute as t;
            let q_simd: Simd<i32, 8> = t(q_simd);
            let mask_low = q_simd.simd_gt(t(low));
            let mask_high = q_simd.simd_gt(t(high));
            let merged = _mm256_packs_epi32(t(mask_low), t(mask_high));
            let mask = _mm256_movemask_epi8(merged);
            mask.count_ones() as usize / 2
        }
    }

    /// This returns the popcount multiplied by 64.
    pub fn find_splat64(&self, q_simd: Simd<u32, 8>) -> usize {
        let low: Simd<u32, 8> = Simd::from_slice(&self.data[0..N / 2]);
        let high: Simd<u32, 8> = Simd::from_slice(&self.data[N / 2..N]);
        unsafe {
            let q_simd: Simd<i32, 8> = t(q_simd);
            let mask_low = q_simd.simd_gt(t(low));
            let mask_high = q_simd.simd_gt(t(high));
            use std::mem::transmute as t;
            let merged = _mm256_packs_epi32(t(mask_low), t(mask_high));
            let mask = _mm256_movemask_epi8(merged);
            mask.count_ones() as usize * 32
        }
    }

    /// This returns the popcount multiplied by 64.
    /// Normal:   last index < query.
    /// Reversed: last index <= query.
    pub fn find_splat_last(&self, q_simd: Simd<u32, 8>) -> usize {
        let low: Simd<u32, 8> = Simd::from_slice(&self.data[0..N / 2]);
        let high: Simd<u32, 8> = Simd::from_slice(&self.data[N / 2..N]);
        unsafe {
            let q_simd: Simd<i32, 8> = t(q_simd);
            let mask_low = q_simd.simd_ge(t(low));
            let mask_high = q_simd.simd_ge(t(high));
            use std::mem::transmute as t;
            let merged = _mm256_packs_epi32(t(mask_low), t(mask_high));
            let mask = _mm256_movemask_epi8(merged);
            mask.count_ones() as usize / 2
        }
    }

    pub fn find_splat64_last(&self, q_simd: Simd<u32, 8>) -> usize {
        let low: Simd<u32, 8> = Simd::from_slice(&self.data[0..N / 2]);
        let high: Simd<u32, 8> = Simd::from_slice(&self.data[N / 2..N]);
        unsafe {
            let q_simd: Simd<i32, 8> = t(q_simd);
            let mask_low = q_simd.simd_ge(t(low));
            let mask_high = q_simd.simd_ge(t(high));
            use std::mem::transmute as t;
            let merged = _mm256_packs_epi32(t(mask_low), t(mask_high));
            let mask = _mm256_movemask_epi8(merged);
            mask.count_ones() as usize * 32
        }
    }
}
