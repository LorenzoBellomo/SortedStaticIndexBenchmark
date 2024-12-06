#ifndef PGM_HPP
#define PGM_HPP

#include <iostream>
#include <pgm/pgm_index.hpp>

template<typename T, int eps>
class PGMInterface {
private:
    std::vector<T> data;
    pgm::PGMIndex<T, eps> pgm_idx;

public:
    void prepare(const std::vector<T>& data_) {
        data = data_;
    }

    void build(const std::vector<T>& data_) {
        pgm_idx = pgm::PGMIndex<T, eps>(data_.begin(), data_.end());
    }

    T next_geq(T q) {
        auto range = pgm_idx.search(q);
        auto lb = std::lower_bound(data.begin() + range.lo, data.begin() + range.hi, q);
        if (lb == data.end())
            return std::numeric_limits<T>::max();
        return *lb;
    }

    size_t size_in_bytes() {
        return pgm_idx.size_in_bytes();
    }

    std::string to_string() {
        return "PGM" + std::to_string(eps);
    }
};

#endif // PGM.HPP