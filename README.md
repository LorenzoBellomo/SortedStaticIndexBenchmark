# SortedStaticBenchmark

This repository contains the benchmark on Sorted Integers for standard, learned, and compressed indices. 

## Requirements
On vanilla Ubuntu 20.04 LTS:
```
sudo apt -y update 
sudo apt -y install zstd python3-pip cmake clang
pip3 install --user numpy scipy matplotlib pandas tabulate
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```