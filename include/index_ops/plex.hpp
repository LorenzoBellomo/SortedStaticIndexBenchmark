#ifndef PLEXINTERFACE_HPP
#define PLEXINTERFACE_HPP

#include <iostream>
#include <vector>
#include <limits>

#include "ts/builder.h"
#include "ts/common.h"

template <typename T, size_t spline_max_error>
class PLEXInterface {
private:
    ts::TrieSpline<T> ts;
    std::vector<T> data;

public:
    void prepare(std::vector<T> data_) {
        data = data_;
    }

    void build(std::vector<T> data_) {
        ts::Builder<T> tsb(data_[0], data_[data_.size() - 1], spline_max_error);
        for (const auto& key : data_) tsb.AddKey(key);
        ts = tsb.Finalize();
    }

    T next_geq(T q) {
        ts::SearchBound bound = ts.GetSearchBound(q);
        auto start = data.begin() + bound.begin;
        auto last = data.begin() + bound.end;
        auto lb = std::lower_bound(start, last, q);
        if (lb == data.end())
            return std::numeric_limits<T>::max();
        if (*lb < q) { // SOMETIME PLEX DOES NOT RETURN THE CORRECT SEARCH BOUND, so in this case check until data.end()
            lb = std::lower_bound(start, data.end(), q); 
            if (lb == data.end())
                return std::numeric_limits<T>::max();
        }
        return *lb;
    }

    size_t size_in_bytes() {
        return ts.GetSize();
    }

    std::string to_string() {
        return "PLEX" + std::to_string(spline_max_error);
    }
};

#endif // PLEXINTERFACE_HPP
