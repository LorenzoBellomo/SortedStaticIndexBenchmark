#ifndef ALEXINTER_HPP
#define ALEXINTER_HPP

#include <iostream>
#include <vector>
#include <limits>
#include <alex.h>

template <typename T>
class ALEXInterface {
private:
    alex::Alex<T, uint8_t> alex_;
    std::pair<T, uint8_t>* for_alex;

public:
    void prepare(std::vector<T> data_) {
        for_alex = new std::pair<T, uint8_t>[data_.size()];
        for (size_t i = 0; i < data_.size(); i++) {
            for_alex[i].first = data_[i];
            for_alex[i].second = static_cast<uint8_t>(0);
        }
    }

    void build(std::vector<T> data_) {
        alex_.bulk_load(for_alex, data_.size());
    }

    T next_geq(T q) {
        auto iter = alex_.lower_bound(q);
        if (iter == alex_.end())
            return std::numeric_limits<T>::max();
        return (*iter).first;
    }

    size_t size_in_bytes() {
        return alex_.model_size();
    }

    std::string to_string() {
        return "ALEX";
    }
};

#endif // ALEXINTER_HPP
