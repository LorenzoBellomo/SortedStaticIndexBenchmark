#ifndef SIMDBTREEINTERFACE_HPP
#define SIMDBTREEINTERFACE_HPP

#include <iostream>
#include <vector>
#include <limits>
#include <b_tree.h>
#include <sampled_b_tree.h>

template <typename T>
class SIMDBTreeInterface {
private:
    std::vector<T> data;
    SIMD_Btree::btree<T> btree;

public:
    void prepare(std::vector<T> data_) {
        data = data_;
    }

    void build(std::vector<T> data_) {
        btree.build(data.begin(), data.end());
    }

    T next_geq(T q) {
        auto idx = btree.search(q);
        if (idx == data.size())
            return std::numeric_limits<T>::max();
        return data[idx];
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
        auto idx = bplustree.search(q);
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

#endif // SIMDBTREEINTERFACE_HPP
