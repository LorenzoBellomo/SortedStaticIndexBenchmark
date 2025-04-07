#ifndef FASTINTERFACE_HPP
#define FASTINTERFACE_HPP

#include <iostream>
#include <vector>
#include <limits>
#include <fast.h>

template <typename T>
class FASTInterface {
private:
    FAST<T> fast_tree;
    std::vector<T> data;

public:
    void prepare(std::vector<T> data_) {
        data = std::move(data_);
    }

    void build(std::vector<T> data_) {
        fast_tree.buildFAST(data_.data(), data_.size());
    }

    T next_geq(T q) {
        size_t x = fast_tree.lower_bound(q);
        if (x == data.size())
            return std::numeric_limits<T>::max();
        return data[x];
    }

    size_t size_in_bytes() {
        return (fast_tree.size_in_bytes());
    }

    std::string to_string() {
        return "FAST";
    }
};

#endif // FASTINTERFACE_HPP
