#ifndef PGM_HPP
#define PGM_HPP

#include "index_interface.hpp"
#include <pgm/pgm_index.hpp>

template<typename T, size_t eps>
class PGMinterface : public IndexInterface<T> {
private:
    std::vector<T> data;
    pgm::PGMIndex<T, eps> pgm_idx;

public:
    void build(const std::vector<T>& data_) override {
        data = data_;
        pgm_idx = pgm::PGMIndex<T, eps>(data.begin(), data.end()-1);
    }

    std::pair<bool, T> next_geq(T q) const override {
        auto range = pgm_idx.search(q);
        auto lb = std::lower_bound(data.begin() + range.lo, data.begin() + range.hi, q);
        if (lb == data.end())
            return std::pair(false, 0);
        return std::pair(true, *lb);
    }

    size_t size_in_bytes() const override {
        return pgm_idx.size_in_bytes();
    }
};

#endif // PGM.HPP