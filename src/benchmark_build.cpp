#include <benchmark/benchmark.h>
#include <iostream>
#include <fstream>
#include <index_ops/std_vector.hpp>
#include <index_ops/pgm.hpp>

template <typename T>
std::vector<T> LoadDataset(const std::string& path_to_file) {
    std::ifstream ifs(path_to_file, std::ios::binary);
    uint64_t N = 0;
    ifs.read(reinterpret_cast<char*>(&N), sizeof(uint64_t));
    std::vector<T> buf(N);
    ifs.read(reinterpret_cast<char*>(buf.data()), N * sizeof(T));
    return buf;
}

template <class IDX, typename T>
void BenchmarkBuild(benchmark::State& state, std::string dataset_name) {
    std::vector<T> data = LoadDataset<T>("data/" + dataset_name);
    IDX idx;
    for (auto _ : state) {
        idx.build(data);
    }
}

template <typename T>
std::vector<T> read_bin_file(const std::string& path_to_file) {
    std::ifstream ifs(path_to_file, std::ios::binary);
    uint64_t N = 0;
    ifs.read(reinterpret_cast<char*>(&N), sizeof(uint64_t));
    std::vector<T> buf(N);
    ifs.read(reinterpret_cast<char*>(buf.data()), N * sizeof(T));
    return buf;
}

int main(int argc, char** argv) {
    std::vector<std::string> datasets({"books_200M_uint32", "companynet_uint32", "fb_200M_uint64", "lognormal_uint32", 
    "normal_uint32", "wiki_ts_200M_uint32", "zipf_uint32", "books_800M_uint64", "exponential_uint32", "friendster_50M_uint32", 
    "osm_cellids_800M_uint64", "wiki_ts_200M_uint64"});
    
    // Register benchmarks
    for (auto dataset : datasets) {
        // registering benchmarks for different datasets, and all the indexes present in index_ops
        benchmark::RegisterBenchmark(dataset+"_std::vector", BenchmarkBuild<StdVectorInterface<uint32_t>, uint32_t>, dataset);
        benchmark::RegisterBenchmark(dataset+"_PGM8", BenchmarkBuild<PGMInterface<uint32_t, 8>, uint32_t>, dataset);
        benchmark::RegisterBenchmark(dataset+"_PGM32", BenchmarkBuild<PGMInterface<uint32_t, 32>, uint32_t>, dataset);
        if (dataset.back() == '2')  { // HERE REGISTERING ONLY THE ONES THAT WORK FOR 32 BIT
            benchmark::RegisterBenchmark(dataset+"_PGM128", BenchmarkBuild<PGMInterface<uint32_t, 128>, uint32_t>, dataset);
        }
    }
    benchmark::Initialize(&argc, argv);
    benchmark::RunSpecifiedBenchmarks();
}
