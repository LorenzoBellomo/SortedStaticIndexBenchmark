#include <iostream>
#include <string>
#include <fstream>
#include <vector>
#include <random>
#include "benchmark/benchmark.h"

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

std::vector<uint32_t> read_bin32_file(const std::string& filename) {
    std::ifstream ifs(filename, std::ios::binary);

    uint64_t N = 0;
    ifs.read(reinterpret_cast<char*>(&N), sizeof(uint64_t));

    std::vector<uint32_t> buf(N);
    ifs.read(reinterpret_cast<char*>(buf.data()), N * sizeof(uint32_t));
    return buf;
}

static void BM_PLEX_32(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        ts::Builder<uint32_t> tsb(data[0], data[data.size() - 1], /*spline_max_error=*/32);
        for (const auto& key : data) tsb.AddKey(key);
        auto ts = tsb.Finalize();
    }
}

static void BM_PLEX_128(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        ts::Builder<uint32_t> tsb(data[0], data[data.size() - 1], /*spline_max_error=*/128);
        for (const auto& key : data) tsb.AddKey(key);
        auto ts = tsb.Finalize();
    }
}

static void BM_PLEX_8(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);;

    for (auto _ : state) {
        ts::Builder<uint32_t> tsb(data[0], data[data.size() - 1], /*spline_max_error=*/8);
        for (const auto& key : data) tsb.AddKey(key);
        auto ts = tsb.Finalize();
    }
}

static void BM_SDSL_EliasFano(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        sdsl::sd_vector<> ef(data.begin(), data.end());
        sdsl::rank_support_sd<> ef_rank(&ef);
        sdsl::select_support_sd<> ef_select(&ef);
    }
}

static void BM_RRR_vector_15(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        wrapper_sdsl<uint32_t, sdsl::rrr_vector<15>> rrr(data);
    }
}


static void BM_RRR_vector_31(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        wrapper_sdsl<uint32_t, sdsl::rrr_vector<31>> rrr(data);
    }
}

static void BM_RRR_vector_127(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        wrapper_sdsl<uint32_t, sdsl::rrr_vector<127>> rrr(data);
    }
}

static void BM_LA_vector_6(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        constexpr auto bpc = 6;
        la_vector<uint32_t, bpc> v1(data);
    }
}

static void BM_LA_vector_8(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        constexpr auto bpc = 8;
        la_vector<uint32_t, bpc> v1(data);
    }
}

static void BM_LA_vector_10(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        constexpr auto bpc = 10;
        la_vector<uint32_t, bpc> v1(data);
    }
}

static void BM_LA_vector_12(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        constexpr auto bpc = 12;
        la_vector<uint32_t, bpc> v1(data);
    }
}

static void BM_LA_vector_opt(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        la_vector_opt<uint32_t> v1(data);
    }
}

static void BM_PGM_128(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        const int epsilon = 128; // space-time trade-off parameter
        pgm::PGMIndex<uint32_t, epsilon, 4, false, 32, float> pgm_idx(data.begin(), data.end()-1);
    }
}

static void BM_PGM_32(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        const int epsilon = 32; // space-time trade-off parameter
        pgm::PGMIndex<uint32_t, epsilon, 4, false, 32, float> pgm_idx(data.begin(), data.end()-1);
    }
}

static void BM_PGM_8(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        const int epsilon = 8; // space-time trade-off parameter
        pgm::PGMIndex<uint32_t, epsilon, 4, false, 32, float> pgm_idx(data.begin(), data.end()-1);
    }
}

static void BM_BranchlessPGM_128(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        const int epsilon = 128; // space-time trade-off parameter
        pgm::PGMIndex<uint32_t, epsilon, 4, true, 32, float> pgm_idx(data.begin(), data.end()-1);
    }
}

static void BM_BranchlessPGM_32(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        const int epsilon = 32; // space-time trade-off parameter
        pgm::PGMIndex<uint32_t, epsilon, 4, true, 32, float> pgm_idx(data.begin(), data.end()-1);
    }
}

