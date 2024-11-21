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
std::string LOOKUP_PATH = "../../data/lookups/books";

namespace rmi_compact_namespace = books_compact;
namespace rmi_large_namespace = books_large;

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

static void BM_PLEX_32(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    ts::Builder<uint32_t> tsb(data[0], data[data.size() - 1], /*spline_max_error=*/32);
    for (const auto& key : data) tsb.AddKey(key);
    auto ts = tsb.Finalize();

    for (auto _ : state) {
        for (auto q : lookups) {
            ts::SearchBound bound = ts.GetSearchBound(q);
            auto start = data.begin() + bound.begin;
            auto last = data.begin() + bound.end;
            auto tmp = *std::lower_bound(start, last, q);
            benchmark::DoNotOptimize(tmp);
            assert(tmp != q);
        }
    }
}

static void BM_PLEX_128(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    ts::Builder<uint32_t> tsb(data[0], data[data.size() - 1], /*spline_max_error=*/128);
    for (const auto& key : data) tsb.AddKey(key);
    auto ts = tsb.Finalize();

    for (auto _ : state) {
        for (auto q : lookups) {
            ts::SearchBound bound = ts.GetSearchBound(q);
            auto start = data.begin() + bound.begin;
            auto last = data.begin() + bound.end;
            auto tmp = *std::lower_bound(start, last, q);
            benchmark::DoNotOptimize(tmp);
            assert(tmp != q);
        }
    }
}

static void BM_PLEX_8(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);;
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    ts::Builder<uint32_t> tsb(data[0], data[data.size() - 1], /*spline_max_error=*/8);
    for (const auto& key : data) tsb.AddKey(key);
    auto ts = tsb.Finalize();

    for (auto _ : state) {
        for (auto q : lookups) {
            ts::SearchBound bound = ts.GetSearchBound(q);
            auto start = data.begin() + bound.begin;
            auto last = data.begin() + bound.end;
            auto tmp = *std::lower_bound(start, last, q);
            benchmark::DoNotOptimize(tmp);
            assert(tmp != q);
        }
    }
}

static void BM_SDSL_EliasFano(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    sdsl::sd_vector<> ef(data.begin(), data.end());
    sdsl::rank_support_sd<> ef_rank(&ef);
    sdsl::select_support_sd<> ef_select(&ef);
    auto access = [&] (size_t i) { return ef_select(i + 1); };

    for (auto _ : state) {
        for (auto q : lookups) {
            auto x = access(ef_rank(q));
            benchmark::DoNotOptimize(x);
            assert(x != q);
        }
    }
}

static void BM_RRR_vector_15(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    wrapper_sdsl<uint32_t, sdsl::rrr_vector<15>> rrr(data);

    for (auto _ : state) {
        for (auto q : lookups) {
            auto x = rrr.select(rrr.rank(q) + 1);
            benchmark::DoNotOptimize(x);
            assert(x != q);
        }
    }
}


static void BM_RRR_vector_31(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    wrapper_sdsl<uint32_t, sdsl::rrr_vector<31>> rrr(data);

    for (auto _ : state) {
        for (auto q : lookups) {
            auto x = rrr.select(rrr.rank(q) + 1);
            benchmark::DoNotOptimize(x);
            assert(x != q);
        }
    }
}

static void BM_RRR_vector_127(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    wrapper_sdsl<uint32_t, sdsl::rrr_vector<127>> rrr(data);

    for (auto _ : state) {
        for (auto q : lookups) {
            auto x = rrr.select(rrr.rank(q) + 1);
            benchmark::DoNotOptimize(x);
            assert(x != q);
        }
    }
}

static void BM_LA_vector_6(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    constexpr auto bpc = 6;
    la_vector<uint32_t, bpc> v1(data);

    for (auto _ : state) {
        for (auto q : lookups) {
            auto lb = v1.lower_bound(q);
            auto x = *lb;
            benchmark::DoNotOptimize(x);
            assert(x != q);
        }
    }
}

static void BM_LA_vector_8(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    constexpr auto bpc = 8;
    la_vector<uint32_t, bpc> v1(data);

    for (auto _ : state) {
        for (auto q : lookups) {
            auto lb = v1.lower_bound(q);
            auto x = *lb;
            benchmark::DoNotOptimize(x);
            assert(x != q);
        }
    }
}

static void BM_LA_vector_10(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    constexpr auto bpc = 10;
    la_vector<uint32_t, bpc> v1(data);

    for (auto _ : state) {
        for (auto q : lookups) {
            auto lb = v1.lower_bound(q);
            auto x = *lb;
            benchmark::DoNotOptimize(x);
            assert(x != q);
        }
    }
}

