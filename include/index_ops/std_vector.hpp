#ifndef STD_VECTOR_HPP
#define STD_VECTOR_HPP

#include <iostream>
#include <vector>

template <typename T>
class StdVectorInterface {
private:
    std::vector<T> data;

public:
    void prepare(const std::vector<T>& data_) {
        data = data_;
    }

    void build(const std::vector<T>& data_) {
    }

    T access(size_t idx) {
        return data[idx];
    }

    std::pair<bool, T> next_geq(T q) {
        auto lb = std::lower_bound(data.begin(), data.end(), q);
        if (lb == data.end())
            return std::pair(false, 0);
        return std::pair(true, *lb);
    }

    size_t size_in_bytes() {
        return (sizeof(T) * data.size());
    }
};

#endif // STD_VECTOR_HPP
