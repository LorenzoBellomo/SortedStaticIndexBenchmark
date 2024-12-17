#ifndef GAMMADELTA_HPP
#define GAMMADELTA_HPP

#include <iostream>
#include <vector>
#include <limits>
#include <wrapper_sdsl.hpp>

template <typename T, size_t dens>
class GammaInterface {
private:
    wrapper_sdsl_enc_vector<sdsl::enc_vector_rank<sdsl::coder::elias_gamma_rank, dens>, std::vector<T>> v_gamma;

public:
    void prepare(std::vector<T> data_) {}

    void build(std::vector<T> data_) {
        v_gamma = wrapper_sdsl_enc_vector<sdsl::enc_vector_rank<sdsl::coder::elias_gamma_rank, dens>, std::vector<T>>(data_);
    }

    T access(size_t idx) {
        return v_gamma.select(idx + 1);
    }

    T next_geq(T q) {
        auto rank = v_gamma.rank(q) + 1;
        if (rank > v_gamma.size())
            return std::numeric_limits<T>::max();
        return v_gamma.select(rank);
    }

    size_t size_in_bytes() {
        return sdsl::size_in_bytes(v_gamma);
    }

    std::string to_string() {
        return "GammaCode" + std::to_string(dens);
    }
};

template <typename T, size_t dens>
class DeltaInterface {
private:
    wrapper_sdsl_enc_vector<sdsl::enc_vector_rank<sdsl::coder::elias_delta_rank, dens>, std::vector<T>> v_delta;

public:
    void prepare(std::vector<T> data_) {}

    void build(std::vector<T> data_) {
        v_delta = wrapper_sdsl_enc_vector<sdsl::enc_vector_rank<sdsl::coder::elias_delta_rank, dens>, std::vector<T>>(data_);
    }

    T access(size_t idx) {
        return v_delta.select(idx + 1);
    }

    T next_geq(T q) {
        auto rank = v_delta.rank(q) + 1;
        if (rank > v_delta.size())
            return std::numeric_limits<T>::max();
        return v_delta.select(rank);
    }

    size_t size_in_bytes() {
        return sdsl::size_in_bytes(v_delta);
    }

    std::string to_string() {
        return "DeltaCode" + std::to_string(dens);
    }
};

template <typename T, size_t block_size>
class RRRInterface {
private:
    wrapper_sdsl<T, sdsl::rrr_vector<block_size>> rrr;
public:
    void prepare(std::vector<T> data_) {}

    void build(std::vector<T> data_) {
        rrr.build(data_);
    }

    T access(size_t idx) {
        return rrr.select(idx + 1);
    }

    T next_geq(T q) {
        auto rank = rrr.rank(q) + 1;
        if (rank > rrr.size())
            return std::numeric_limits<T>::max();
        return rrr.select(rank);
    }

    size_t size_in_bytes() {
        return (rrr.bits_per_element()*rrr.size()) / 8;
    }

    std::string to_string() {
        return "RRR-Vector" + std::to_string(block_size);
    }
};

#endif // GAMMADELTA_HPP