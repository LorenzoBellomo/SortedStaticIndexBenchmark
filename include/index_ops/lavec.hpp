#ifndef LAVEC_HPP
#define LAVEC_HPP

#include <iostream>
#include <vector>
#include <limits>
#include "la_vector.hpp"

template <typename T, size_t bpc>
class LaVectorInterface {
private:
    la_vector<T, bpc> v1;

public:
    void prepare(std::vector<T> data_) {}

    void build(std::vector<T> data_) {
        v1 = la_vector<T, bpc>(data_);
    }

    T access(size_t idx) {
        v1.select(idx);
    }

    T next_geq(T q) {
        auto lb = v1.lower_bound(q);
        if (lb == v1.end())
            return std::numeric_limits<T>::max();
        return *lb;
    }

    size_t size_in_bytes() {
        return v1.size_in_bytes();
    }

    std::string to_string() {
        return "LA-vector" + std::to_string(bpc);
    }
};

template <typename T>
class LaVectorOptInterface {
private:
    la_vector_opt<T> v1;

public:
    void prepare(std::vector<T> data_) {}

    void build(std::vector<T> data_) {
        v1 = la_vector_opt<T>(data_);
    }

    T access(size_t idx) {
        v1.select(idx);
    }

    T next_geq(T q) {
        auto lb = v1.lower_bound(q);
        if (lb == v1.end())
            return std::numeric_limits<T>::max();
        return *lb;
    }

    size_t size_in_bytes() {
        return v1.size_in_bytes();
    }

    std::string to_string() {
        return "LA-vectoropt";
    }
};

#endif // LAVEC_HPP
