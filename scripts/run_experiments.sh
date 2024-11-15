cd build
cmake -DCOMPILE=existing ..
make 
./wiki --benchmark_out=../benchmark_raw_output/existing/wiki.json --benchmark_out_format=json
./companynet --benchmark_out=../benchmark_raw_output/existing/companynet.json --benchmark_out_format=json
./books --benchmark_out=../benchmark_raw_output/existing/books.json --benchmark_out_format=json
./friendster --benchmark_out=../benchmark_raw_output/existing/friendster.json --benchmark_out_format=json
./normal --benchmark_out=../benchmark_raw_output/existing/normal.json --benchmark_out_format=json
./lognormal --benchmark_out=../benchmark_raw_output/existing/lognormal.json --benchmark_out_format=json
./exponential --benchmark_out=../benchmark_raw_output/existing/exponential.json --benchmark_out_format=json
./zipf --benchmark_out=../benchmark_raw_output/existing/zipf.json --benchmark_out_format=json
./wiki64 --benchmark_out=../benchmark_raw_output/existing/wiki64.json --benchmark_out_format=json
./fb64 --benchmark_out=../benchmark_raw_output/existing/fb64.json --benchmark_out_format=json
./books64 --benchmark_out=../benchmark_raw_output/existing/books64.json --benchmark_out_format=json
./osm64 --benchmark_out=../benchmark_raw_output/existing/friendster64.json --benchmark_out_format=json
make clean

cmake -DCOMPILE=missing ..
make 
./wiki --benchmark_out=../benchmark_raw_output/missing/wiki.json --benchmark_out_format=json
./companynet --benchmark_out=../benchmark_raw_output/missing/companynet.json --benchmark_out_format=json
./books --benchmark_out=../benchmark_raw_output/missing/books.json --benchmark_out_format=json
./friendster --benchmark_out=../benchmark_raw_output/missing/friendster.json --benchmark_out_format=json
./normal --benchmark_out=../benchmark_raw_output/missing/normal.json --benchmark_out_format=json
./lognormal --benchmark_out=../benchmark_raw_output/missing/lognormal.json --benchmark_out_format=json
./exponential --benchmark_out=../benchmark_raw_output/missing/exponential.json --benchmark_out_format=json
./zipf --benchmark_out=../benchmark_raw_output/missing/zipf.json --benchmark_out_format=json
./wiki64 --benchmark_out=../benchmark_raw_output/missing/wiki64.json --benchmark_out_format=json
./fb64 --benchmark_out=../benchmark_raw_output/missing/fb64.json --benchmark_out_format=json
./books64 --benchmark_out=../benchmark_raw_output/missing/books64.json --benchmark_out_format=json
./osm64 --benchmark_out=../benchmark_raw_output/missing/friendster64.json --benchmark_out_format=json
make clean

cmake -DCOMPILE=build ..
make 
./wiki --benchmark_out=../benchmark_raw_output/buildtime/wiki.json --benchmark_out_format=json
./companynet --benchmark_out=../benchmark_raw_output/buildtime/companynet.json --benchmark_out_format=json
./books --benchmark_out=../benchmark_raw_output/buildtime/books.json --benchmark_out_format=json
./friendster --benchmark_out=../benchmark_raw_output/buildtime/friendster.json --benchmark_out_format=json
./normal --benchmark_out=../benchmark_raw_output/buildtime/normal.json --benchmark_out_format=json
./lognormal --benchmark_out=../benchmark_raw_output/buildtime/lognormal.json --benchmark_out_format=json
./exponential --benchmark_out=../benchmark_raw_output/buildtime/exponential.json --benchmark_out_format=json
./zipf --benchmark_out=../benchmark_raw_output/buildtime/zipf.json --benchmark_out_format=json
./wiki64 --benchmark_out=../benchmark_raw_output/buildtime/wiki64.json --benchmark_out_format=json
./fb64 --benchmark_out=../benchmark_raw_output/buildtime/fb64.json --benchmark_out_format=json
./books64 --benchmark_out=../benchmark_raw_output/buildtime/books64.json --benchmark_out_format=json
./osm64 --benchmark_out=../benchmark_raw_output/buildtime/friendster64.json --benchmark_out_format=json
make clean

