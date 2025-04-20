#ifndef RMIINTERFACE_HPP
#define RMIINTERFACE_HPP

#include <iostream>
#include <vector>
#include <limits>
#include <string>

#include "rmi.h"

/*
0 = wiki_ts_200M_uint64
1 = lognormal_uint32
2 = fb_200M_uint64
3 = companynet_uint32
4 = normal_uint32
5 = wiki_ts_200M_uint32
6 = zipf_uint32
7 = books_800M_uint64
8 = exponential_uint32
9 = friendster_50M_uint32
10 = osm_cellids_800M_uint64
11 = books_200M_uint32
12 = books_50M_uint64
13 = normal_800M_uint32
*/
template <typename T, int dataset, bool compact>
class RMIInterface {
private:
    std::vector<T> data;
    bool built;

public:
    RMIInterface() : built(false) {}

    ~RMIInterface() {
        clear();
    }

    void prepare(std::vector<T>& data_) {
        data = std::move(data_);
    }

    void build(std::vector<T> data_) {
        (void)data_;

        clear();
        
        std::string model_path = "../indices/RMI/rmi_data";
        switch(dataset) {
            case 0:
                if (compact) 
                    built = wiki64_compact::load(model_path.c_str());
                else
                    built = wiki64_large::load(model_path.c_str());
                break;
            case 1:
                if (compact) 
                    built = lognormal_compact::load(model_path.c_str());
                else
                    built = lognormal_large::load(model_path.c_str());
                break;
            case 2:
                if (compact) 
                    built = fb64_compact::load(model_path.c_str());
                else
                    built = fb64_large::load(model_path.c_str());
                break;
            case 3:
                if (compact) 
                    built = companynet_compact::load(model_path.c_str());
                else
                    built = companynet_large::load(model_path.c_str());
                break;
            case 4:
                if (compact) 
                    built = normal_compact::load(model_path.c_str());
                else
                    built = normal_large::load(model_path.c_str());
                break;
            case 5:
                if (compact) 
                    built = wiki_compact::load(model_path.c_str());
                else
                    built = wiki_large::load(model_path.c_str());
                break;
            case 6:
                if (compact) 
                    built = zipf_compact::load(model_path.c_str());
                else
                    built = zipf_large::load(model_path.c_str());
                break;
            case 7:
                if (compact) 
                    built = books64_compact::load(model_path.c_str());
                else
                    built = books64_large::load(model_path.c_str());
                break;
            case 8:
                if (compact) 
                    built = exponential_compact::load(model_path.c_str());
                else
                    built = exponential_large::load(model_path.c_str());
                break;
            case 9:
                if (compact) 
                    built = friendster_compact::load(model_path.c_str());
                else
                    built = friendster_large::load(model_path.c_str());
                break;
            case 10:
                if (compact) 
                    built = osm64_compact::load(model_path.c_str());
                else
                    built = osm64_large::load(model_path.c_str());
                break;
            case 11:
                if (compact) 
                    built = books_compact::load(model_path.c_str());
                else
                    built = books_large::load(model_path.c_str());
                break;
            case 12:
                if (compact) 
                    built = books64small_compact::load(model_path.c_str());
                else
                    built = books64small_large::load(model_path.c_str());
                break;
            case 13:
                if (compact) 
                    built = normal800_compact::load(model_path.c_str());
                else
                    built = normal800_large::load(model_path.c_str());
                break;
            default: 
                throw std::invalid_argument("Unknown dataset");
        }
    }

    // TODO: this is wrong because it assumes one and only one build() call
    void clear() {
        if (!built) {
            return;
        }
        built = false;

        switch(dataset) {
            case 0:
                if (compact) 
                    wiki64_compact::cleanup();
                else
                    wiki64_large::cleanup();
                break;
            case 1:
                if (compact) 
                    lognormal_compact::cleanup();
                else
                    lognormal_large::cleanup();
                break;
            case 2:
                if (compact) 
                    fb64_compact::cleanup();
                else
                    fb64_large::cleanup();
                break;
            case 3:
                if (compact) 
                    companynet_compact::cleanup();
                else
                    companynet_large::cleanup();
                break;
            case 4:
                if (compact) 
                    normal_compact::cleanup();
                else
                    normal_large::cleanup();
                break;
            case 5:
                if (compact) 
                    wiki_compact::cleanup();
                else
                    wiki_large::cleanup();
                break;
            case 6:
                if (compact) 
                    zipf_compact::cleanup();
                else
                    zipf_large::cleanup();
                break;
            case 7:
                if (compact) 
                    books64_compact::cleanup();
                else
                    books64_large::cleanup();
                break;
            case 8:
                if (compact) 
                    exponential_compact::cleanup();
                else
                    exponential_large::cleanup();
                break;
            case 9:
                if (compact) 
                    friendster_compact::cleanup();
                else
                    friendster_large::cleanup();
                break;
            case 10:
                if (compact) 
                    osm64_compact::cleanup();
                else
                    osm64_large::cleanup();
                break;
            case 11:
                if (compact) 
                    books_compact::cleanup();
                else
                    books_large::cleanup();
                break;
            case 12:
                if (compact) 
                    books64small_compact::cleanup();
                else
                    books64small_large::cleanup();
                break;
            case 13:
                if (compact) 
                    normal800_compact::cleanup();
                else
                    normal800_large::cleanup();
                break;
            default: 
                throw std::invalid_argument("Unknown dataset");
        }
    }

