#ifndef STD_VECTOR_HPP
#define STD_VECTOR_HPP

#include "index_interface.hpp"

template <typename T>
class StdVectorInterface : public IndexInterface {
private:
    std::vector<T> data;

public:
    void build(const std::vector<T>& data_) override {
        data = data_;
    }

    T access(size_t idx) const override {
        return data[idx];
    }

    std::pair<bool, T> next_geq(T q) const override {
        auto lb = std::lower_bound(data.begin(), data.end(), q);
        if (lb == data.end())
            return std::pair(false, 0);
        return std::pair(true, *lb);
    }

    size_t size_in_bytes() const override {
        return (sizeof(T) * data.size());
    }

    std::string name() const override {
        return "std_vector";
    }
};

#endif // STD_VECTOR_HPP
