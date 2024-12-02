#include <benchmark/benchmark.h>
#include <iostream>
#include <json.hpp>
#include <util.hpp>
#include <index_interface.hpp>

// Load JSON and datasets dynamically
template <typename T>
void BenchmarkBuild(benchmark::State& state, std::string dataset_name, IndexInterface<T>* index) {
    std::vector<T> data = LoadDataset<T>("data/" + dataset_name);
    for (auto _ : state) {
        index->build(data);
    }
    delete index;
}

template <class IDX> void BenchmarkBuild(benchmark::State& state, std::string dataset_name) {
    if (dataset.back() == '2') { // 32 bit
        std::vector<T> data = LoadDataset<T>("data/" + dataset_name);
    else 
        for (auto _ : state) {
    for (int i = state.range(0); i-=step; )
    q.push(v);
    for (int e = state.range(0); e-=step; )
    q.Wait(&v);
    }
    // actually messages, not bytes:
    state.SetBytesProcessed(
    static_cast<int64_t>(state.iterations())*state.range(0));
}

int main(int argc, char** argv) {
    // Load configuration
    auto configs = LoadConfigJSON("../config.json");
    /*std::vector<std::string> datasets({"books_200M_uint32", "companynet_uint32", "fb_200M_uint64", "lognormal_uint32", "normal_uint32",            wiki_ts_200M_uint32  zipf_uint32
           "books_800M_uint64", "exponential_uint32", "friendster_50M_uint32", "osm_cellids_800M_uint64", "wiki_ts_200M_uint64"});*/
    std::vector<std::string> datasets({"lognormal_uint32", "companynet_uint32"});

    // Register benchmarks
    for (const auto& config : configs) {
        std::cout << config["name"] << std::endl;
        std::cout << config["parameters"] << std::endl;
        std::string idx_name = config["name"];
        std::vector<std::vector<std::string>> params(config["parameters"]);
        for (auto dataset : datasets) {
            for (auto param : params) { // POTREBBE ESSERE VUOTO
                std::string bm_name = dataset + "_" + idx_name + "_" + PARAMETRI DA SERIALIZZARE;
                if (dataset.back() == '2') { // 32 bit
                    auto idx = CreateIndex<uint32_t>(idx_name, params);
                    benchmark::RegisterBenchmark(bm_name, BenchmarkBuild<uint32_t>, dataset, idx);
                } else if (! config["exclude_64_bits_experiments"]) {
                    auto idx = CreateIndex<uint64_t>(idx_name, params);
                    benchmark::RegisterBenchmark(bm_name, BenchmarkBuild<uint64_t>, dataset, idx);
                }
            }
        }
    }
    benchmark::Initialize(&argc, argv);
    benchmark::RunSpecifiedBenchmarks();
}