    T next_geq(T q) {
        size_t err;
        T x;
        switch(dataset) {
            case 0:
                if (compact) 
                    x = wiki64_compact::lookup(q, &err);
                else
                    x = wiki64_large::lookup(q, &err);
                break;
            case 1:
                if (compact) 
                    x = lognormal_compact::lookup(q, &err);
                else
                    x = lognormal_large::lookup(q, &err);
                break;
            case 2:
                if (compact) 
                    x = fb64_compact::lookup(q, &err);
                else
                    x = fb64_large::lookup(q, &err);
                break;
            case 3:
                if (compact) 
                    x = companynet_compact::lookup(q, &err);
                else
                    x = companynet_large::lookup(q, &err);
                break;
            case 4:
                if (compact) 
                    x = normal_compact::lookup(q, &err);
                else
                    x = normal_large::lookup(q, &err);
                break;
            case 5:
                if (compact) 
                    x = wiki_compact::lookup(q, &err);
                else
                    x = wiki_large::lookup(q, &err);
                break;
            case 6:
                if (compact) 
                    x = zipf_compact::lookup(q, &err);
                else
                    x = zipf_large::lookup(q, &err);
                break;
            case 7:
                if (compact) 
                    x = books64_compact::lookup(q, &err);
                else
                    x = books64_large::lookup(q, &err);
                break;
            case 8:
                if (compact) 
                    x = exponential_compact::lookup(q, &err);
                else
                    x = exponential_large::lookup(q, &err);
                break;
            case 9:
                if (compact) 
                    x = friendster_compact::lookup(q, &err);
                else
                    x = friendster_large::lookup(q, &err);
                break;
            case 10:
                if (compact) 
                    x = osm64_compact::lookup(q, &err);
                else
                    x = osm64_large::lookup(q, &err);
                break;
            case 11:
                if (compact) 
                    x = books_compact::lookup(q, &err);
                else
                    x = books_large::lookup(q, &err);
                break;
            case 12:
                if (compact) 
                    books64small_compact::lookup(q, &err);
                else
                    books64small_large::lookup(q, &err);
                break;
            case 13:
                if (compact) 
                    normal800_compact::lookup(q, &err);
                else
                    normal800_large::lookup(q, &err);
                break;
            default: 
                throw std::invalid_argument("Unknown dataset");
        }
        long diff_inf = (long) x - (long) err;
        auto begin_index = std::max(diff_inf, (long) 0);
        auto end_idx = std::min(x + err, data.size());
        auto lb = std::lower_bound(data.begin() + begin_index, data.begin() + end_idx, q);
        if (lb == data.end())
            return std::numeric_limits<T>::max();
        return *lb;
    }

    size_t size_in_bytes() {
        switch(dataset) {
            case 0:
                if (compact) 
                    return wiki64_compact::RMI_SIZE;
                else
                    return wiki64_large::RMI_SIZE;
                break;
            case 1:
                if (compact) 
                    return lognormal_compact::RMI_SIZE;
                else
                    return lognormal_large::RMI_SIZE;
                break;
            case 2:
                if (compact) 
                    return fb64_compact::RMI_SIZE;
                else
                    return fb64_large::RMI_SIZE;
                break;
            case 3:
                if (compact) 
                    return companynet_compact::RMI_SIZE;
                else
                    return companynet_large::RMI_SIZE;
                break;
            case 4:
                if (compact) 
                    return normal_compact::RMI_SIZE;
                else
                    return normal_large::RMI_SIZE;
                break;
            case 5:
                if (compact) 
                    return wiki_compact::RMI_SIZE;
                else
                    return wiki_large::RMI_SIZE;
                break;
            case 6:
                if (compact) 
                    return zipf_compact::RMI_SIZE;
                else
                    return zipf_large::RMI_SIZE;
                break;
            case 7:
                if (compact) 
                    return books64_compact::RMI_SIZE;
                else
                    return books64_large::RMI_SIZE;
                break;
            case 8:
                if (compact) 
                    return exponential_compact::RMI_SIZE;
                else
                    return exponential_large::RMI_SIZE;
                break;
            case 9:
                if (compact) 
                    return friendster_compact::RMI_SIZE;
                else
                    return friendster_large::RMI_SIZE;
                break;
            case 10:
                if (compact) 
                    return osm64_compact::RMI_SIZE;
                else
                    return osm64_large::RMI_SIZE;
                break;
            case 11:
                if (compact) 
                    return books_compact::RMI_SIZE;
                else
                    return books_large::RMI_SIZE;
                break;
            case 12:
                if (compact) 
                    books64small_compact::RMI_SIZE;
                else
                    books64small_large::RMI_SIZE;
                break;
            case 13:
                if (compact) 
                    normal800_compact::RMI_SIZE;
                else
                    normal800_large::RMI_SIZE;
                break;
            default: 
                throw std::invalid_argument("Unknown dataset");
        }
    }

    std::string to_string() {
        std::string to_ret = (compact)? "RMI-compact" : "RMI-large";
        return to_ret;
    }
};

#endif // RMIINTERFACE_HPP
