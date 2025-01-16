#ifndef SIMDBTREEINTERFACE_HPP
#define SIMDBTREEINTERFACE_HPP

#include <iostream>
#include <vector>
#include <limits>
#include <b_tree.h>
#include <sampled_b_tree.h>
#include "b_tree_pgm.h"

template <typename T, size_t sz = 0>
class SIMDBTreeInterface {
private:
    std::vector<T> data;
    static constexpr size_t log_sz = sz == 0 ? 0 : SIMD_Btree::btree<T, 0>::log_size(sz);
    SIMD_Btree::btree<T, log_sz> btree;

public:
    void prepare(std::vector<T> data_) {
        data = data_;
    }

    void build(std::vector<T> data_) {
        btree.build(data.begin(), data.end());
    }

    T next_geq(T q) {
        auto idx = btree.lower_bound_idx(q);
        if (idx == data.size())
            return std::numeric_limits<T>::max();
        return data[idx];
    }

    size_t size_in_bytes() {
        return btree.size_in_bytes();
    }

    std::string to_string() {
        return (sz == 0) ? "SIMD-BTree" : "SIMD-BTreeOptimized";
    }
};

template <typename T>
class SIMDSampledBTreeInterface {
private:
    std::vector<T> data;
    SIMD_Btree::b_plus_tree<T> bplustree;

public:
    void prepare(std::vector<T> data_) {
        data = data_;
    }

    void build(std::vector<T> data_) {
        bplustree.build(data.begin(), data.end());
    }

    T next_geq(T q) {
        auto idx = bplustree.lower_bound_idx(q);
        if (idx == data.size())
            return std::numeric_limits<T>::max();
        return data[idx];
    }

    size_t size_in_bytes() {
        return bplustree.size_in_bytes();
    }

    std::string to_string() {
        return "SIMD-SampledBTree";
    }
};


template<typename T, int eps>
class SIMDPGMBTreeInterface {
private:
    std::vector<T> data;
    SIMDBTree::pgm::BTreePGMIndex<T, eps> pgmbtree_idx;

public:
    void prepare(const std::vector<T>& data_) {
        data = data_;
    }

    void build(const std::vector<T>& data_) {
        pgmbtree_idx.build(data_.begin(), data_.end());
    }

    T next_geq(T q) {
        auto range = pgmbtree_idx.search(q);
        auto lb = std::lower_bound(data.begin() + range.lo, data.begin() + range.hi, q);
        if (lb == data.end())
            return std::numeric_limits<T>::max();
        return *lb;
    }

    size_t size_in_bytes() {
        return pgmbtree_idx.size_in_bytes();
    }

    std::string to_string() {
        return "PGMBTree" + std::to_string(eps);
    }
};

#endif // SIMDBTREEINTERFACE_HPP
