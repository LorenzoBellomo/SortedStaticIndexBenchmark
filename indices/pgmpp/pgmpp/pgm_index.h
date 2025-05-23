// This file is part of PGM-index <https://github.com/gvinciguerra/PGM-index>.
// Copyright (c) 2018 Giorgio Vinciguerra.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#pragma once

#include "piecewise_linear_model.h"
#include "search_algo.h"
#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <iterator>
#include <limits>
#include <stdexcept>
#include <string>
#include <type_traits>
#include <utility>
#include <vector>

#include <iostream>
namespace pgmpp {

#define PGM_SUB_EPS(x, epsilon) ((x) <= (epsilon) ? 0 : ((x) - (epsilon)))
#define PGM_ADD_EPS(x, epsilon, size) ((x) + (epsilon) + 2 >= (size) ? (size) : (x) + (epsilon) + 2)

/**
 * A struct that stores the result of a query to a @ref PGMIndex, that is, a range [@ref lo, @ref hi)
 * centered around an approximate position @ref pos of the sought key.
 */
struct ApproxPos {
    size_t pos; ///< The approximate position of the key.
    size_t lo;  ///< The lower bound of the range.
    size_t hi;  ///< The upper bound of the range.
};

/**
 * A space-efficient index that enables fast search operations on a sorted sequence of numbers.
 *
 * A search returns a struct @ref ApproxPos containing an approximate position of the sought key in the sequence and
 * the bounds of a range where the sought key is guaranteed to be found if present.
 * If the key is not present, a @ref std::lower_bound search on the range finds a key that is greater or equal to the
 * sought key, if any.
 * In the case of repeated keys, the index finds the position of the first occurrence of a key.
 *
 * The @p Epsilon template parameter should be set according to the desired space-time trade-off. A smaller value
 * makes the estimation more precise and the range smaller but at the cost of increased space usage.
 *
 * Internally the index uses a succinct piecewise linear mapping from keys to their position in the sorted order.
 * This mapping is represented as a sequence of linear models (segments) which, if @p EpsilonRecursive is not zero, are
 * themselves recursively indexed by other piecewise linear mappings.
 *
 * @tparam K the type of the indexed keys
 * @tparam Epsilon controls the size of the returned search range
 * @tparam EpsilonRecursive controls the size of the search range in the internal structure
 * @tparam Floating the floating-point type to use for slopes
 */
template<typename K, size_t Epsilon = 64, size_t EpsilonRecursive = 4, bool BranchLessSearch = false, size_t linear_search_threshold = 32, typename Floating = float>
class PGMIndex {
protected:
    template<typename, size_t, size_t, uint8_t, typename>
    friend class BucketingPGMIndex;

    template<typename, size_t, typename>
    friend class EliasFanoPGMIndex;

    static_assert(Epsilon > 0);
    struct Segment;

    size_t n;                           ///< The number of elements this index was built on.
    K first_key;                        ///< The smallest element.
    std::vector<Segment> segments;      ///< The segments composing the index.
    std::vector<size_t> levels_offsets; ///< The starting position of each level in segments[], in reverse order.
    std::vector<size_t> levels_segment_count;
    int start_level;
    
    /// Sentinel value to avoid bounds checking.
    static constexpr K sentinel = std::numeric_limits<K>::has_infinity ? std::numeric_limits<K>::infinity()
                                                                       : std::numeric_limits<K>::max();

