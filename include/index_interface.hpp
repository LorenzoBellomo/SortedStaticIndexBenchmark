#ifndef INDEX_INTERFACE_HPP
#define INDEX_INTERFACE_HPP

#include <vector>
#include <cstdint>
#include <stdio.h>

class IndexInterface {
public:
    virtual ~IndexInterface() = default;

    virtual void build(const std::vector<uint32_t>& data) = 0;
    virtual void build(const std::vector<uint64_t>& data) = 0;

    virtual uint32_t access32(size_t idx) const = 0;
    virtual uint64_t access64(size_t idx) const = 0;
};

#endif // INDEX_INTERFACE_HPP
