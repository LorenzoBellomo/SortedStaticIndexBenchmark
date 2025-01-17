#ifndef STD_VECTOR_HPP
#define STD_VECTOR_HPP

#include <iostream>
#include <vector>
#include <limits>

template <typename T>
class StdVectorInterface {
private:
    std::vector<T> data;

public:
    void prepare(std::vector<T>& data_) {
        data = std::move(data_);
    }

    void build(std::vector<T> data_) {}

    T access(size_t idx) {
        return data[idx];
    }

    T next_geq(T q) {
        auto lb = std::lower_bound(data.begin(), data.end(), q);
        if (lb == data.end())
            return std::numeric_limits<T>::max();
        return *lb;
    }

    size_t size_in_bytes() {
        return (sizeof(T) * data.size());
    }

    std::string to_string() {
        return "std::vector";
    }
};

#endif // STD_VECTOR_HPP
