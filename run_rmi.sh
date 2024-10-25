
cd indices/RMI

echo "Books uint32"
cargo run --release -- ../../data/books_100M_uint32 books_compact linear_spline,linear 262144
cargo run --release -- ../../data/books_100M_uint32 books_large linear_spline,linear 16777216

echo "Alba uint32"
cargo run --release -- ../../data/alba_uint32 alba_compact linear_spline,linear 262144
cargo run --release -- ../../data/alba_uint32 alba_large linear_spline,linear 16777216

echo "Friendster uint32"
cargo run --release -- ../../data/friendster_50M_uint32 friendster_compact linear_spline,linear 262144
cargo run --release -- ../../data/friendster_50M_uint32 friendster_large linear_spline,linear 16777216

echo "Wiki uint32"
cargo run --release -- ../../data/wiki_100M_uint32 wiki_compact linear_spline,linear 262144
cargo run --release -- ../../data/wiki_100M_uint32 wiki_large linear_spline,linear 16777216

echo "exponential uint32"
cargo run --release -- ../../data/exponential_uint32 exponential_compact radix,linear 262144
cargo run --release -- ../../data/exponential_uint32 exponential_large radix,linear 16777216

echo "zipf uint32"
cargo run --release -- ../../data/zipf_uint32 exponential_compact radix,linear 262144
cargo run --release -- ../../data/zipf_uint32 exponential_large radix,linear 16777216

echo "normal uint32"
cargo run --release -- ../../data/normal_uint32 normal_compact radix,linear 262144
cargo run --release -- ../../data/normal_uint32 normal_large linear_spline,linear 16777216

echo "lognormal uint32"
cargo run --release -- ../../data/lognormal_uint32 lognormal_compact cubic,linear 262144
cargo run --release -- ../../data/lognormal_uint32 lognormal_large cubic,linear 16777216

echo "Books uint64"
cargo run --release -- ../../data/books_800M_uint64 books64_compact linear_spline,linear 262144
cargo run --release -- ../../data/books_800M_uint64 books64_large linear_spline,linear 16777216

echo "Wiki uint64"
cargo run --release -- ../../data/wiki_ts_200M_uint64 wiki64_compact linear_spline,linear 262144
cargo run --release -- ../../data/wiki_ts_200M_uint64 wiki64_large linear_spline,linear 16777216

echo "OSM uint64"
cargo run --release -- ../../data/osm_cellids_800M_uint64 osm64_compact cubic,linear 262144
cargo run --release -- ../../data/osm_cellids_800M_uint64 osm64_large cubic,linear 16777216

echo "FB uint64"
cargo run --release -- ../../data/fb_200M_uint64 fb64_compact robust_linear,linear 262144
cargo run --release -- ../../data/fb_200M_uint64 fb64_large robust_linear,linear 16777216

# Moving every c++ file to the rmi_source_files folder
mv *.h ../../rmi_source_files
mv *.cpp ../../rmi_source_files