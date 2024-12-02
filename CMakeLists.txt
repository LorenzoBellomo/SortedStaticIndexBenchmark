cmake_minimum_required(VERSION 3.10)
project(SortedStaticBenchmark)

# Set C++ standard
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(BENCHMARK_DOWNLOAD_DEPENDENCIES ON)
add_subdirectory(benchmark)

# Compiler flags
#set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Wextra -fopenmp -ltbb -L~/sdb/libs/tbb/build -march=native") #-O2")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Wextra")

# Include directories
include_directories(
    indices/CSS-tree/include # CSS Btree
    rmi_source_files # RMI
    indices/PLEX/include # PLEX
    indices/la_vector/include # LA Vector
    lib/include # SDSL -> RRR vector, gamma and delta code, EliasFano
    lib/lib # all required for sdsl
    indices # PGM plus plus (pgmpp/pgm_index.h)
    indices/PGM-index/include # regular PGM
    indices/ALEX # ALEX
    indices/simdstructures # SIMD (sampled) btree
    include # implementations of basic operations for each index + json + utils
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

# Linker flags
#set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS}-pthread -L lib/lib -lsdsl -ldivsufsort -ldivsufsort64")

# Source files
#file(GLOB_RECURSE RMI_SOURCE_FILES "rmi_source_files/*.cpp")

file(GLOB SRC_FILES "src/*.cpp")

foreach(src_file ${SRC_FILES})
    get_filename_component( filename_ ${src_file} NAME_WE )
    #add_executable( ${filename_} ${src_file} ${RMI_SOURCE_FILES})
    add_executable( ${filename_} ${src_file})
    target_link_libraries(${filename_} ${CMAKE_EXE_LINKER_FLAGS})
    target_include_directories(${filename_} PRIVATE include)
    target_link_libraries(${filename_} roaring::roaring benchmark::benchmark)
endforeach(src_file ${SRC_FILES})