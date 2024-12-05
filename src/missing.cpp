#include <benchmark/benchmark.h>
#include <iostream>
#include <fstream>
#include <util.hpp>

#include <index_ops/std_vector.hpp>
#include <index_ops/pgm.hpp>
#include <index_ops/pgmpp.hpp>
#include <index_ops/sdsl_eliasfano.hpp>

template <class IDX, typename T>
void Benchmark(benchmark::State& state, std::string dataset_name) {
    auto data = LoadDataset<T>("../data/" + dataset_name);
    IDX idx;
    auto lookups = LoadDataset<T>("../data/lookups/" + dataset_name);
    idx.prepare(data);
    idx.build(data);
    for (auto _ : state) {
        for (auto q : lookups) {
            T x = idx.next_geq(q);
            benchmark::DoNotOptimize(x);
#ifdef DEBUG
            assert(x > q);
#endif
        }
    }
}

int main(int argc, char** argv) {
    std::vector<std::string> datasets({"wiki_ts_200M_uint64", "books_200M_uint32", "lognormal_uint32", "companynet_uint32",
    "normal_uint32", "wiki_ts_200M_uint32", "zipf_uint32", "books_800M_uint64", "exponential_uint32", "friendster_50M_uint32", 
    "osm_cellids_800M_uint64", "fb_200M_uint64"});

#ifdef DEBUG
    auto num_iter = 1;
#else
    auto num_iter = 10;
#endif
    
    // Register benchmarks
    for (auto dataset : datasets) {
        // registering benchmarks for different datasets, and all the indexes present in index_ops
        if (dataset.back() == '2')  { // HERE REGISTERING ONLY THE ONES THAT WORK FOR 32 BIT
            /*benchmark::RegisterBenchmark(dataset+"_std::vector", Benchmark<StdVectorInterface<uint32_t>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM8", Benchmark<PGMInterface<uint32_t, 8>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM32", Benchmark<PGMInterface<uint32_t, 32>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM128", Benchmark<PGMInterface<uint32_t, 128>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM++8", Benchmark<PGMPPInterface<uint32_t, 8>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM++32", Benchmark<PGMPPInterface<uint32_t, 32>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM++128", Benchmark<PGMPPInterface<uint32_t, 128>, uint32_t>, dataset)->Iterations(num_iter);*/
            benchmark::RegisterBenchmark(dataset+"_SDSL-EliasFano", Benchmark<SDSLEliasFanoInterface<uint32_t>, uint32_t>, dataset)->Iterations(num_iter);
        } else {
            //benchmark::RegisterBenchmark(dataset+"_std::vector", Benchmark<StdVectorInterface<uint64_t>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_SDSL-EliasFano", Benchmark<SDSLEliasFanoInterface<uint64_t>, uint64_t>, dataset)->Iterations(num_iter);
            /*benchmark::RegisterBenchmark(dataset+"_PGM8", Benchmark<PGMInterface<uint64_t, 8>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM32", Benchmark<PGMInterface<uint64_t, 32>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM128", Benchmark<PGMInterface<uint64_t, 128>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM++8", Benchmark<PGMPPInterface<uint64_t, 8>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM++32", Benchmark<PGMPPInterface<uint64_t, 32>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM++128", Benchmark<PGMPPInterface<uint64_t, 128>, uint64_t>, dataset)->Iterations(num_iter);*/
        }
    }
    benchmark::Initialize(&argc, argv);
    benchmark::RunSpecifiedBenchmarks();
    benchmark::Shutdown();
}
