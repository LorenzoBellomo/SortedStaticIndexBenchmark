#pragma once

#include <iostream>
#include <fstream>
#include <vector>
#include <numeric>
#include <random>

#define M1 1000000

template<class BidiIter >
BidiIter random_unique(BidiIter begin, BidiIter end, size_t num_random, int seed) {
    srand(seed);
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

template <typename T>
std::vector<T> LoadDataset(const std::string& path_to_file) {
    std::ifstream ifs(path_to_file, std::ios::binary);
    if (!ifs.good()) {
        std::cerr << "Error opening file: " << path_to_file << std::endl;
        throw std::runtime_error("File not found");
    }
    uint64_t N = 0;
    ifs.read(reinterpret_cast<char*>(&N), sizeof(uint64_t));
    std::vector<T> buf(N);
    ifs.read(reinterpret_cast<char*>(buf.data()), N * sizeof(T));
    return buf;
}

template <typename T>
std::vector<T> GenerateLookups(std::vector<T> data, int seed = 42) {
    std::vector<T> lookups(data);
    random_unique(lookups.begin(), lookups.end(), M1, seed);
    lookups.resize(M1);
    return lookups;
}

template <typename T>
std::vector<size_t> GenerateScanLookups(std::vector<T> data, size_t scan_size, int seed = 42) {
    std::vector<size_t> lookups(data.size() - scan_size); 
    std::iota(std::begin(lookups), std::end(lookups), 0);
    random_unique(lookups.begin(), lookups.end(), 100000, seed);
    lookups.resize(100000);
    return lookups;
}

template<typename T>
std::vector<std::pair<size_t, T>> generate_missing_lookups(std::vector<T> data, size_t size_, int seed = 42) {
    std::vector<T> lookups(size_ * 10);
    std::uniform_int_distribution<T> distr(data[0] + 1, data[data.size() - 1] - 1);
    std::default_random_engine generator(seed);
    for (auto x = 0; x < size_*10; x++) {
        lookups[x] = distr(generator); 
    }
    std::sort(lookups.begin(), lookups.end());
    std::vector<std::pair<size_t, T>> final_lookups;

    auto lookup_iter = lookups.begin();
    auto curr = data.begin();
    while(curr < data.end() && lookup_iter < lookups.end()) {
        if (*curr == *lookup_iter) {
            auto prev = *curr;
            while (prev == *curr) {
                curr++;
            } 
            prev = *lookup_iter;
            while (prev == *lookup_iter++) {
                lookup_iter++;
            } 
        } else if (*lookup_iter < *curr) {
            size_t idx = curr - data.begin();
            final_lookups.emplace_back(std::make_pair(idx, *lookup_iter));
            lookup_iter++;
        } else {
            curr++;
        }
    }
    random_unique(final_lookups.begin(), final_lookups.end(), size_, seed);
    final_lookups.resize(size_);
    return final_lookups;
}