#include <iostream>
#include <string>
#include <fstream>
#include <vector>
#include <random>
#include "benchmark/benchmark.h"
#include <numeric>

#include "la_vector.hpp"
#include "pgm/pgm_index.h"
#include "books_compact.h"
#include "books_large.h"
//PLEX
#include "include/ts/builder.h"
#include "include/ts/common.h"
//B+tree
#include <csstree.hpp>
#include <sdsl/bit_vectors.hpp>
#include <sdsl/vectors.hpp>

#include <wrapper_sdsl.hpp>
#include <b_tree.h>
#include <b_plus_tree.h>

#include "roaring.h"
#include "roaring.hh"
#include "roaring64map.hh"

#include "core/alex.h"

std::string DATASET_PATH = "../../data/books_200M_uint32";

namespace rmi_compact_namespace = books_compact;
namespace rmi_large_namespace = books_large;

#define DATA_SIZE 200000000
#define NUM_OF_SCANS 10
#define NUM_LOOKUPS 100000

template<class BidiIter >
BidiIter random_unique(BidiIter begin, BidiIter end, size_t num_random) {
    srand(42);
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

std::vector<uint32_t> read_bin32_file(const std::string& filename) {
    std::ifstream ifs(filename, std::ios::binary);

    uint64_t N = 0;
    ifs.read(reinterpret_cast<char*>(&N), sizeof(uint64_t));

    std::vector<uint32_t> buf(N);
    ifs.read(reinterpret_cast<char*>(buf.data()), N * sizeof(uint32_t));
    return buf;
}

static void BM_SDSL_EliasFano(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<size_t> lookups(DATA_SIZE - NUM_OF_SCANS); 
    std::iota(std::begin(lookups), std::end(lookups), 0);
    random_unique(lookups.begin(), lookups.end(), NUM_LOOKUPS);
    lookups.resize(NUM_LOOKUPS);

    sdsl::sd_vector<> ef(data.begin(), data.end());
    sdsl::rank_support_sd<> ef_rank(&ef);
    sdsl::select_support_sd<> ef_select(&ef);
    auto access = [&] (size_t i) { return ef_select(i + 1); };

    uint32_t results[NUM_OF_SCANS];
    for (auto _ : state) {
        for (auto q_idx : lookups) {
            for(auto i = 0; i < NUM_OF_SCANS; i++) {
                results[i] = access(q_idx + i);
                benchmark::DoNotOptimize(results[i]);
            }
            for(auto i = 0; i < NUM_OF_SCANS; i++) 
                assert(results[i] == data[q_idx + i]);
        }
    }
}

static void BM_RRR_vector_15(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<size_t> lookups(DATA_SIZE - NUM_OF_SCANS); 
    std::iota(std::begin(lookups), std::end(lookups), 0);
    random_unique(lookups.begin(), lookups.end(), NUM_LOOKUPS);
    lookups.resize(NUM_LOOKUPS);

    wrapper_sdsl<uint32_t, sdsl::rrr_vector<15>> rrr(data);

    uint32_t results[NUM_OF_SCANS];
    for (auto _ : state) {
        for (auto q_idx : lookups) {
            for(auto i = 0; i < NUM_OF_SCANS; i++) {
                results[i] = rrr.select(q_idx + i + 1);
                benchmark::DoNotOptimize(results[i]);
            }
            for(auto i = 0; i < NUM_OF_SCANS; i++) 
                assert(results[i] == data[q_idx + i]);
        }
    }
}


static void BM_RRR_vector_31(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<size_t> lookups(DATA_SIZE - NUM_OF_SCANS); 
    std::iota(std::begin(lookups), std::end(lookups), 0);
    random_unique(lookups.begin(), lookups.end(), NUM_LOOKUPS);
    lookups.resize(NUM_LOOKUPS);

    wrapper_sdsl<uint32_t, sdsl::rrr_vector<31>> rrr(data);

    uint32_t results[NUM_OF_SCANS];
    for (auto _ : state) {
        for (auto q_idx : lookups) {
            for(auto i = 0; i < NUM_OF_SCANS; i++) {
                results[i] = rrr.select(q_idx + i + 1);
                benchmark::DoNotOptimize(results[i]);
            }
            for(auto i = 0; i < NUM_OF_SCANS; i++) 
                assert(results[i] == data[q_idx + i]);
        }
    }
}

static void BM_RRR_vector_127(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<size_t> lookups(DATA_SIZE - NUM_OF_SCANS); 
    std::iota(std::begin(lookups), std::end(lookups), 0);
    random_unique(lookups.begin(), lookups.end(), NUM_LOOKUPS);
    lookups.resize(NUM_LOOKUPS);

    wrapper_sdsl<uint32_t, sdsl::rrr_vector<127>> rrr(data);

    uint32_t results[NUM_OF_SCANS];
    for (auto _ : state) {
        for (auto q_idx : lookups) {
            for(auto i = 0; i < NUM_OF_SCANS; i++) {
                results[i] = rrr.select(q_idx + i + 1);
                benchmark::DoNotOptimize(results[i]);
            }
            for(auto i = 0; i < NUM_OF_SCANS; i++) 
                assert(results[i] == data[q_idx + i]);
        }
    }
}

static void BM_LA_vector_6(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<size_t> lookups(DATA_SIZE - NUM_OF_SCANS); 
    std::iota(std::begin(lookups), std::end(lookups), 0);
    random_unique(lookups.begin(), lookups.end(), NUM_LOOKUPS);
    lookups.resize(NUM_LOOKUPS);

    constexpr auto bpc = 6;
    la_vector<uint32_t, bpc> v1(data);

    uint32_t results[NUM_OF_SCANS];
    for (auto _ : state) {
        for (auto q_idx : lookups) {
            for(auto i = 0; i < NUM_OF_SCANS; i++) {
                results[i] = v1[q_idx];
                benchmark::DoNotOptimize(results[i]);
            }
            for(auto i = 0; i < NUM_OF_SCANS; i++) 
                assert(results[i] == data[q_idx + i]);
        }
    }
}

static void BM_LA_vector_8(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<size_t> lookups(DATA_SIZE - NUM_OF_SCANS); 
    std::iota(std::begin(lookups), std::end(lookups), 0);
    random_unique(lookups.begin(), lookups.end(), NUM_LOOKUPS);
    lookups.resize(NUM_LOOKUPS);

    constexpr auto bpc = 8;
    la_vector<uint32_t, bpc> v1(data);

    uint32_t results[NUM_OF_SCANS];
    for (auto _ : state) {
        for (auto q_idx : lookups) {
            for(auto i = 0; i < NUM_OF_SCANS; i++) {
                results[i] = v1[q_idx];
                benchmark::DoNotOptimize(results[i]);
            }
            for(auto i = 0; i < NUM_OF_SCANS; i++) 
                assert(results[i] == data[q_idx + i]);
        }
    }
}

static void BM_LA_vector_10(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<size_t> lookups(DATA_SIZE - NUM_OF_SCANS); 
    std::iota(std::begin(lookups), std::end(lookups), 0);
    random_unique(lookups.begin(), lookups.end(), NUM_LOOKUPS);
    lookups.resize(NUM_LOOKUPS);

    constexpr auto bpc = 10;
    la_vector<uint32_t, bpc> v1(data);

    uint32_t results[NUM_OF_SCANS];
    for (auto _ : state) {
        for (auto q_idx : lookups) {
            for(auto i = 0; i < NUM_OF_SCANS; i++) {
                results[i] = v1[q_idx];
                benchmark::DoNotOptimize(results[i]);
            }
            for(auto i = 0; i < NUM_OF_SCANS; i++) 
                assert(results[i] == data[q_idx + i]);
        }
    }
}

static void BM_LA_vector_12(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<size_t> lookups(DATA_SIZE - NUM_OF_SCANS); 
    std::iota(std::begin(lookups), std::end(lookups), 0);
    random_unique(lookups.begin(), lookups.end(), NUM_LOOKUPS);
    lookups.resize(NUM_LOOKUPS);

    constexpr auto bpc = 12;
    la_vector<uint32_t, bpc> v1(data);

    uint32_t results[NUM_OF_SCANS];
    for (auto _ : state) {
        for (auto q_idx : lookups) {
            for(auto i = 0; i < NUM_OF_SCANS; i++) {
                results[i] = v1[q_idx];
                benchmark::DoNotOptimize(results[i]);
            }
            for(auto i = 0; i < NUM_OF_SCANS; i++) 
                assert(results[i] == data[q_idx + i]);
        }
    }
}

static void BM_LA_vector_opt(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<size_t> lookups(DATA_SIZE - NUM_OF_SCANS); 
    std::iota(std::begin(lookups), std::end(lookups), 0);
    random_unique(lookups.begin(), lookups.end(), NUM_LOOKUPS);
    lookups.resize(NUM_LOOKUPS);

    la_vector_opt<uint32_t> v1(data);

    uint32_t results[NUM_OF_SCANS];
    for (auto _ : state) {
        for (auto q_idx : lookups) {
            for(auto i = 0; i < NUM_OF_SCANS; i++) {
                results[i] = v1[q_idx];
                benchmark::DoNotOptimize(results[i]);
            }
            for(auto i = 0; i < NUM_OF_SCANS; i++) 
                assert(results[i] == data[q_idx + i]);
        }
    }
}

static void BM_std_vector(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<size_t> lookups(DATA_SIZE - NUM_OF_SCANS); 
    std::iota(std::begin(lookups), std::end(lookups), 0);
    random_unique(lookups.begin(), lookups.end(), NUM_LOOKUPS);
    lookups.resize(NUM_LOOKUPS);

    uint32_t results[NUM_OF_SCANS];
    for (auto _ : state) {
        for (auto q_idx : lookups) {
            auto q = data[q_idx];
            auto lb = std::lower_bound(data.begin(), data.end(), q);
            auto computed_idx = lb - data.begin();
            for (auto i = 0; i < NUM_OF_SCANS; i++) {
                results[i] = *lb;
                benchmark::DoNotOptimize(results[i]);
                lb++;
            }
            for (auto i = 0; i < NUM_OF_SCANS; i++) 
                assert(results[i] == data[computed_idx + i]);
        }
    }
}

static void BM_GammaCode_16(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<size_t> lookups(DATA_SIZE - NUM_OF_SCANS); 
    std::iota(std::begin(lookups), std::end(lookups), 0);
    random_unique(lookups.begin(), lookups.end(), NUM_LOOKUPS);
    lookups.resize(NUM_LOOKUPS);

    const auto dens = 16;
    wrapper_sdsl_enc_vector<sdsl::enc_vector_rank<sdsl::coder::elias_gamma_rank, dens>, std::vector<uint32_t>> v_gamma(data);

    uint32_t results[NUM_OF_SCANS];
    for (auto _ : state) {
        for (auto q_idx : lookups) {
            for(auto i = 0; i < NUM_OF_SCANS; i++) {
                results[i] = v_gamma.select(q_idx + i + 1);
                benchmark::DoNotOptimize(results[i]);
            }
            for(auto i = 0; i < NUM_OF_SCANS; i++) 
                assert(results[i] == data[q_idx + i]);
        }
    }
}

static void BM_GammaCode_32(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<size_t> lookups(DATA_SIZE - NUM_OF_SCANS); 
    std::iota(std::begin(lookups), std::end(lookups), 0);
    random_unique(lookups.begin(), lookups.end(), NUM_LOOKUPS);
    lookups.resize(NUM_LOOKUPS);

    const auto dens = 32;
    wrapper_sdsl_enc_vector<sdsl::enc_vector_rank<sdsl::coder::elias_gamma_rank, dens>, std::vector<uint32_t>> v_gamma(data);

    uint32_t results[NUM_OF_SCANS];
    for (auto _ : state) {
        for (auto q_idx : lookups) {
            for(auto i = 0; i < NUM_OF_SCANS; i++) {
                results[i] = v_gamma.select(q_idx + i + 1);
                benchmark::DoNotOptimize(results[i]);
            }
            for(auto i = 0; i < NUM_OF_SCANS; i++) 
                assert(results[i] == data[q_idx + i]);
        }
    }
}

static void BM_DeltaCode_16(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<size_t> lookups(DATA_SIZE - NUM_OF_SCANS); 
    std::iota(std::begin(lookups), std::end(lookups), 0);
    random_unique(lookups.begin(), lookups.end(), NUM_LOOKUPS);
    lookups.resize(NUM_LOOKUPS);

    const auto dens = 16;
    wrapper_sdsl_enc_vector<sdsl::enc_vector_rank<sdsl::coder::elias_delta_rank, dens>, std::vector<uint32_t>> v_delta(data);

    uint32_t results[NUM_OF_SCANS];
    for (auto _ : state) {
        for (auto q_idx : lookups) {
            for(auto i = 0; i < NUM_OF_SCANS; i++) {
                results[i] = v_delta.select(q_idx + i + 1);
                benchmark::DoNotOptimize(results[i]);
            }
            for(auto i = 0; i < NUM_OF_SCANS; i++) 
                assert(results[i] == data[q_idx + i]);
        }
    }
}

static void BM_DeltaCode_32(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<size_t> lookups(DATA_SIZE - NUM_OF_SCANS); 
    std::iota(std::begin(lookups), std::end(lookups), 0);
    random_unique(lookups.begin(), lookups.end(), NUM_LOOKUPS);
    lookups.resize(NUM_LOOKUPS);

    const auto dens = 32;
    wrapper_sdsl_enc_vector<sdsl::enc_vector_rank<sdsl::coder::elias_delta_rank, dens>, std::vector<uint32_t>> v_delta(data);

    uint32_t results[NUM_OF_SCANS];
    for (auto _ : state) {
        for (auto q_idx : lookups) {
            for(auto i = 0; i < NUM_OF_SCANS; i++) {
                results[i] = v_delta.select(q_idx + i + 1);
                benchmark::DoNotOptimize(results[i]);
            }
            for(auto i = 0; i < NUM_OF_SCANS; i++) 
                assert(results[i] == data[q_idx + i]);
        }
    }
}

static void BM_Roaring(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<size_t> lookups(DATA_SIZE - NUM_OF_SCANS); 
    std::iota(std::begin(lookups), std::end(lookups), 0);
    random_unique(lookups.begin(), lookups.end(), NUM_LOOKUPS);
    lookups.resize(NUM_LOOKUPS);
    roaring::Roaring r1;
    for (auto x : data) {
        r1.add(x);
    }
    r1.runOptimize();

    uint32_t results[NUM_OF_SCANS];
    for (auto _ : state) {
        for (auto q_idx : lookups) {
            auto iter = r1.begin();
            auto q = data[q_idx];
            iter.equalorlarger(q);
            for(auto i = 0; i < NUM_OF_SCANS; i++) {
                results[i] = *iter;
                benchmark::DoNotOptimize(results[i]);
                iter++;
            }
        }
    }
}

// Register the benchmark with different vector sizes
BENCHMARK(BM_Roaring)->Iterations(10);
BENCHMARK(BM_SDSL_EliasFano)->Iterations(10);
BENCHMARK(BM_GammaCode_16)->Iterations(10);
BENCHMARK(BM_GammaCode_32)->Iterations(10);
BENCHMARK(BM_DeltaCode_16)->Iterations(10);
BENCHMARK(BM_DeltaCode_32)->Iterations(10);
BENCHMARK(BM_RRR_vector_15)->Iterations(10);
BENCHMARK(BM_RRR_vector_31)->Iterations(10);
BENCHMARK(BM_RRR_vector_127)->Iterations(10);
BENCHMARK(BM_LA_vector_6)->Iterations(10);
BENCHMARK(BM_LA_vector_8)->Iterations(10);
BENCHMARK(BM_LA_vector_10)->Iterations(10);
BENCHMARK(BM_LA_vector_12)->Iterations(10);
BENCHMARK(BM_LA_vector_opt)->Iterations(10);
BENCHMARK(BM_std_vector)->Iterations(10);

// Run the benchmark
BENCHMARK_MAIN();