cmake -DCOMPILE=scan ..
make 
./wiki_10 --benchmark_out=../benchmark_raw_output/scan_neighbors/wiki_10.json --benchmark_out_format=json
./companynet_10 --benchmark_out=../benchmark_raw_output/scan_neighbors/companynet_10.json --benchmark_out_format=json
./books_10 --benchmark_out=../benchmark_raw_output/scan_neighbors/books_10.json --benchmark_out_format=json
./wiki64_10 --benchmark_out=../benchmark_raw_output/scan_neighbors/wiki64_10.json --benchmark_out_format=json
./wiki_100 --benchmark_out=../benchmark_raw_output/scan_neighbors/wiki_100.json --benchmark_out_format=json
./companynet_100 --benchmark_out=../benchmark_raw_output/scan_neighbors/companynet_100.json --benchmark_out_format=json
./books_100 --benchmark_out=../benchmark_raw_output/scan_neighbors/books_100.json --benchmark_out_format=json
./wiki64_100 --benchmark_out=../benchmark_raw_output/scan_neighbors/wiki64_100.json --benchmark_out_format=json
./wiki_1000 --benchmark_out=../benchmark_raw_output/scan_neighbors/wiki_1000.json --benchmark_out_format=json
./companynet_1000 --benchmark_out=../benchmark_raw_output/scan_neighbors/companynet_1000.json --benchmark_out_format=json
./books_1000 --benchmark_out=../benchmark_raw_output/scan_neighbors/books_1000.json --benchmark_out_format=json
./wiki64_1000 --benchmark_out=../benchmark_raw_output/scan_neighbors/wiki64_1000.json --benchmark_out_format=json
./wiki_10000 --benchmark_out=../benchmark_raw_output/scan_neighbors/wiki_10000.json --benchmark_out_format=json
./companynet_10000 --benchmark_out=../benchmark_raw_output/scan_neighbors/companynet_10000.json --benchmark_out_format=json
./books_10000 --benchmark_out=../benchmark_raw_output/scan_neighbors/books_10000.json --benchmark_out_format=json 
./wiki64_10000 --benchmark_out=../benchmark_raw_output/scan_neighbors/wiki64_10000.json --benchmark_out_format=json

options=("ALEX" "SIMDBtree" "SIMDBPlustree" "Roaring" "EliasFano" "GammaCode_16" "GammaCode_32" "DeltaCode_16" "DeltaCode_32" "RRR_vector_15" "RRR_vector_31" "RRR_vector_127" "LA_vector_6" "LA_vector_8" "LA_vector_10" "LA_vector_12" "LA_vector_opt" "CSSTree" "PLEX_8" "PLEX_32" "PLEX_128" "PGM_8" "PGM_32" "PGM_128" "BranchlessPGM_8" "BranchlessPGM_32" "BranchlessPGM_128" "std_vector")
options64=("ALEX" "SIMDBtree" "SIMDBPlustree" "Roaring" "EliasFano" "GammaCode_16" "GammaCode_32" "DeltaCode_16" "DeltaCode_32" "LA_vector_6" "LA_vector_8" "LA_vector_10" "LA_vector_12" "CSSTree" "PLEX_8" "PLEX_32" "PLEX_128" "PGM_8" "PGM_32" "PGM_128" "BranchlessPGM_8" "BranchlessPGM_32" "BranchlessPGM_128" "std_vector")
datasets=("books" "companynet" "wiki" "friendster" "normal" "lognormal" "exponential" "zipf")
datasets64=("books64" "wiki64" "osm64" "fb64")

output_file="../memory_usage.txt"
> "$output_file" 

for dataset in "${datasets[@]}"; do
    echo "$dataset" >> "$output_file"
    for option in "${options[@]}"; do
        max_mem_kb=$( /usr/bin/time -v ./memory_consumption "${option}" "${dataset}" 2>&1 | grep "Maximum resident set size" | awk '{print $6}' )
        max_mem_mb=$(( max_mem_kb / 1024 ))
        echo "$option - $max_mem_mb" >> "$output_file"
    done
    echo "" >> "$output_file"
done

for dataset in "${datasets64[@]}"; do
    echo "$dataset" >> "$output_file"
    for option in "${options64[@]}"; do
        max_mem_kb=$( /usr/bin/time -v ./memory_consumption "${option}" "${dataset}" 2>&1 | grep "Maximum resident set size" | awk '{print $6}' )
        max_mem_mb=$(( max_mem_kb / 1024 ))
        echo "$option - $max_mem_mb" >> "$output_file"
    done
    echo "" >> "$output_file"
done

make clean