static void BM_LA_vector_12(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    constexpr auto bpc = 12;
    la_vector<uint32_t, bpc> v1(data);

    for (auto _ : state) {
        for (auto q : lookups) {
            auto lb = v1.lower_bound(q);
            auto x = *lb;
            benchmark::DoNotOptimize(x);
            assert(x != q);
        }
    }
}

static void BM_LA_vector_opt(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    la_vector_opt<uint32_t> v1(data);

    for (auto _ : state) {
        for (auto q : lookups) {
            auto lb = v1.lower_bound(q);
            auto x = *lb;
            benchmark::DoNotOptimize(x);
            assert(x != q);
        }
    }
}

static void BM_PGM_128(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    const int epsilon = 128; // space-time trade-off parameter
    pgm::PGMIndex<uint32_t, epsilon, 4, false, 32, float> pgm_idx(data.begin(), data.end()-1);

    for (auto _ : state) {
        for (auto q : lookups) {
            auto x = *pgm_idx.search_data(data.begin(), q);
            benchmark::DoNotOptimize(x);
            assert(x == q);
        }
    }
}

static void BM_PGM_32(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    const int epsilon = 32; // space-time trade-off parameter
    pgm::PGMIndex<uint32_t, epsilon, 4, false, 32, float> pgm_idx(data.begin(), data.end()-1);

    for (auto _ : state) {
        for (auto q : lookups) {
            auto x = *pgm_idx.search_data(data.begin(), q);
            benchmark::DoNotOptimize(x);
            assert(x == q);
        }
    }
}

static void BM_PGM_8(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    const int epsilon = 8; // space-time trade-off parameter
    pgm::PGMIndex<uint32_t, epsilon, 4, false, 32, float> pgm_idx(data.begin(), data.end()-1);

    for (auto _ : state) {
        for (auto q : lookups) {
            auto x = *pgm_idx.search_data(data.begin(), q);
            benchmark::DoNotOptimize(x);
            assert(x == q);
        }
    }
}

static void BM_BranchlessPGM_128(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    const int epsilon = 128; // space-time trade-off parameter
    pgm::PGMIndex<uint32_t, epsilon, 4, true, 32, float> pgm_idx(data.begin(), data.end()-1);

    for (auto _ : state) {
        for (auto q : lookups) {
            auto x = *pgm_idx.search_data(data.begin(), q);
            benchmark::DoNotOptimize(x);
            assert(x != q);
        }
    }
}

static void BM_BranchlessPGM_32(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    const int epsilon = 32; // space-time trade-off parameter
    pgm::PGMIndex<uint32_t, epsilon, 4, true, 32, float> pgm_idx(data.begin(), data.end()-1);

    for (auto _ : state) {
        for (auto q : lookups) {
            auto x = *pgm_idx.search_data(data.begin(), q);
            benchmark::DoNotOptimize(x);
            assert(x != q);
        }
    }
}

static void BM_BranchlessPGM_8(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    const int epsilon = 8; // space-time trade-off parameter
    pgm::PGMIndex<uint32_t, epsilon, 4, true, 32, float> pgm_idx(data.begin(), data.end()-1);

    for (auto _ : state) {
        for (auto q : lookups) {
            auto x = *pgm_idx.search_data(data.begin(), q);
            benchmark::DoNotOptimize(x);
            assert(x != q);
        }
    }
}

static void BM_CSSTree(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    CSSTree<64, uint32_t> tree(data);

    for (auto _ : state) {
        for (auto q : lookups) {
            auto x = *tree.find(q);
            benchmark::DoNotOptimize(x);
            assert(x != q);
        }
    }
}

static void BM_RMI_large(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);
    rmi_large_namespace::load("/home/sadas/lorenzo/indices/RMI/rmi_data");

    for (auto _ : state) {
        size_t err;
        for (auto q : lookups) {
            uint32_t x = rmi_large_namespace::lookup(q, &err);
            long diff_inf = (long) x - (long) err;
            auto begin_index = std::max(diff_inf, (long) 0);
            auto end_idx = std::min(x + err, data.size());
            auto p = *std::lower_bound(data.begin() + begin_index, data.begin() + end_idx, q);
            benchmark::DoNotOptimize(p);
            assert(p != q);
        }
    }
}

