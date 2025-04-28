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
#include <index_ops/gammadelta.hpp>
#include <index_ops/csstree.hpp>
#include <index_ops/alex.hpp>
#include <index_ops/plex.hpp>
#include <index_ops/rmi.hpp>
#include <index_ops/simdbtree.hpp>
#include <index_ops/fast.hpp>

template <class IDX, typename T>
void Benchmark(benchmark::State& state, std::string dataset_name) {
    auto data = LoadDataset<T>("../data/" + dataset_name);
    #ifdef DEBUG
    auto data_copy = data;
    #endif
    IDX idx;
    auto lookups = generate_missing_lookups(data, M1);
    try {
        idx.prepare(data);
        idx.build(data);
        for (auto _ : state) {
            for (auto q : lookups) {
                T x = idx.next_geq(q.second);
                benchmark::DoNotOptimize(x);
    #ifdef DEBUG
                assert(x > q.second);
                auto idx = q.first;
                T curr_elem = x;
                while(curr_elem > q.second) {
                    if (idx != 0) {
                        idx--;
                        curr_elem = data_copy[idx];
                        if (curr_elem != x) 
                            assert(curr_elem < q.second); 
                    }
                }
    #endif
            }
        }
    } catch (std::exception& e) {
        state.SkipWithError(e.what());
    }
}

template <typename T>
void register_RMIs(const std::string &dataset, int num_iter) {
    if (dataset == "wiki_ts_200M_uint64") {
        benchmark::RegisterBenchmark(dataset+"_RMI-compact", Benchmark<RMIInterface<T, 0, true>, T>, dataset)->Iterations(num_iter);
        benchmark::RegisterBenchmark(dataset+"_RMI-large", Benchmark<RMIInterface<T, 0, false>, T>, dataset)->Iterations(num_iter);
    } else if (dataset == "lognormal_uint32") {
        benchmark::RegisterBenchmark(dataset+"_RMI-compact", Benchmark<RMIInterface<T, 1, true>, T>, dataset)->Iterations(num_iter);
        benchmark::RegisterBenchmark(dataset+"_RMI-large", Benchmark<RMIInterface<T, 1, false>, T>, dataset)->Iterations(num_iter);
    } else if (dataset == "fb_200M_uint64") {
        benchmark::RegisterBenchmark(dataset+"_RMI-compact", Benchmark<RMIInterface<T, 2, true>, T>, dataset)->Iterations(num_iter);
        benchmark::RegisterBenchmark(dataset+"_RMI-large", Benchmark<RMIInterface<T, 2, false>, T>, dataset)->Iterations(num_iter);
    } else if (dataset == "companynet_uint32") {
        benchmark::RegisterBenchmark(dataset+"_RMI-compact", Benchmark<RMIInterface<T, 3, true>, T>, dataset)->Iterations(num_iter);
        benchmark::RegisterBenchmark(dataset+"_RMI-large", Benchmark<RMIInterface<T, 3, false>, T>, dataset)->Iterations(num_iter);
    } else if (dataset == "normal_uint32") {
        benchmark::RegisterBenchmark(dataset+"_RMI-compact", Benchmark<RMIInterface<T, 4, true>, T>, dataset)->Iterations(num_iter);
        benchmark::RegisterBenchmark(dataset+"_RMI-large", Benchmark<RMIInterface<T, 4, false>, T>, dataset)->Iterations(num_iter);
    } else if (dataset == "wiki_ts_200M_uint32") {
        benchmark::RegisterBenchmark(dataset+"_RMI-compact", Benchmark<RMIInterface<T, 5, true>, T>, dataset)->Iterations(num_iter);
        benchmark::RegisterBenchmark(dataset+"_RMI-large", Benchmark<RMIInterface<T, 5, false>, T>, dataset)->Iterations(num_iter);
    } else if (dataset == "zipf_uint32") {
        benchmark::RegisterBenchmark(dataset+"_RMI-compact", Benchmark<RMIInterface<T, 6, true>, T>, dataset)->Iterations(num_iter);
        benchmark::RegisterBenchmark(dataset+"_RMI-large", Benchmark<RMIInterface<T, 6, false>, T>, dataset)->Iterations(num_iter);
    } else if (dataset == "books_800M_uint64") {
        benchmark::RegisterBenchmark(dataset+"_RMI-compact", Benchmark<RMIInterface<T, 7, true>, T>, dataset)->Iterations(num_iter);
        benchmark::RegisterBenchmark(dataset+"_RMI-large", Benchmark<RMIInterface<T, 7, false>, T>, dataset)->Iterations(num_iter);
    } else if (dataset == "exponential_uint32") {
        benchmark::RegisterBenchmark(dataset+"_RMI-compact", Benchmark<RMIInterface<T, 8, true>, T>, dataset)->Iterations(num_iter);
        benchmark::RegisterBenchmark(dataset+"_RMI-large", Benchmark<RMIInterface<T, 8, false>, T>, dataset)->Iterations(num_iter);
    } else if (dataset == "friendster_50M_uint32") {
        benchmark::RegisterBenchmark(dataset+"_RMI-compact", Benchmark<RMIInterface<T, 9, true>, T>, dataset)->Iterations(num_iter);
        benchmark::RegisterBenchmark(dataset+"_RMI-large", Benchmark<RMIInterface<T, 9, false>, T>, dataset)->Iterations(num_iter);
    } else if (dataset == "osm_cellids_800M_uint64") {
        benchmark::RegisterBenchmark(dataset+"_RMI-compact", Benchmark<RMIInterface<T, 10, true>, T>, dataset)->Iterations(num_iter);
        benchmark::RegisterBenchmark(dataset+"_RMI-large", Benchmark<RMIInterface<T, 10, false>, T>, dataset)->Iterations(num_iter);
    } else if (dataset == "books_200M_uint32") {
        benchmark::RegisterBenchmark(dataset+"_RMI-compact", Benchmark<RMIInterface<T, 11, true>, T>, dataset)->Iterations(num_iter);
        benchmark::RegisterBenchmark(dataset+"_RMI-large", Benchmark<RMIInterface<T, 11, false>, T>, dataset)->Iterations(num_iter);
    } else if (dataset == "books_50M_uint64") {
        benchmark::RegisterBenchmark(dataset+"_RMI-compact", Benchmark<RMIInterface<T, 12, true>, T>, dataset)->Iterations(num_iter);
        benchmark::RegisterBenchmark(dataset+"_RMI-large", Benchmark<RMIInterface<T, 12, false>, T>, dataset)->Iterations(num_iter);
    } else if (dataset == "normal_800M_uint32") {
        benchmark::RegisterBenchmark(dataset+"_RMI-compact", Benchmark<RMIInterface<T, 13, true>, T>, dataset)->Iterations(num_iter);
        benchmark::RegisterBenchmark(dataset+"_RMI-large", Benchmark<RMIInterface<T, 13, false>, T>, dataset)->Iterations(num_iter);
    } else {
        throw std::invalid_argument("Unknown dataset");
    }
}

