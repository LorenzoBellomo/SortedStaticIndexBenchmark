#ifndef ROARINGINTERFACE_HPP
#define ROARINGINTERFACE_HPP

#include <iostream>
#include <vector>
#include <limits>
#include "roaring.h"
#include "roaring.hh"
#include "roaring64map.hh"

template <typename T>
class RoaringInterface {
private:
    roaring::Roaring r32;
    roaring::Roaring64Map r64;

public:
    void prepare(std::vector<T> data_) {}

    void build(std::vector<T> data_) {
        if constexpr(sizeof(T) == 4) {
            for (auto x : data_) 
                r32.add(x);
            r32.runOptimize();
        } else {
            for (auto x : data_) 
                r64.add(x);
            r64.runOptimize();
        }
    }

    T access(size_t idx) {
        T elem; 
        if constexpr(sizeof(T) == 4) 
            r32.select(idx, &elem);
         else 
            r64.select(idx, &elem);
        return elem;
    }

    T next_geq(T q) {
        if constexpr(sizeof(T) == 4) {
            auto iter = r32.begin();
            iter.equalorlarger(q);
            return *iter;
        }
        auto iter = r64.begin();
        iter.move(q);
        return *iter;
    }

    size_t size_in_bytes() {
        if constexpr(sizeof(T) == 4) 
            return r32.getSizeInBytes();
        return r64.getSizeInBytes();
    }

    std::string to_string() {
        return "Roaring";
    }
};

#endif // ROARINGINTERFACE_HPP
