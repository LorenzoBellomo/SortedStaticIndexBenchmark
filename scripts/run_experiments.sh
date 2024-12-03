cd build
cmake ..
make 
./buildtime --benchmark_out=../output/buildtime.json --benchmark_out_format=json
./existing --benchmark_out=../output/existing.json --benchmark_out_format=json
./missing --benchmark_out=../output/missing.json --benchmark_out_format=json
./scan --benchmark_out=../output/scan.json --benchmark_out_format=json

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

cd ../..
echo "Plotting"
python3 -m pip install -r requirements.txt
python3 plotter.py