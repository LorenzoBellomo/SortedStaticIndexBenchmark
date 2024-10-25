# Creating necessary folders
mkdir benchmark_raw_output
mkdir benchmark_raw_output/existing
mkdir benchmark_raw_output/missing
mkdir benchmark_raw_output/buildtime
mkdir benchmark_raw_output/scan_neighbors

# GOOGLE BENCHMARK
echo "Installing Google Benchmark"
cd benchmark
cmake -E make_directory "build"
cmake -E chdir "build" cmake -DBENCHMARK_DOWNLOAD_DEPENDENCIES=on -DCMAKE_BUILD_TYPE=Release ../
cmake --build "build" --config Release 
cd ..

# for roaring (used through cmake)
echo "Installing required tool for using Roaring through CMake"
mkdir build 
cd build 
mkdir -p cmake
wget -O cmake/CPM.cmake https://github.com/cpm-cmake/CPM.cmake/releases/latest/download/get_cpm.cmake
cd ..

# SDSL
mkdir lib
cd indices/sdsl-lite
./install ../../lib
cd ../..

# TODO MANCA DS2I, roba per rmi, matplotlib
