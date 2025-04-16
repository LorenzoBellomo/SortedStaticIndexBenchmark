run *args='':
    @cargo run -r --quiet -- {{args}}
build:
    @cargo build -r --quiet
record *args='': build
    @perf record ./target/release/sa-layout {{args}}
report:
    @perf report
perf: record report
flame: build
    @cargo flamegraph --open

stat *args='': build
    perf stat -d ./target/release/sa-layout {{args}}

test2:
    cargo test -r

perf2:
    cargo build -r --tests
    perf record cargo test -r query_substring -- --nocapture
    # perf report
stat2:
    cargo build -r --tests
    perf stat cargo test -r query_substring -- --nocapture

py:
    source .env/bin/activate && maturin develop -r


cpufreq:
    sudo cpupower frequency-set --governor powersave -d 2.6GHz -u 2.6GHz


mevi_init:
     sudo sysctl -w vm.unprivileged_userfaultfd=1
mevi:
    cargo build -r --example stat
    export RUST_LOG=warn
    mevi ./target/release/examples/stat
