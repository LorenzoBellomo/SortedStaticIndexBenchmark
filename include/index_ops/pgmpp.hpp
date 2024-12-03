#ifndef PGMPP_HPP
#define PGMPP_HPP

#include <iostream>
#include <pgmpp/pgm_index.h>

template<typename T, int eps>
class PGMPPInterface {
private:
    std::vector<T> data;
    pgmpp::PGMIndex<T, eps, 4, true, 32, float> pgmpp_idx;

public:
    void prepare(const std::vector<T>& data_) {
        data = data_;
    }

    void build(const std::vector<T>& data_) {
        pgmpp_idx = pgmpp::PGMIndex<T, eps, 4, true, 32, float>(data_.begin(), data_.end());
    }

    std::pair<bool, T> next_geq(T q) {
        auto iter = pgmpp_idx.search_data(data.begin(), q);
        if (iter == data.end())
            return std::pair(false, 0);
        return std::pair(true, *iter);
    }

    size_t size_in_bytes() {
        return pgmpp_idx.size_in_bytes();
    }
};

#endif // PGM.HPP