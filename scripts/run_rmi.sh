
mkdir -p rmi_source_files

cd indices/RMI

echo "Books uint32"
cargo run --release -- ../../data/books_200M_uint32 books_compact linear,linear 262144
cargo run --release -- ../../data/books_200M_uint32 books_large linear,linear 16777216

echo "CompanyNet uint32"
cargo run --release -- ../../data/companynet_uint32 companynet_compact linear_spline,linear 262144
cargo run --release -- ../../data/companynet_uint32 companynet_large linear_spline,linear 16777216

echo "Friendster uint32"
cargo run --release -- ../../data/friendster_50M_uint32 friendster_compact linear_spline,linear 262144
cargo run --release -- ../../data/friendster_50M_uint32 friendster_large linear_spline,linear 16777216

echo "Wiki uint32"
cargo run --release -- ../../data/wiki_ts_200M_uint32 wiki_compact linear_spline,linear 262144
cargo run --release -- ../../data/wiki_ts_200M_uint32 wiki_large linear_spline,linear 16777216

echo "exponential uint32"
cargo run --release -- ../../data/exponential_uint32 exponential_compact linear_spline,linear 262144
cargo run --release -- ../../data/exponential_uint32 exponential_large radix,linear 16777216

echo "zipf uint32"
cargo run --release -- ../../data/zipf_uint32 zipf_compact linear_spline,linear 262144
cargo run --release -- ../../data/zipf_uint32 zipf_large radix,linear 16777216

echo "normal uint32"
cargo run --release -- ../../data/normal_uint32 normal_compact linear_spline,linear 262144
cargo run --release -- ../../data/normal_uint32 normal_large linear_spline,linear 16777216

echo "lognormal uint32"
cargo run --release -- ../../data/lognormal_uint32 lognormal_compact linear_spline,linear 262144
cargo run --release -- ../../data/lognormal_uint32 lognormal_large linear_spline,linear 16777216

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

echo "Books 50M uint64"
cargo run --release -- ../../data/books_50M_uint64 books64small_compact linear_spline,linear 262144
cargo run --release -- ../../data/books_50M_uint64 books64small_large linear_spline,linear 16777216

echo "normal 800M uint32"
cargo run --release -- ../../data/normal_800M_uint32 normal800_compact radix,linear 262144
cargo run --release -- ../../data/normal_800M_uint32 normal800_large linear_spline,linear 16777216

# PUTTING ALL .cpp in one whole file (same for _data.h and .h files)
echo "" > ../../rmi_source_files/rmi.h
echo "" > ../../rmi_source_files/rmi_data.h
ls *.h | grep -v 'data\.h$' | xargs -I {} cat {} >> ../../rmi_source_files/rmi.h
cat *_data.h >> ../../rmi_source_files/rmi_data.h

cat *.cpp >> tmp.cpp
echo "#include <rmi.h>" > ../../rmi_source_files/rmi.cpp # writing include rmi.h
echo "#include <rmi_data.h>" >> ../../rmi_source_files/rmi.cpp # writing include rmi_data.h
grep -v '^#include "' tmp.cpp >> ../../rmi_source_files/rmi.cpp # writing everything but includes with the specific rmi models

rm *.cpp
rm *.h