static void BM_RMI_compact(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);
    rmi_compact_namespace::load("/home/sadas/lorenzo/indices/RMI/rmi_data");

    for (auto _ : state) {
        size_t err;
        for (auto q : lookups) {
            uint32_t x = rmi_compact_namespace::lookup(q, &err);
            long diff_inf = (long) x - (long) err;
            auto begin_index = std::max(diff_inf, (long) 0);
            auto end_idx = std::min(x + err, data.size());
            auto p = *std::lower_bound(data.begin() + begin_index, data.begin() + end_idx, q);
            benchmark::DoNotOptimize(p);
            assert(p != q);
        }
    }
}

static void BM_std_vector(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);
    for (auto _ : state) {
        for (auto q : lookups) {
            auto x = *std::lower_bound(data.begin(), data.end(), q);
            benchmark::DoNotOptimize(x);
            assert(x != q);
        }
    }
}

static void BM_GammaCode_16(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    const auto dens = 16;
    wrapper_sdsl_enc_vector<sdsl::enc_vector_rank<sdsl::coder::elias_gamma_rank, dens>, std::vector<uint32_t>> v_gamma(data);

    for (auto _ : state) {
        for (auto q : lookups) {
            auto x = v_gamma.select(v_gamma.rank(q) + 1);
            benchmark::DoNotOptimize(x);
            assert(x != q);
        }
    }
}

static void BM_GammaCode_32(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    const auto dens = 32;
    wrapper_sdsl_enc_vector<sdsl::enc_vector_rank<sdsl::coder::elias_gamma_rank, dens>, std::vector<uint32_t>> v_gamma(data);

    for (auto _ : state) {
        for (auto q : lookups) {
            auto x = v_gamma.select(v_gamma.rank(q) + 1);
            benchmark::DoNotOptimize(x);
            assert(x != q);
        }
    }
}

static void BM_DeltaCode_16(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    const auto dens = 16;
    wrapper_sdsl_enc_vector<sdsl::enc_vector_rank<sdsl::coder::elias_delta_rank, dens>, std::vector<uint32_t>> v_delta(data);

    for (auto _ : state) {
        for (auto q : lookups) {
            auto x = v_delta.select(v_delta.rank(q) + 1);
            benchmark::DoNotOptimize(x);
            assert(x != q);
        }
    }
}

static void BM_DeltaCode_32(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    const auto dens = 32;
    wrapper_sdsl_enc_vector<sdsl::enc_vector_rank<sdsl::coder::elias_delta_rank, dens>, std::vector<uint32_t>> v_delta(data);

    for (auto _ : state) {
        for (auto q : lookups) {
            auto x = v_delta.select(v_delta.rank(q) + 1);
            benchmark::DoNotOptimize(x);
            assert(x != q);
        }
    }
}

static void BM_SIMDBtree(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    SIMD_Btree::btree<uint32_t> btree;
    btree.build(data.data(), data.data() + data.size());

    for (auto _ : state) {
        for (auto q : lookups) {
            auto idx = btree.search(q);
            auto tmp = data[idx];
            benchmark::DoNotOptimize(tmp);
            assert(tmp != q);
        }
    }
}

static void BM_SIMDBPlustree(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    SIMD_Btree::b_plus_tree<uint32_t, SIMD_Btree::SIMD_ext::AVX512, 2> btree;
    btree.build(data.data(), data.data() + data.size());

    for (auto _ : state) {
        for (auto q : lookups) {
            auto idx = btree.search(q);
            auto tmp = data[idx];
            benchmark::DoNotOptimize(tmp);
            assert(tmp != q);
        }
    }
}

static void BM_Roaring(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    roaring::Roaring r1;
    for (auto x : data) {
        r1.add(x);
    }
    r1.runOptimize();

    for (auto _ : state) {
        for (auto q : lookups) {
            auto iter = r1.begin();
            iter.equalorlarger(q);
            auto x = *iter;
            benchmark::DoNotOptimize(x);
            assert(x != q);
        }
    }
}

static void BM_ALEX(benchmark::State& state) {
    std::vector<uint32_t> data = read_bin32_file(DATASET_PATH);
    std::vector<uint32_t> lookups = read_bin32_file(LOOKUP_PATH);

    auto for_alex = new std::pair<uint32_t, uint8_t>[data.size()];
    for (size_t i = 0; i < data.size(); i++) {
        for_alex[i].first = data[i];
        for_alex[i].second = static_cast<uint8_t>(0);
    }

    alex::Alex<uint32_t, uint8_t> alex_;
    alex_.bulk_load(for_alex, data.size());

    for (auto _ : state) {
        for (auto q : lookups) {
            auto iter = alex_.find(q);
            benchmark::DoNotOptimize(iter);
            assert(iter == alex_.end());
        }
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
BENCHMARK(BM_std_vector)->Iterations(10);

// Run the benchmark
BENCHMARK_MAIN();