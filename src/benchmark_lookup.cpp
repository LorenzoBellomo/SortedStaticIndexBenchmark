#include <benchmark/benchmark.h>
#include <iostream>
#include <json.hpp>
#include <util.hpp>
#include <index_interface.hpp>

// Load JSON and datasets dynamically
void BenchmarkLookup(benchmark::State& state, const std::string& index_name, IndexInterface* index) {
    std::vector<uint32_t> data = LoadDataset<uint32_t>("data/companynet_uint32");
    index->build(data);
    for (auto _ : state) {
        benchmark::DoNotOptimize(index->access32(state.range(0)));
    }
    delete index;
}

int main(int argc, char** argv) {
    // Load configuration
    auto configs = LoadConfig("../config.json");

    // Register benchmarks
    for (const auto& config : configs) {
        std::cout << config["name"] << std::endl;
        std::cout << config["parameters"] << std::endl;
        std::string name = config["name"];
        std::vector<std::string> params(config["parameters"]);
        IndexInterface* idx = CreateIndex(name, params);
        benchmark::RegisterBenchmark(name, BenchmarkLookup, name, idx);
    }
    benchmark::Initialize(&argc, argv);
    benchmark::RunSpecifiedBenchmarks();
}