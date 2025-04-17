cmake_minimum_required(VERSION 3.10)
project(SortedStaticBenchmark)

# Set C++ standard
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(BENCHMARK_DOWNLOAD_DEPENDENCIES ON)
add_subdirectory(benchmark)

set(CMAKE_CXX_FLAGS_DEBUG "-Wall -Wextra -fopenmp -march=native -O3 -DDEBUG")
set(CMAKE_CXX_FLAGS_RELEASE "-w -fopenmp -march=native -O3")

# Include directories
include_directories(
    indices/CSS-BTree # CSS Btree
    rmi_source_files # RMI
    indices/PLEX/include # PLEX
    indices/la_vector/include # LA Vector
    lib/include # SDSL -> RRR vector, gamma and delta code, EliasFano
    indices/pgmpp # PGM plus plus (pgmpp/pgm_index.h)
    indices/PGM-index/include # regular PGM
    indices/ALEX # ALEX
    indices/s-tree/include # SIMD (sampled) btree
    include # implementations of basic operations for each index + utils + Wrapper SDSL
    indices/FAST # FAST
    indices/suffix-array-searching # RUST SIMD static search tree
)

# You can add CPM.cmake like so:
# mkdir -p cmake
# wget -O cmake/CPM.cmake https://github.com/cpm-cmake/CPM.cmake/releases/latest/download/get_cpm.cmake
include(build/cmake/CPM.cmake)
CPMAddPackage(
  NAME roaring
  GITHUB_REPOSITORY "RoaringBitmap/CRoaring"
  GIT_TAG v2.0.4
  OPTIONS "BUILD_TESTING OFF"
)

# Source files
file(GLOB_RECURSE RMI_SOURCE_FILES "rmi_source_files/*.cpp")

file(GLOB SRC_FILES "src/*.cpp")

foreach(src_file ${SRC_FILES})
    get_filename_component( filename_ ${src_file} NAME_WE )
    add_executable( ${filename_} ${src_file} ${RMI_SOURCE_FILES})
    target_link_libraries(${filename_} 
      benchmark::benchmark 
      roaring::roaring 
      ${CMAKE_SOURCE_DIR}/lib/lib/libsdsl.a 
      ${CMAKE_SOURCE_DIR}/lib/lib/libdivsufsort.a 
      ${CMAKE_SOURCE_DIR}/lib/lib/libdivsufsort64.a
      -L${CMAKE_SOURCE_DIR}/indices/suffix-array-searching/target/release 
      -lstatic_search_tree 
      -Wl,-rpath=${CMAKE_SOURCE_DIR}/indices/suffix-array-searching/target/release
    )
endforeach(src_file ${SRC_FILES})