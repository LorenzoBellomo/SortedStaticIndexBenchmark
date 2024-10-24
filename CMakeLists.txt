cmake_minimum_required(VERSION 3.10)
project(SortedStaticBenchmark)

# Set C++ standard
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

SET(COMPILE "existing" CACHE STRING "existing")

# Compiler flags
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Wextra -fopenmp -ltbb -L~/sdb/libs/tbb/build -march=native -O2")

find_package(Boost)

# Include directories
include_directories(
    ../indices/CSS-tree/include
    /home/sadas/include
    ../indices/CSS-tree/include
    rmi_source_files
    ../indices/PLEX
    ../compressed_vectors/la_vector/include
    ../compressed_vectors/la_vector/lib/sdsl-lite/include
    benchmark/include
    ../indices/PGMPP
    .
    ../compressed_vectors/ds2i
    ../indices/ALEX/src
    ../indices/ALEX/src/core
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
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS}-pthread -L/home/sadas/lorenzo/nuovi_esperimenti/benchmark/build/src -lbenchmark -L/home/sadas/lib -lsdsl -ldivsufsort -ldivsufsort64")

# Source files
set(RMI_SOURCE_FILES rmi_source_files/rmi.cpp)
#set(ZIPF_GEN zipf_gen.cpp)

file(GLOB EXISTING "existing/*.cpp")
file(GLOB MISSING "missing/*.cpp")
file(GLOB BUILDTIME "build_time/*.cpp")
file(GLOB SCAN "scan_neighbors/*.cpp")

add_executable(dataset_generator dataset_generator.cpp)

if (COMPILE STREQUAL "existing")
    message("QUERIES OF EXISTING ITEMS")
    foreach(existing_file ${EXISTING})
        get_filename_component( filename_ ${existing_file} NAME_WE )
        add_executable( ${filename_} ${existing_file} ${RMI_SOURCE_FILES})
        target_link_libraries(${filename_} ${CMAKE_EXE_LINKER_FLAGS})
        target_link_libraries(${filename_} roaring::roaring)
    endforeach(existing_file ${EXISTING})
endif()

if (COMPILE STREQUAL "missing")
    message("QUERIES OF MISSING ITEMS")
    foreach(missing_file ${MISSING})
        get_filename_component( filename_ ${missing_file} NAME_WE )
        add_executable( ${filename_} ${missing_file} ${RMI_SOURCE_FILES} )
        target_link_libraries(${filename_} ${CMAKE_EXE_LINKER_FLAGS})
        target_link_libraries(${filename_} roaring::roaring)
    endforeach(missing_file ${MISSING})
endif()

if (COMPILE STREQUAL "build")
    message("MEASURING BUILD TIME")
    foreach(buildtime_file ${BUILDTIME})
        get_filename_component( filename_ ${buildtime_file} NAME_WE )
        add_executable( ${filename_} ${buildtime_file} ${RMI_SOURCE_FILES} )
        target_link_libraries(${filename_} ${CMAKE_EXE_LINKER_FLAGS})
        target_link_libraries(${filename_} roaring::roaring)
    endforeach(buildtime_file ${BUILDTIME})
endif()

if (COMPILE STREQUAL "scan")
    message("SCAN QUERIES")
    foreach(scan_file ${SCAN})
        get_filename_component( filename_ ${scan_file} NAME_WE )
        add_executable( ${filename_} ${scan_file} ${RMI_SOURCE_FILES})
        target_link_libraries(${filename_} ${CMAKE_EXE_LINKER_FLAGS})
        target_link_libraries(${filename_} roaring::roaring)
    endforeach(scan_file ${SCAN})
endif()