int main(int argc, char** argv) {
    std::vector<std::string> datasets({"wiki_ts_200M_uint64", "lognormal_uint32", "fb_200M_uint64", "companynet_uint32",
    "normal_uint32", "wiki_ts_200M_uint32", "zipf_uint32", "books_800M_uint64", "exponential_uint32", "friendster_50M_uint32", 
    "osm_cellids_800M_uint64", "books_200M_uint32", "books_50M_uint64", "normal_800M_uint32"});
    std::vector<std::string> no_duplicate_datasets({"fb_200M_uint64", "friendster_50M_uint32", "books_800M_uint64", "osm_cellids_800M_uint64", "books_50M_uint64"});

#ifdef DEBUG
    auto num_iter = 1;
#else
    auto num_iter = 10;
#endif
    
    // Register benchmarks
    for (auto dataset : datasets) {
        // registering benchmarks for different datasets, and all the indexes present in index_ops
        if (dataset.back() == '2')  { // HERE REGISTERING ONLY THE ONES THAT WORK FOR 32 BIT
            register_RMIs<uint32_t>(dataset, num_iter);
            benchmark::RegisterBenchmark(dataset+"_SIMD-BTree", Benchmark<SIMDBTreeInterface<uint32_t>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_SIMD-SampledBTree", Benchmark<SIMDSampledBTreeInterface<uint32_t>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PLEX8", Benchmark<PLEXInterface<uint32_t, 8>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PLEX32", Benchmark<PLEXInterface<uint32_t, 32>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PLEX128", Benchmark<PLEXInterface<uint32_t, 128>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_ALEX", Benchmark<ALEXInterface<uint32_t>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_CSS-Btree", Benchmark<CSSInterface<uint32_t, 32>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_GammaCode16", Benchmark<GammaInterface<uint32_t, 16>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_GammaCode32", Benchmark<GammaInterface<uint32_t, 32>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_DeltaCode16", Benchmark<DeltaInterface<uint32_t, 16>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_DeltaCode32", Benchmark<DeltaInterface<uint32_t, 32>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_LA-vectoropt", Benchmark<LaVectorOptInterface<uint32_t>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_LA-vector6", Benchmark<LaVectorInterface<uint32_t, 6>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_LA-vector8", Benchmark<LaVectorInterface<uint32_t, 8>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_LA-vector10", Benchmark<LaVectorInterface<uint32_t, 10>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_LA-vector12", Benchmark<LaVectorInterface<uint32_t, 12>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_FAST", Benchmark<FASTInterface<uint32_t>, uint32_t>, dataset)->Iterations(num_iter);
            if (std::find(no_duplicate_datasets.begin(), no_duplicate_datasets.end(), dataset) != no_duplicate_datasets.end()) {
                benchmark::RegisterBenchmark(dataset+"_Roaring", Benchmark<RoaringInterface<uint32_t>, uint32_t>, dataset)->Iterations(num_iter);
            }
            benchmark::RegisterBenchmark(dataset+"_std::vector", Benchmark<StdVectorInterface<uint32_t>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_EliasFano", Benchmark<SDSLEliasFanoInterface<uint32_t>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM-index8", Benchmark<PGMInterface<uint32_t, 8>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM-index32", Benchmark<PGMInterface<uint32_t, 32>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM-index128", Benchmark<PGMInterface<uint32_t, 128>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM++8", Benchmark<PGMPPInterface<uint32_t, 8>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM++32", Benchmark<PGMPPInterface<uint32_t, 32>, uint32_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM++128", Benchmark<PGMPPInterface<uint32_t, 128>, uint32_t>, dataset)->Iterations(num_iter);
        } else {
            register_RMIs<uint64_t>(dataset, num_iter);
            benchmark::RegisterBenchmark(dataset+"_SIMD-BTree", Benchmark<SIMDBTreeInterface<uint64_t>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_SIMD-SampledBTree", Benchmark<SIMDSampledBTreeInterface<uint64_t>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PLEX8", Benchmark<PLEXInterface<uint64_t, 8>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PLEX32", Benchmark<PLEXInterface<uint64_t, 32>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PLEX128", Benchmark<PLEXInterface<uint64_t, 128>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_ALEX", Benchmark<ALEXInterface<uint64_t>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_CSS-Btree", Benchmark<CSSInterface<uint64_t, 64>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_GammaCode16", Benchmark<GammaInterface<uint64_t, 16>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_GammaCode32", Benchmark<GammaInterface<uint64_t, 32>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_DeltaCode16", Benchmark<DeltaInterface<uint64_t, 16>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_DeltaCode32", Benchmark<DeltaInterface<uint64_t, 32>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_LA-vectoropt", Benchmark<LaVectorOptInterface<uint64_t>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_LA-vector6", Benchmark<LaVectorInterface<uint64_t, 6>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_LA-vector8", Benchmark<LaVectorInterface<uint64_t, 8>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_LA-vector10", Benchmark<LaVectorInterface<uint64_t, 10>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_LA-vector12", Benchmark<LaVectorInterface<uint64_t, 12>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_FAST", Benchmark<FASTInterface<uint64_t>, uint64_t>, dataset)->Iterations(num_iter);
            if (std::find(no_duplicate_datasets.begin(), no_duplicate_datasets.end(), dataset) != no_duplicate_datasets.end()) {
                benchmark::RegisterBenchmark(dataset+"_Roaring", Benchmark<RoaringInterface<uint64_t>, uint64_t>, dataset)->Iterations(num_iter);
            }
            if (dataset != "fb_200M_uint64")
                benchmark::RegisterBenchmark(dataset+"_EliasFano", Benchmark<SDSLEliasFanoInterface<uint64_t>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM-index8", Benchmark<PGMInterface<uint64_t, 8>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM-index32", Benchmark<PGMInterface<uint64_t, 32>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM-index128", Benchmark<PGMInterface<uint64_t, 128>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM++8", Benchmark<PGMPPInterface<uint64_t, 8>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM++32", Benchmark<PGMPPInterface<uint64_t, 32>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_PGM++128", Benchmark<PGMPPInterface<uint64_t, 128>, uint64_t>, dataset)->Iterations(num_iter);
            benchmark::RegisterBenchmark(dataset+"_std::vector", Benchmark<StdVectorInterface<uint64_t>, uint64_t>, dataset)->Iterations(num_iter);
        }
    }
    benchmark::Initialize(&argc, argv);
    benchmark::RunSpecifiedBenchmarks();
    benchmark::Shutdown();
}