    template<typename RandomIt>
    static void build(RandomIt first, RandomIt last,
                      size_t epsilon, size_t epsilon_recursive,
                      std::vector<Segment> &segments,
                      std::vector<size_t> &levels_offsets,
                      std::vector<size_t> &levels_segment_count,
                      int &start_level) {
        auto n = (size_t) std::distance(first, last);
        if (n == 0)
            return;

        levels_offsets.push_back(0);
        segments.reserve(n / (epsilon * epsilon));

        if (*std::prev(last) == sentinel)
            throw std::invalid_argument("The value " + std::to_string(sentinel) + " is reserved as a sentinel.");

        auto build_level = [&](auto epsilon, auto in_fun, auto out_fun, size_t last_n) {
            auto n_segments = internal::make_segmentation_par(last_n, epsilon, in_fun, out_fun);
            if (segments.back() == sentinel)
                --n_segments;
            else {
                if (segments.back()(sentinel - 1) < last_n)
                    segments.emplace_back(*std::prev(last) + 1, 0, last_n); // Ensure keys > last are mapped to last_n
                segments.emplace_back(sentinel, 0, last_n);
            }
            return n_segments;
        };

        // Build first level
        auto in_fun = [&](auto i) { return K(first[i]); };
        auto out_fun = [&](auto cs) { segments.emplace_back(cs); };
        auto last_n = build_level(epsilon, in_fun, out_fun, n);
        levels_offsets.push_back(segments.size());

        // Build upper levels
        while (epsilon_recursive && last_n > 1) {
            auto offset = levels_offsets[levels_offsets.size() - 2];
            auto in_fun_rec = [&](auto i) { return segments[offset + i].key; };
            last_n = build_level(epsilon_recursive, in_fun_rec, out_fun, last_n);
            levels_offsets.push_back(segments.size());
        }
        
        // Compute level segment count
        for (size_t i=1; i<levels_offsets.size(); ++i) {
            levels_segment_count.push_back(levels_offsets[i] - levels_offsets[i-1]);
        }
        
        for (size_t i=0; i<levels_segment_count.size(); ++i) {
            if (levels_segment_count[i] <= 16) {
                start_level = i;
                break;
            }
        }
    }

    /**
     * Returns the segment responsible for a given key, that is, the rightmost segment having key <= the sought key.
     * @param key the value of the element to search for
     * @return an iterator to the segment responsible for the given key
     */
    auto segment_for_key(const K &key) const {
        if constexpr (EpsilonRecursive == 0) {
            if constexpr (BranchLessSearch) {
                return std::prev(search::upper_bound_branchless(segments.begin(), segments.begin() + segments_count(), key));
            } else {
                return std::prev(std::upper_bound(segments.begin(), segments.begin() + segments_count(), key));
            }
        }

        auto it = segments.begin() + *(levels_offsets.end() - 2);
        auto l = int(height()) - 2;
        
//        if (BranchLessSearch) {
//            auto start_begin = segments.begin() + levels_offsets[start_level];
//            for (; std::next(start_begin)->key <= key; ++start_begin)
//                continue;
//            it = start_begin;
//            l = start_level - 1;
//        }
        
        for (; l >= 0; --l) {
            auto level_begin = segments.begin() + levels_offsets[l];
            auto pos = std::min<size_t>((*it)(key), std::next(it)->intercept);
            auto lo = level_begin + PGM_SUB_EPS(pos, EpsilonRecursive + 1);

//            static constexpr size_t linear_search_threshold = 8 * 64 / sizeof(Segment);
            if constexpr (EpsilonRecursive <= linear_search_threshold) {
                for (; std::next(lo)->key <= key; ++lo)
                    continue;
                it = lo;
            } else {
                auto level_size = levels_offsets[l + 1] - levels_offsets[l] - 1;
                auto hi = level_begin + PGM_ADD_EPS(pos, EpsilonRecursive, level_size);
                if constexpr (BranchLessSearch) {
                    it = std::prev(search::upper_bound_branchless(lo, hi, key));
                } else {
                    it = std::prev(std::upper_bound(lo, hi, key));
                }
            }
        }
        return it;
    }

public:

    static constexpr size_t epsilon_value = Epsilon;

    /**
     * Constructs an empty index.
     */
    PGMIndex() = default;

    /**
     * Constructs the index on the given sorted vector.
     * @param data the vector of keys to be indexed, must be sorted
     */
    explicit PGMIndex(const std::vector<K> &data) : PGMIndex(data.begin(), data.end()) {}

    /**
     * Constructs the index on the sorted keys in the range [first, last).
     */
    template<typename RandomIt>
    PGMIndex(RandomIt first, RandomIt last)
        : n(std::distance(first, last)),
          start_level(0),
          first_key(n ? *first : K(0)),
          segments(),
          levels_offsets(),
          levels_segment_count(){
        build(first, last, Epsilon, EpsilonRecursive, segments, levels_offsets, levels_segment_count, start_level);
    }

