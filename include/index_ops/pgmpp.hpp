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
        data = std::move(data_);
    }

    void build(const std::vector<T>& data_) {
        (void)data_;
        pgmpp_idx = pgmpp::PGMIndex<T, eps, 4, true, 32, float>(data.begin(), data.end());
    }

    T next_geq(T q) {
        auto iter = pgmpp_idx.search_data(data.begin(), q);
        if (iter == data.end())
            return std::numeric_limits<T>::max();
        if (*iter < q) { // search_data by PGM++ does not actually run the lower bound, but sometimes returns the item before 
            iter++;
            if (iter == data.end())
                return std::numeric_limits<T>::max();
        }
        return *iter;
    }

    size_t size_in_bytes() {
        return pgmpp_idx.size_in_bytes();
    }

    std::string to_string() {
        return "PGM++" + std::to_string(eps);
    }
};

#endif // PGM.HPP