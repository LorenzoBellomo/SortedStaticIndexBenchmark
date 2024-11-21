cd indices/RMI

cargo run --release -- --optimize ../../output/optimizer_output/companynet.json ../../data/companynet_uint32
cargo run --release -- --optimize ../../output/optimizer_output/books.json ../../data/books_200M_uint32
cargo run --release -- --optimize ../../output/optimizer_output/fb64.json ../../data/fb_200M_uint64
cargo run --release -- --optimize ../../output/optimizer_output/lognormal.json ../../data/lognormal_uint32
cargo run --release -- --optimize ../../output/optimizer_output/osm64.json ../../data/osm_cellids_800M_uint64
cargo run --release -- --optimize ../../output/optimizer_output/wiki64.json ../../data/wiki_ts_200M_uint64
cargo run --release -- --optimize ../../output/optimizer_output/books64.json ../../data/books_800M_uint64
cargo run --release -- --optimize ../../output/optimizer_output/exponential.json ../../data/exponential_uint32
cargo run --release -- --optimize ../../output/optimizer_output/friendster.json ../../data/friendster_50M_uint32
cargo run --release -- --optimize ../../output/optimizer_output/normal.json ../../data/normal_uint32
cargo run --release -- --optimize ../../output/optimizer_output/wiki.json ../../data/wiki_ts_200M_uint32
cargo run --release -- --optimize ../../output/optimizer_output/zipf.json ../../data/zipf_uint32