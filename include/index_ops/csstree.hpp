#ifndef CSSINTERFACE_HPP
#define CSSINTERFACE_HPP

#include <iostream>
#include <vector>
#include <limits>
#include <csstree.hpp>

template <typename T, size_t node_size>
class CSSInterface {
private:
    CSSTree<node_size, T> css;

public:
    void prepare(std::vector<T> data_) {}

    void build(std::vector<T> data_) {
        css = CSSTree<node_size, T>(data_);
    }

    T next_geq(T q) {
        auto x = css.find(q);
        if (x == css.end())
            return std::numeric_limits<T>::max();
        return *x;
    }

    size_t size_in_bytes() {
        return (css.size_in_bytes());
    }

    std::string to_string() {
        return "CSS-Btree";
    }
};

#endif // CSSINTERFACE_HPP
