cd build
cmake -DCMAKE_BUILD_TYPE=Release .. # OR USE DEBUG IF YOU WANT TO RUN THE ASSERTS
make -j
./buildtime --benchmark_out=../output/buildtime.json --benchmark_out_format=json
./existing --benchmark_out=../output/existing.json --benchmark_out_format=json
./missing --benchmark_out=../output/missing.json --benchmark_out_format=json
./scan --benchmark_out=../output/scan.json --benchmark_out_format=json

# measuring memory footprint

options=("ALEX" "Roaring" "EliasFano" "GammaCode16" "GammaCode32" "DeltaCode16" "DeltaCode32" "LA-vectoropt" "LA-vector6" "LA-vector8" "LA-vector10" "LA-vector12" "CSS-BTree" "PLEX8" "PLEX32" "PLEX128" "PGM8" "PGM32" "PGM128" "PGM++8" "PGM++32" "PGM++128" "std::vector" "SIMD-BTree" "SIMD-SampledBTree")
datasets=("books_200M_uint32" "companynet_uint32" "fb_200M_uint64" "lognormal_uint32" "normal_uint32" "wiki_ts_200M_uint32" "zipf_uint32" "books_800M_uint64" "exponential_uint32" "friendster_50M_uint32" "osm_cellids_800M_uint64" "wiki_ts_200M_uint64" "normal_800M_uint32" "books_50M_uint64")

output_file="../output/memory_footprint.txt"
> "$output_file" 

for dataset in "${datasets[@]}"; do
    echo "$dataset" >> "$output_file"
    for option in "${options[@]}"; do
        max_mem_kb=$( /usr/bin/time -v ./memory_footprint "${option}" "${dataset}" 2>&1 | grep "Maximum resident set size" | awk '{print $6}' )
        max_mem_mb=$(( max_mem_kb / 1024 ))
        echo "$option - $max_mem_mb" >> "$output_file"
    done
    echo "" >> "$output_file"
done

make clean