static void BM_BranchlessPGM_8(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        const int epsilon = 8; // space-time trade-off parameter
        pgm::PGMIndex<uint32_t, epsilon, 4, true, 32, float> pgm_idx(data.begin(), data.end()-1);
    }
}

static void BM_CSSTree(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        CSSTree<64, uint32_t> tree(data);
    }
}

static void BM_RMI_large(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        rmi_large_namespace::load("/home/sadas/lorenzo/indices/RMI/rmi_data");
    }
}

static void BM_RMI_compact(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        rmi_compact_namespace::load("/home/sadas/lorenzo/indices/RMI/rmi_data");
    }
}

static void BM_GammaCode_16(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        const auto dens = 16;
        wrapper_sdsl_enc_vector<sdsl::enc_vector_rank<sdsl::coder::elias_gamma_rank, dens>, std::vector<uint32_t>> v_gamma(data);
    }
}

static void BM_GammaCode_32(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        const auto dens = 32;
        wrapper_sdsl_enc_vector<sdsl::enc_vector_rank<sdsl::coder::elias_gamma_rank, dens>, std::vector<uint32_t>> v_gamma(data);
    }
}

static void BM_DeltaCode_16(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        const auto dens = 16;
        wrapper_sdsl_enc_vector<sdsl::enc_vector_rank<sdsl::coder::elias_delta_rank, dens>, std::vector<uint32_t>> v_delta(data);
    }
}

static void BM_DeltaCode_32(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        const auto dens = 32;
        wrapper_sdsl_enc_vector<sdsl::enc_vector_rank<sdsl::coder::elias_delta_rank, dens>, std::vector<uint32_t>> v_delta(data);
    }
}

static void BM_SIMDBtree(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        SIMD_Btree::btree<uint32_t> btree;
        btree.build(data.data(), data.data() + data.size());
    }
}

static void BM_SIMDBPlustree(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        SIMD_Btree::b_plus_tree<uint32_t, SIMD_Btree::SIMD_ext::AVX512, 2> btree;
        btree.build(data.data(), data.data() + data.size());
    }
}

static void BM_Roaring(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    for (auto _ : state) {
        roaring::Roaring r1;
        for (auto x : data) {
            r1.add(x);
        }
        r1.runOptimize();
    }
}

static void BM_ALEX(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);

    auto for_alex = new std::pair<uint32_t, uint8_t>[data.size()];
    for (size_t i = 0; i < data.size(); i++) {
        for_alex[i].first = data[i];
        for_alex[i].second = static_cast<uint8_t>(0);
    }

    for (auto _ : state) {
        alex::Alex<uint32_t, uint8_t> alex_;
        alex_.bulk_load(for_alex, data.size());
    }
}

// Register the benchmark with different vector sizes
BENCHMARK(BM_ALEX)->Iterations(10);
BENCHMARK(BM_SIMDBtree)->Iterations(10);
BENCHMARK(BM_SIMDBPlustree)->Iterations(10);
BENCHMARK(BM_Roaring)->Iterations(10);
BENCHMARK(BM_RMI_compact)->Iterations(10);
BENCHMARK(BM_RMI_large)->Iterations(10);
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
BENCHMARK(BM_PLEX_8)->Iterations(10);
BENCHMARK(BM_PLEX_32)->Iterations(10);
BENCHMARK(BM_PLEX_128)->Iterations(10);
BENCHMARK(BM_CSSTree)->Iterations(10);
BENCHMARK(BM_PGM_128)->Iterations(10);
BENCHMARK(BM_PGM_32)->Iterations(10);
BENCHMARK(BM_PGM_8)->Iterations(10);
BENCHMARK(BM_BranchlessPGM_128)->Iterations(10);
BENCHMARK(BM_BranchlessPGM_32)->Iterations(10);
BENCHMARK(BM_BranchlessPGM_8)->Iterations(10);

// Run the benchmark
BENCHMARK_MAIN();