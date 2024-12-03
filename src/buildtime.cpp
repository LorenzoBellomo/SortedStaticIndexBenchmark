#include <benchmark/benchmark.h>
#include <iostream>
#include <fstream>
#include <index_ops/std_vector.hpp>
#include <index_ops/pgm.hpp>
#include <index_ops/pgmpp.hpp>
#include <util.hpp>

template <class IDX, typename T>
void Benchmark(benchmark::State& state, std::string dataset_name) {
    std::vector<T> data = LoadDataset<T>("../data/" + dataset_name);
    IDX idx;
    idx.prepare(data);
    for (auto _ : state) {
        idx.build(data);
        benchmark::DoNotOptimize(idx);
    }
}

int main(int argc, char** argv) {
    std::vector<std::string> datasets({"fb_200M_uint64", "books_200M_uint32", "lognormal_uint32", "companynet_uint32",
    "normal_uint32", "wiki_ts_200M_uint32", "zipf_uint32", "books_800M_uint64", "exponential_uint32", "friendster_50M_uint32", 
    "osm_cellids_800M_uint64", "wiki_ts_200M_uint64"});
    
    // Register benchmarks
    for (auto dataset : datasets) {
        // registering benchmarks for different datasets, and all the indexes present in index_ops
        if (dataset.back() == '2')  { // HERE REGISTERING ONLY THE ONES THAT WORK FOR 32 BIT
            //benchmark::RegisterBenchmark(dataset+"_std::vector", Benchmark<StdVectorInterface<uint32_t>, uint32_t>, dataset)->Iterations(10);
            benchmark::RegisterBenchmark(dataset+"_PGM8", Benchmark<PGMInterface<uint32_t, 8>, uint32_t>, dataset)->Iterations(10);
            benchmark::RegisterBenchmark(dataset+"_PGM32", Benchmark<PGMInterface<uint32_t, 32>, uint32_t>, dataset)->Iterations(10);
            benchmark::RegisterBenchmark(dataset+"_PGM128", Benchmark<PGMInterface<uint32_t, 128>, uint32_t>, dataset)->Iterations(10);
            benchmark::RegisterBenchmark(dataset+"_PGM++8", Benchmark<PGMPPInterface<uint32_t, 8>, uint32_t>, dataset)->Iterations(10);
            benchmark::RegisterBenchmark(dataset+"_PGM++32", Benchmark<PGMPPInterface<uint32_t, 32>, uint32_t>, dataset)->Iterations(10);
            benchmark::RegisterBenchmark(dataset+"_PGM++128", Benchmark<PGMPPInterface<uint32_t, 128>, uint32_t>, dataset)->Iterations(10);
        } else {
            //benchmark::RegisterBenchmark(dataset+"_std::vector", Benchmark<StdVectorInterface<uint32_t>, uint32_t>, dataset)->Iterations(10);
            benchmark::RegisterBenchmark(dataset+"_PGM8", Benchmark<PGMInterface<uint64_t, 8>, uint64_t>, dataset)->Iterations(10);
            benchmark::RegisterBenchmark(dataset+"_PGM32", Benchmark<PGMInterface<uint64_t, 32>, uint64_t>, dataset)->Iterations(10);
            benchmark::RegisterBenchmark(dataset+"_PGM128", Benchmark<PGMInterface<uint64_t, 128>, uint64_t>, dataset)->Iterations(10);
            benchmark::RegisterBenchmark(dataset+"_PGM++8", Benchmark<PGMPPInterface<uint64_t, 8>, uint64_t>, dataset)->Iterations(10);
            benchmark::RegisterBenchmark(dataset+"_PGM++32", Benchmark<PGMPPInterface<uint64_t, 32>, uint64_t>, dataset)->Iterations(10);
            benchmark::RegisterBenchmark(dataset+"_PGM++128", Benchmark<PGMPPInterface<uint64_t, 128>, uint64_t>, dataset)->Iterations(10);
        }
    }
    benchmark::Initialize(&argc, argv);
    benchmark::RunSpecifiedBenchmarks();
    benchmark::Shutdown();
}
