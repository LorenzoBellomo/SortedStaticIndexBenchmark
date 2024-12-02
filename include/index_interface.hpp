#ifndef INDEX_INTERFACE_HPP
#define INDEX_INTERFACE_HPP

#include <vector>
#include <cstdint>
#include <stdio.h>

template<typename T>
class IndexInterface {
public:
    virtual ~IndexInterface() = default;

    virtual static std::string index_name() const = 0;
    virtual static std::string parameters_stringified() const = 0;
};

#endif // INDEX_INTERFACE_HPP
