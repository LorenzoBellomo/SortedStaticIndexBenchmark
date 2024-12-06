#ifndef SDSLEF_HPP
#define SDSLEF_HPP

#include <iostream>
#include <vector>
#include <limits>
#include <sdsl/bit_vectors.hpp>
#include <sdsl/vectors.hpp>

template <typename T>
class SDSLEliasFanoInterface {
private:
    sdsl::sd_vector<> ef;
    sdsl::rank_support_sd<> ef_rank;
    sdsl::select_support_sd<> ef_select;

public:

    void prepare(std::vector<T> data_) {}

    void build(std::vector<T> data_) {
        ef = sdsl::sd_vector(data_.begin(), data_.end());
        ef_rank = sdsl::rank_support_sd<>(&ef);
        ef_select = sdsl::select_support_sd<>(&ef);
    }

    T access(size_t idx) {
        return ef_select(idx + 1);
    }

    T next_geq(T q) {
        auto rank = ef_rank(q);
        if (rank >= ef.size()) 
            return std::numeric_limits<T>::max();
        return access(rank);
    }

    size_t size_in_bytes() {
        return sdsl::size_in_bytes(ef);
    }

    std::string to_string() {
        return "EliasFano";
    }
};

#endif // SDSLEF_HPP
