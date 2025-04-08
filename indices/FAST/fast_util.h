#pragma once

// Taken from: https://github.com/curtis-sun/TLI/blob/13224d31d2dcb8ad4481529154c4c286cf4c32fc/util.h

#include <immintrin.h>

__m256i static forceinline _mm256_cmpge_epu32(__m256i a, __m256i b) {
  return _mm256_cmpeq_epi32(_mm256_max_epu32(a, b), a);
}

__m256i static forceinline _mm256_cmple_epu32(__m256i a, __m256i b) {
  return _mm256_cmpge_epu32(b, a);
}

__m256i static forceinline _mm256_cmpgt_epu32(__m256i a, __m256i b) {
  return _mm256_xor_si256(_mm256_cmple_epu32(a, b), _mm256_set1_epi32(-1));
}

__m256i static forceinline _mm256_cmplt_epu32(__m256i a, __m256i b) {
  return _mm256_cmpgt_epu32(b, a);
}

__m256i static forceinline _mm256_cmpgt_epu64(__m256i a, __m256i b) {  
  const static __m256i highBit = _mm256_set1_epi64x((long long)0x8000000000000000);   
  a = _mm256_xor_si256(a, highBit);
  b = _mm256_xor_si256(b, highBit);
  return _mm256_cmpgt_epi64(a, b);
}

__m256i static forceinline _mm256_cmplt_epu64(__m256i a, __m256i b) {  
  return _mm256_cmpgt_epu64(b, a);
}

__m256i static forceinline _mm256_cmpge_epu64(__m256i a, __m256i b) {  
  return _mm256_xor_si256(_mm256_cmplt_epu64(a, b), _mm256_set1_epi32(-1));
}

__m256i static forceinline _mm256_cmple_epu64(__m256i a, __m256i b) {  
  return _mm256_cmpge_epu64(b, a);
}
