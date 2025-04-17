#ifndef STATICTREE_HPP
#define STATICTREE_HPP

#include <iostream>
#include <static-search-tree.hpp>

template<typename T>
class StaticSearchTreeInterface {
private:
    std::vector<T> data;
    STree16 *stree;

public:

    ~StaticSearchTreeInterface() {
        if (stree != nullptr) {
            stree16_free(stree);
        }
        stree = nullptr;
    }
    
    void prepare(const std::vector<T>& data_) {
        data = std::move(data_);
    }

    void build(const std::vector<T>& data_) {
        (void)data_;
        stree = stree16_new(data.data(), data.size());
    }

    T next_geq(T q) {
        // CONTROLLARE COSA SUCCEDE SUGLI EDGE CASE
        return stree16_search(stree, q);
        /*if (iter == data.end())
            return std::numeric_limits<T>::max();*/
    }

    size_t size_in_bytes() {
        return stree16_size(stree);
    }

    std::string to_string() {
        return "StaticSearchTree";
    }
};

#endif // STATICTREE_HPP