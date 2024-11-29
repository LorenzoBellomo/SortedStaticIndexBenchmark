#include <benchmark/benchmark.h>
#include <iostream>
#include <json.hpp>
#include <util.hpp>
#include <index_interface.hpp>

// Load JSON and datasets dynamically
void BenchmarkBuild(benchmark::State& state, const std::string& index_name, IndexInterface* index) {
    std::vector<uint32_t> data = LoadDataset<uint32_t>("data/companynet_uint32");
    for (auto _ : state) {
        index->build(data);
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
        benchmark::RegisterBenchmark(name, BenchmarkBuild, name, idx);
    }
    benchmark::Initialize(&argc, argv);
    benchmark::RunSpecifiedBenchmarks();
}