    /**
     * Returns the approximate position and the range where @p key can be found.
     * @param key the value of the element to search for
     * @return a struct with the approximate position and bounds of the range
     */
    ApproxPos search(const K &key) const {
        auto k = std::max(first_key, key);
        auto it = segment_for_key(k);
        auto pos = std::min<size_t>((*it)(k), std::next(it)->intercept);
        auto lo = PGM_SUB_EPS(pos, Epsilon);
        auto hi = PGM_ADD_EPS(pos, Epsilon, n);
        return {pos, lo, hi};
    }
    
    template<typename RandomIt>
    RandomIt search_data(RandomIt start, const K &key) {
        auto k = std::max(first_key, key);
        auto it = segment_for_key(k);
        auto pos = std::min<size_t>((*it)(k), std::next(it)->intercept);
        auto lo = PGM_SUB_EPS(pos, Epsilon);
        auto hi = PGM_ADD_EPS(pos, Epsilon, n);
        if constexpr (BranchLessSearch) {
            return search::lower_bound_branchless(start+lo, start+hi, key);
        } else {
            return std::lower_bound(start+lo, start+hi, key);
        }
    }
    

    std::vector<Segment> get_segments() const {
        return segments;
    }

    std::vector<size_t> get_levels_offsets() const {
        return levels_offsets;
    }
    
    std::vector<size_t> get_levels_segment_count() const {
        return levels_segment_count;
    }

    size_t internal_segments_count() const { return levels_offsets.back() - levels_offsets[1]; }

    /**
     * Returns the number of segments in the last level of the index.
     * @return the number of segments
     */
    size_t segments_count() const { return segments.empty() ? 0 : levels_offsets[1] - 1; }

    /**
     * Returns the number of levels of the index.
     * @return the number of levels of the index
     */
    size_t height() const { return levels_offsets.size() - 1; }

    /**
     * Returns the size of the index in bytes.
     * @return the size of the index in bytes
     */
    size_t size_in_bytes() const { return segments.size() * sizeof(Segment) + levels_offsets.size() * sizeof(size_t); }
};

#pragma pack(push, 1)

template<typename K, size_t Epsilon, size_t EpsilonRecursive, bool BranchLessSearch, size_t linear_search_threshold, typename Floating>
struct PGMIndex<K, Epsilon, EpsilonRecursive, BranchLessSearch, linear_search_threshold, Floating>::Segment {
    K key;              ///< The first key that the segment indexes.
    Floating slope;     ///< The slope of the segment.
    uint32_t intercept; ///< The intercept of the segment.

    Segment() = default;

    Segment(K key, Floating slope, uint32_t intercept) : key(key), slope(slope), intercept(intercept) {};

    explicit Segment(const typename internal::OptimalPiecewiseLinearModel<K, size_t>::CanonicalSegment &cs)
        : key(cs.get_first_x()) {
        auto[cs_slope, cs_intercept] = cs.get_floating_point_segment(key);
        if (cs_intercept > std::numeric_limits<decltype(intercept)>::max())
            throw std::overflow_error("Change the type of Segment::intercept to uint64");
        if (cs_intercept < 0)
            throw std::overflow_error("Unexpected intercept < 0");
        slope = cs_slope;
        intercept = cs_intercept;
    }

    friend inline bool operator<(const Segment &s, const K &k) { return s.key < k; }
    friend inline bool operator<(const K &k, const Segment &s) { return k < s.key; }
    friend inline bool operator<(const Segment &s, const Segment &t) { return s.key < t.key; }
    friend inline bool operator<=(const Segment &s, const K &k) { return s.key <= k; }
    friend inline bool operator<=(const K &k, const Segment &s) { return k <= s.key; }
    friend inline bool operator<=(const Segment &s, const Segment &t) { return s.key <= t.key; }

    operator K() { return key; };

    /**
     * Returns the approximate position of the specified key.
     * @param k the key whose position must be approximated
     * @return the approximate position of the specified key
     */
    inline size_t operator()(const K &k) const {
        size_t pos;
        if constexpr (std::is_same_v<K, int64_t> || std::is_same_v<K, int32_t>)
            pos = size_t(slope * double(std::make_unsigned_t<K>(k) - key));
        else
            pos = size_t(slope * double(k - key));
        return pos + intercept;
    }
};

#pragma pack(pop)

}
