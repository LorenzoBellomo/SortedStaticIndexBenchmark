#include <iostream>
#include <fstream>
#include <distr.h>
#include <algorithm>
#include <vector>
#include <cassert>
#include <zipf_gen.hpp>

#define M50 50000000
#define M1 1000000

template<class BidiIter >
BidiIter random_unique(BidiIter begin, BidiIter end, size_t num_random) {
    srand(42);
    size_t left = std::distance(begin, end);
    while (num_random--) {
        BidiIter r = begin;
        std::advance(r, rand()%left);
        std::swap(*begin, *r);
        ++begin;
        --left;
    }
    return begin;
}

std::vector<uint32_t> read_bin32_file(const std::string& filename) {
    std::ifstream ifs(filename, std::ios::binary);

    uint64_t N = 0;
    ifs.read(reinterpret_cast<char*>(&N), sizeof(uint64_t));

    std::vector<uint32_t> buf(N);
    ifs.read(reinterpret_cast<char*>(buf.data()), N * sizeof(uint32_t));
    return buf;
}

std::vector<uint64_t> read_bin64_file(const std::string& filename) {
    std::ifstream ifs(filename, std::ios::binary);

    uint64_t N = 0;
    ifs.read(reinterpret_cast<char*>(&N), sizeof(uint64_t));

    std::vector<uint64_t> buf(N);
    ifs.read(reinterpret_cast<char*>(buf.data()), N * sizeof(uint64_t));
    return buf;
}

void write_bin32_file(std::string fname, std::vector<uint32_t> data) {
    std::ofstream out(fname, std::ios_base::binary);
    uint64_t size = data.size();
    out.write(reinterpret_cast<char*>(&size), sizeof(uint64_t));
    out.write(reinterpret_cast<char*>(data.data()), size * sizeof(uint32_t));
}

void write_bin64_file(std::string fname, std::vector<uint64_t> data) {
    std::ofstream out(fname, std::ios_base::binary);
    uint64_t size = data.size();
    out.write(reinterpret_cast<char*>(&size), sizeof(uint64_t));
    out.write(reinterpret_cast<char*>(data.data()), size * sizeof(uint64_t));
}

template<typename T>
std::vector<T> generate_missing_lookups(std::vector<T> data, size_t size_) {
    std::vector<T> lookups(size_ * 10);
    std::uniform_int_distribution<T> distr(data[0], data[size_-1]);
    std::default_random_engine generator(42);
    for (auto x = 0; x < size_*10; x++) {
        lookups[x] = distr(generator); 
    }
    std::sort(lookups.begin(), lookups.end());
    std::vector<T> final_lookups;

    auto lookup_iter = lookups.begin();
    auto curr = data.begin();
    while(curr < data.end() && lookup_iter < lookups.end()) {
        if (*curr == *lookup_iter) {
            curr++;
            lookup_iter++;
        } else if (*lookup_iter < *curr) {
            final_lookups.emplace_back(*lookup_iter);
            lookup_iter++;
        } else {
            curr++;
        }
    }
    random_unique(final_lookups.begin(), final_lookups.end(), size_);
    final_lookups.resize(size_);
    return final_lookups;
}

std::vector<uint32_t> generate_uniform_distr(size_t size) {
    std::uniform_int_distribution<uint32_t> distr(0, UINT32_MAX);
    std::default_random_engine generator(0);
    std::vector<uint32_t> data(size);
    for (auto x = 0; x < size; x++) {
        data[x] = distr(generator); 
    }
    std::sort(data.begin(), data.end());
    return data;
}

std::vector<uint32_t> generate_normal_distr(size_t size) {

    std::default_random_engine generator(1);
    std::normal_distribution<double> distribution(UINT32_MAX/2,UINT32_MAX/4);
    std::vector<uint32_t> data(size);
    for (auto x = 0; x < size; x++) {
        double sample;
        do{
            sample = distribution(generator);
        } while( sample<0 || sample>(UINT32_MAX >> 1));
        data[x] = (int) sample;
    }
    std::sort(data.begin(), data.end());
    return data;
}

