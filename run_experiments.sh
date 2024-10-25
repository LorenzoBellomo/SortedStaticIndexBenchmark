cd build
cmake -DCOMPILE=existing ..
make 
./wiki --benchmark_out=../benchmark_raw_output/existing/wiki.json --benchmark_out_format=json
./alba --benchmark_out=../benchmark_raw_output/existing/alba.json --benchmark_out_format=json
./books --benchmark_out=../benchmark_raw_output/existing/books.json --benchmark_out_format=json
./friendster --benchmark_out=../benchmark_raw_output/existing/friendster.json --benchmark_out_format=json
./normal --benchmark_out=../benchmark_raw_output/existing/normal.json --benchmark_out_format=json
./lognormal --benchmark_out=../benchmark_raw_output/existing/lognormal.json --benchmark_out_format=json
./exponential --benchmark_out=../benchmark_raw_output/existing/exponential.json --benchmark_out_format=json
./zipf --benchmark_out=../benchmark_raw_output/existing/zipf.json --benchmark_out_format=json
./wiki64 --benchmark_out=../benchmark_raw_output/existing/wiki64.json --benchmark_out_format=json
./fb64 --benchmark_out=../benchmark_raw_output/existing/alba64.json --benchmark_out_format=json
./books64 --benchmark_out=../benchmark_raw_output/existing/books64.json --benchmark_out_format=json
./osm64 --benchmark_out=../benchmark_raw_output/existing/friendster64.json --benchmark_out_format=json

cmake -DCOMPILE=missing ..
make 
./wiki --benchmark_out=../benchmark_raw_output/missing/wiki.json --benchmark_out_format=json
./alba --benchmark_out=../benchmark_raw_output/missing/alba.json --benchmark_out_format=json
./books --benchmark_out=../benchmark_raw_output/missing/books.json --benchmark_out_format=json
./friendster --benchmark_out=../benchmark_raw_output/missing/friendster.json --benchmark_out_format=json
./normal --benchmark_out=../benchmark_raw_output/missing/normal.json --benchmark_out_format=json
./lognormal --benchmark_out=../benchmark_raw_output/missing/lognormal.json --benchmark_out_format=json
./exponential --benchmark_out=../benchmark_raw_output/missing/exponential.json --benchmark_out_format=json
./zipf --benchmark_out=../benchmark_raw_output/missing/zipf.json --benchmark_out_format=json
./wiki64 --benchmark_out=../benchmark_raw_output/missing/wiki64.json --benchmark_out_format=json
./fb64 --benchmark_out=../benchmark_raw_output/missing/alba64.json --benchmark_out_format=json
./books64 --benchmark_out=../benchmark_raw_output/missing/books64.json --benchmark_out_format=json
./osm64 --benchmark_out=../benchmark_raw_output/missing/friendster64.json --benchmark_out_format=json

cmake -DCOMPILE=build ..
make 
./wiki --benchmark_out=../benchmark_raw_output/buildtime/wiki.json --benchmark_out_format=json
./alba --benchmark_out=../benchmark_raw_output/buildtime/alba.json --benchmark_out_format=json
./books --benchmark_out=../benchmark_raw_output/buildtime/books.json --benchmark_out_format=json
./friendster --benchmark_out=../benchmark_raw_output/buildtime/friendster.json --benchmark_out_format=json
./normal --benchmark_out=../benchmark_raw_output/buildtime/normal.json --benchmark_out_format=json
./lognormal --benchmark_out=../benchmark_raw_output/buildtime/lognormal.json --benchmark_out_format=json
./exponential --benchmark_out=../benchmark_raw_output/buildtime/exponential.json --benchmark_out_format=json
./zipf --benchmark_out=../benchmark_raw_output/buildtime/zipf.json --benchmark_out_format=json
./wiki64 --benchmark_out=../benchmark_raw_output/buildtime/wiki64.json --benchmark_out_format=json
./fb64 --benchmark_out=../benchmark_raw_output/buildtime/alba64.json --benchmark_out_format=json
./books64 --benchmark_out=../benchmark_raw_output/buildtime/books64.json --benchmark_out_format=json
./osm64 --benchmark_out=../benchmark_raw_output/buildtime/friendster64.json --benchmark_out_format=json

cmake -DCOMPILE=scan ..
make 
./wiki_10 --benchmark_out=../benchmark_raw_output/scan_neighbors/wiki_10.json --benchmark_out_format=json
./alba_10 --benchmark_out=../benchmark_raw_output/scan_neighbors/alba_10.json --benchmark_out_format=json
./books_10 --benchmark_out=../benchmark_raw_output/scan_neighbors/books_10.json --benchmark_out_format=json
./osm_10 --benchmark_out=../benchmark_raw_output/scan_neighbors/osm_10.json --benchmark_out_format=json
./wiki_100 --benchmark_out=../benchmark_raw_output/scan_neighbors/wiki_100.json --benchmark_out_format=json
./alba_100 --benchmark_out=../benchmark_raw_output/scan_neighbors/alba_100.json --benchmark_out_format=json
./books_100 --benchmark_out=../benchmark_raw_output/scan_neighbors/books_100.json --benchmark_out_format=json
./osm_100 --benchmark_out=../benchmark_raw_output/scan_neighbors/osm_100.json --benchmark_out_format=json
./wiki_1000 --benchmark_out=../benchmark_raw_output/scan_neighbors/wiki_1000.json --benchmark_out_format=json
./alba_1000 --benchmark_out=../benchmark_raw_output/scan_neighbors/alba_1000.json --benchmark_out_format=json
./books_1000 --benchmark_out=../benchmark_raw_output/scan_neighbors/books_1000.json --benchmark_out_format=json
./osm_1000 --benchmark_out=../benchmark_raw_output/scan_neighbors/osm_1000.json --benchmark_out_format=json
./wiki_10000 --benchmark_out=../benchmark_raw_output/scan_neighbors/wiki_10000.json --benchmark_out_format=json
./alba_10000 --benchmark_out=../benchmark_raw_output/scan_neighbors/alba_10000.json --benchmark_out_format=json
./books_10000 --benchmark_out=../benchmark_raw_output/scan_neighbors/books_10000.json --benchmark_out_format=json 
./osm_10000 --benchmark_out=../benchmark_raw_output/scan_neighbors/osm_10000.json --benchmark_out_format=json