#include <benchmark/benchmark.h>
#include <iostream>
#include <fstream>
#include <util.hpp>

#include <index_ops/std_vector.hpp>
#include <index_ops/sdsl_eliasfano.hpp>
#include <index_ops/roaring.hpp>
#include <index_ops/lavec.hpp>
#include <index_ops/gammadelta.hpp>

template <class IDX, typename T>
void Benchmark(benchmark::State& state, std::string dataset_name, size_t scan_size) {
    auto data = LoadDataset<T>("../data/" + dataset_name);
    IDX idx;
    auto lookups = GenerateScanLookups<T>(data, scan_size);
    idx.prepare(data);
    idx.build(data);
    T results[scan_size];
    for (auto _ : state) {
        for (auto q_idx : lookups) {
            for(auto i = 0; i < scan_size; i++) 
                results[i] = idx.access(q_idx + i);
#ifdef DEBUG
            for(auto i = 0; i < scan_size; i++) {
                assert(results[i] == data[q_idx + i]);
            }
#endif
        }
    }
}

int main(int argc, char** argv) {
    std::vector<std::string> datasets({"wiki_ts_200M_uint32", "companynet_uint32", "fb_200M_uint64", "books_200M_uint32"});

#ifdef DEBUG
    auto num_iter = 1;
#else
    auto num_iter = 10;
#endif

    std::vector<size_t> scan_sizes({10, 100, 1000, 10000});
    
    // Register benchmarks
    for (auto dataset : datasets) {
        for (size_t scan_size : scan_sizes) {
            // registering benchmarks for different datasets, and all the indexes present in index_ops
            std::string string_base = dataset + "_" + std::to_string(scan_size);
            if (dataset.back() == '2')  { // HERE REGISTERING ONLY THE ONES THAT WORK FOR 32 BIT
                //benchmark::RegisterBenchmark(string_base+"_Roaring", Benchmark<RoaringInterface<uint32_t>, uint32_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_GammaCode16", Benchmark<GammaInterface<uint32_t, 16>, uint32_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_GammaCode32", Benchmark<GammaInterface<uint32_t, 32>, uint32_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_DeltaCode16", Benchmark<DeltaInterface<uint32_t, 16>, uint32_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_DeltaCode32", Benchmark<DeltaInterface<uint32_t, 32>, uint32_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_LA-vectoropt", Benchmark<LaVectorOptInterface<uint32_t>, uint32_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_LA-vector6", Benchmark<LaVectorInterface<uint32_t, 6>, uint32_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_LA-vector8", Benchmark<LaVectorInterface<uint32_t, 8>, uint32_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_LA-vector10", Benchmark<LaVectorInterface<uint32_t, 10>, uint32_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_LA-vector12", Benchmark<LaVectorInterface<uint32_t, 12>, uint32_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_std::vector", Benchmark<StdVectorInterface<uint32_t>, uint32_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_EliasFano", Benchmark<SDSLEliasFanoInterface<uint32_t>, uint32_t>, dataset, scan_size)->Iterations(num_iter);
            } else {
                //benchmark::RegisterBenchmark(string_base+"_Roaring", Benchmark<RoaringInterface<uint64_t>, uint64_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_GammaCode16", Benchmark<GammaInterface<uint64_t, 16>, uint64_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_GammaCode32", Benchmark<GammaInterface<uint64_t, 32>, uint64_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_DeltaCode16", Benchmark<DeltaInterface<uint64_t, 16>, uint64_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_DeltaCode32", Benchmark<DeltaInterface<uint64_t, 32>, uint64_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_LA-vectoropt", Benchmark<LaVectorOptInterface<uint64_t>, uint64_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_LA-vector6", Benchmark<LaVectorInterface<uint64_t, 6>, uint64_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_LA-vector8", Benchmark<LaVectorInterface<uint64_t, 8>, uint64_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_LA-vector10", Benchmark<LaVectorInterface<uint64_t, 10>, uint64_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_LA-vector12", Benchmark<LaVectorInterface<uint64_t, 12>, uint64_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_EliasFano", Benchmark<SDSLEliasFanoInterface<uint64_t>, uint64_t>, dataset, scan_size)->Iterations(num_iter);
                benchmark::RegisterBenchmark(string_base+"_std::vector", Benchmark<StdVectorInterface<uint64_t>, uint64_t>, dataset, scan_size)->Iterations(num_iter);
            }
        }
    }
    benchmark::Initialize(&argc, argv);
    benchmark::RunSpecifiedBenchmarks();
    benchmark::Shutdown();
}