std::vector<uint32_t> generate_lognormal_distr(size_t size) {

    std::default_random_engine generator(2);
    std::lognormal_distribution<double> distribution(0, 0.5);
    std::vector<uint32_t> data(size);
    for (auto x = 0; x < size; x++) {
        double sample;
        do{
            sample = distribution(generator) * UINT32_MAX/5;
        } while( sample<0 || sample>(UINT32_MAX >> 1) );
        data[x] = (int) sample;
    }
    std::sort(data.begin(), data.end());
    return data;
}

std::vector<uint32_t> generate_exponential_distr(size_t size) {

    std::default_random_engine generator(3);
    std::exponential_distribution<double> distribution(2);
    std::vector<uint32_t> data(size);
    for (auto x = 0; x < size; x++) {
        double sample;
        do{
            sample = distribution(generator) * UINT32_MAX/5;
        } while( sample<0 || sample>(UINT32_MAX >> 1) );
        data[x] = (int) sample;
    }
    std::sort(data.begin(), data.end());
    return data;
}

std::vector<uint32_t> generate_chisquared_distr(size_t size) {

    std::default_random_engine generator(4);
    std::chi_squared_distribution<double> distribution(1);
    std::vector<uint32_t> data(size);
    for (auto x = 0; x < size; x++) {
        double sample;
        do{
            sample = distribution(generator) * UINT32_MAX/10;
        } while( sample<0 || sample>(UINT32_MAX >> 1) );
        data[x] = (int) sample;
    }
    std::sort(data.begin(), data.end());
    return data;
}

std::vector<uint32_t> generate_zipf_distr(size_t size) {
    std::mt19937 mt(5);  
    std::vector<uint32_t> data(size);
    zipf_distribution<uint32_t, double> distribution(UINT32_MAX/2, 0.7);
    for (auto i = 0; i < size; i++) {
        double sample;
        do{
            sample = distribution(mt);
        } while( sample<0 || sample>(UINT32_MAX >> 1) );
        data[i] = (uint32_t) sample;
    }
    std::sort(data.begin(), data.end());
    return data;
}

template<typename T>
void print_stats(std::vector<T> data) {
    std::cout << "SIZE: " << data.size() << std::endl;
    auto num_dup = 0;
    uint64_t tot_gap = 0;
    auto curr = data.begin() + 1;
    T prev_elem = data[0];
    while(curr != data.end()) {
        if (prev_elem == *curr) 
            num_dup++;
        
        assert (*curr >= prev_elem);
        tot_gap += (*curr - prev_elem);
        prev_elem = *curr;
        curr++;
    }
    std::cout << "AVERAGE GAP: " << (tot_gap / (double)(data.size() - 1)) << std::endl;
    std::cout << "# duplicates: " << num_dup << std::endl;
    std::cout << "===========================================" << std::endl;
}

int main() {

    std::vector<uint32_t> data = generate_normal_distr(M50);
    write_bin32_file("../data/normal_uint32", data);
    std::cout << "NORMAL" << std::endl;
    print_stats(data);

    data = generate_exponential_distr(M50);
    write_bin32_file("../data/exponential_uint32", data);
    std::cout << "EXPONENTIAL" << std::endl;
    print_stats(data);

    data = generate_lognormal_distr(M50);
    write_bin32_file("../data/lognormal_uint32", data);
    std::cout << "LOGNORMAL" << std::endl;
    print_stats(data);

    data = generate_lognormal_distr(M50);
    write_bin32_file("../data/zipf_uint32", data);
    std::cout << "ZIPF" << std::endl;
    print_stats(data);

    data = read_bin32_file("../data/books_200M_uint32");
    std::cout << "BOOKS" << std::endl;
    print_stats(data);

    // TODO wiki32, friendster, companynet

    std::vector<uint64_t> data64 = read_bin64_file("../data/fb_200M_uint64");
    std::cout << "FACEBOOK 64" << std::endl;
    print_stats(data64);

    data64 = read_bin64_file("../data/books_800M_uint64");
    std::cout << "BOOKS 64" << std::endl;
    print_stats(data64);

    data64 = read_bin64_file("../data/osm_cellids_800M_uint64");
    std::cout << "OSM CELLIDS 64" << std::endl;
    print_stats(data64);

    data64 = read_bin64_file("../data/wiki_ts_200M_uint64");
    std::cout << "WIKI 64" << std::endl;
    print_stats(data64);


    return 0;
}