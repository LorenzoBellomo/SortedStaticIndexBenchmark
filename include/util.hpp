#pragma once

#include <iostream>
#include <fstream>
#include <vector>
#include <numeric>

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