#ifndef SIMDBTREEINTERFACE_HPP
#define SIMDBTREEINTERFACE_HPP

#include <iostream>
#include <vector>
#include <limits>
#include <b_tree.h>
#include <sampled_b_tree.h>

template <typename T, size_t sz = 0>
class SIMDBTreeInterface {
private:
    std::vector<T> data;
    static constexpr size_t log_sz = sz == 0 ? 0 : SIMD_Btree::btree<T, 0>::log_size(sz);
    SIMD_Btree::btree<T, log_sz> btree;

public:
    void prepare(std::vector<T> data_) {
        (void)data_;
    }

    void build(std::vector<T> data_) {
        btree.build(data_.begin(), data_.end());
    }

    T next_geq(T q) {
        auto idx = btree.lower_bound_idx(q);
        if (idx == btree.size())
            return std::numeric_limits<T>::max();
        return btree[idx];
    }

    T access(size_t idx) {
        return btree[idx];
    }

    size_t size_in_bytes() {
        return btree.size_in_bytes();
    }

    std::string to_string() {
        return "SIMD-BTree";
    }
};

template <typename T>
class SIMDSampledBTreeInterface {
private:
    SIMD_Btree::sampled_btree<T> sampled_tree;

public:
    void prepare(std::vector<T> &data_) {}

    void build(std::vector<T> data_) {
        sampled_tree.build(data_.begin(), data_.end());
    }

    T next_geq(T q) {
        auto idx = sampled_tree.lower_bound_idx(q);
        if (idx == sampled_tree.size())
            return std::numeric_limits<T>::max();
        return sampled_tree[idx];
    }

    size_t size_in_bytes() {
        return sampled_tree.size_in_bytes();
    }

    std::string to_string() {
        return "SIMD-SampledBTree";
    }
};

#endif // SIMDBTREEINTERFACE_HPP
