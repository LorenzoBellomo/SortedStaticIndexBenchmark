#ifndef STD_VECTOR_HPP
#define STD_VECTOR_HPP

#include "index_interface.hpp"

class StdVector : public IndexInterface {
private:
    std::vector<uint32_t> data32;
    std::vector<uint64_t> data64;

public:
    void build(const std::vector<uint32_t>& data) override {
        data32 = data;
    }

    void build(const std::vector<uint64_t>& data) override {
        data64 = data;
    }

    uint32_t access32(size_t idx) const override {
        return data32[idx];
    }

    uint64_t access64(size_t idx) const override {
        return data64[idx];
    }
};

#endif // STD_VECTOR_HPP
