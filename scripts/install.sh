
# GOOGLE BENCHMARK
echo "Installing Google Benchmark"
cd benchmark
cmake -E make_directory "build"
cmake -E chdir "build" cmake -DBENCHMARK_DOWNLOAD_DEPENDENCIES=ON -DCMAKE_BUILD_TYPE=Release ../
cmake --build "build" --config Release
cd ..

# for roaring (used through cmake)
echo "Installing required tool for using Roaring through CMake"
mkdir -p build 
cd build 
mkdir -p cmake
wget -O cmake/CPM.cmake https://github.com/cpm-cmake/CPM.cmake/releases/latest/download/get_cpm.cmake
cd ..

# SDSL
mkdir -p lib
cd indices/sdsl-lite
./install.sh ../../lib
cd ../..

echo "Done"