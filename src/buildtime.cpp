#include <benchmark/benchmark.h>
#include <iostream>
#include <fstream>
#include <util.hpp>

#include <index_ops/std_vector.hpp>
#include <index_ops/pgm.hpp>
#include <index_ops/pgmpp.hpp>
#include <index_ops/sdsl_eliasfano.hpp>
#include <index_ops/roaring.hpp>
#include <index_ops/lavec.hpp>

template <class IDX, typename T>
void Benchmark(benchmark::State& state, std::string dataset_name) {
    std::vector<T> data = LoadDataset<T>("../data/" + dataset_name);
    IDX idx;
    idx.prepare(data);
    for (auto _ : state) {
        idx.build(data);
    }
}

int main(int argc, char** argv) {
    std::vector<std::string> datasets({"wiki_ts_200M_uint64", "lognormal_uint32", "fb_200M_uint64", "companynet_uint32",
    "normal_uint32", "wiki_ts_200M_uint32", "zipf_uint32", "books_800M_uint64", "exponential_uint32", "friendster_50M_uint32", 
    "osm_cellids_800M_uint64", "books_200M_uint32"});

#ifdef DEBUG
    auto num_iter = 1;
#else
    auto num_iter = 10;
#endif
    
    // Register benchmarks
    for (auto dataset : datasets) {
        // registering benchmarks for different datasets, and all the indexes present in index_ops
        if (dataset.back() == '2')  { // HERE REGISTERING ONLY THE ONES THAT WORK FOR 32 BIT
            benchmark::RegisterBenchmark(dataset+"_LA-vector6", Benchmark<LaVectorInterface<uint32_t, 6>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_LA-vectoropt", Benchmark<LaVectorOptInterface<uint32_t>, uint32_t>, dataset)->Iterations(num_iter);
            /*benchmark::RegisterBenchmark(dataset+"_LA-vector8", Benchmark<LaVectorInterface<uint32_t, 8>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_LA-vector10", Benchmark<LaVectorInterface<uint32_t, 10>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_LA-vector12", Benchmark<LaVectorInterface<uint32_t, 12>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_Roaring", Benchmark<RoaringInterface<uint32_t>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_std::vector", Benchmark<StdVectorInterface<uint32_t>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"__EliasFano", Benchmark<SDSLEliasFanoInterface<uint32_t>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM8", Benchmark<PGMInterface<uint32_t, 8>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM32", Benchmark<PGMInterface<uint32_t, 32>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM128", Benchmark<PGMInterface<uint32_t, 128>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM++8", Benchmark<PGMPPInterface<uint32_t, 8>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM++32", Benchmark<PGMPPInterface<uint32_t, 32>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM++128", Benchmark<PGMPPInterface<uint32_t, 128>, uint32_t>, dataset)->Iterations(num_iter);*/
        } else {
            benchmark::RegisterBenchmark(dataset+"_LA-vector6", Benchmark<LaVectorInterface<uint64_t, 6>, uint64_t>, dataset)->Iterations(num_iter);
            //benchmark::RegisterBenchmark(dataset+"_LA-vectoropt", Benchmark<LaVectorOptInterface<uint64_t>, uint64_t>, dataset)->Iterations(num_iter);
            /*benchmark::RegisterBenchmark(dataset+"_LA-vector8", Benchmark<LaVectorInterface<uint64_t, 8>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_LA-vector10", Benchmark<LaVectorInterface<uint64_t, 10>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_LA-vector12", Benchmark<LaVectorInterface<uint64_t, 12>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_Roaring", Benchmark<RoaringInterface<uint64_t>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_EliasFano", Benchmark<SDSLEliasFanoInterface<uint64_t>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM8", Benchmark<PGMInterface<uint64_t, 8>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM32", Benchmark<PGMInterface<uint64_t, 32>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM128", Benchmark<PGMInterface<uint64_t, 128>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM++8", Benchmark<PGMPPInterface<uint64_t, 8>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM++32", Benchmark<PGMPPInterface<uint64_t, 32>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM++128", Benchmark<PGMPPInterface<uint64_t, 128>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_std::vector", Benchmark<StdVectorInterface<uint64_t>, uint64_t>, dataset)->Iterations(num_iter);*/
        }
    }
    benchmark::Initialize(&argc, argv);
    benchmark::RunSpecifiedBenchmarks();
    benchmark::Shutdown();
}
