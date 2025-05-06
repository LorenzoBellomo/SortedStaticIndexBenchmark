# Sorted Static Index Benchmark

This repository contains the benchmark on Sorted Integers for standard, learned, and compressed indexes.

**Contributions:**
- Comprehensive analysis of time/space performance of indexing data structures for static integer sets;
- a highly optimized implementation of a BTree exploiting SIMD instructions, based on this [Algorithmica article](https://en.algorithmica.org/hpc/data-structures/s-tree/), and that achieves state-of-the-art results;
- an easily extensible benchmark framework consisting of 5 compressed indexes, 3 standard indexes, and 5 learned indexes, selected among the most well-known and best-performing ones;
- a new hybrid index deploying compression, SIMD instructions, and learned models. 

**Tested Indexes (compressed, standard, and learned):**
- [RMI](https://github.com/learnedsystems/RMI) (learned index): original RUST implementation. We tested two configurations on each dataset. The "large" configuration is the fastest (and largest) according to the optimizer on that dataset; the "compact" one is the optimal configuration for the model of size of $\sim 6$ MB.
- [PGM](https://github.com/gvinciguerra/PGM-index) (learned index): original implementation.
- [ALEX](https://github.com/microsoft/ALEX) (learned index): ALEX is not imported as a module because the original implementation does not support the C++ 20 standard, which is required for this project. The implementation is identical to a snapshot of the repository in November 2024, with 4 lines of code changed to support the latest C++ standard.
- [PLEX](https://github.com/stoianmihail/PLEX/) (learned index): original implementation
- [Cache Sensitive BTree](https://github.com/gvinciguerra/CSS-tree) (standard index): CSS-Tree is not imported as a module because an empty constructor is required. The implementation is identical to a snapshot of the repository in November 2024, with the addition of a default empty constructor.
- [PGM++](https://github.com/qyliu-hkust/bench_search) (learned index): PGM++ is not imported as a module because it uses the same namespace as the PGM-index. The implementation is identical to a snapshot of the repository in November 2024, with some changes in the namespaces.
- [Gamma and Delta code vectors](https://github.com/simongog/sdsl-lite) (compressed indexes): use the implementation from the SDSL library.
- [La-Vector](https://github.com/gvinciguerra/la_vector) (compressed index): original implementation.
- [Roaring](https://github.com/RoaringBitmap/CRoaring) (compressed index - only for non-duplicate datasets): original C/C++ implementation, dynamically loaded through CMake.
- [Elias Fano](https://github.com/simongog/sdsl-lite) (compressed index): use the implementation from the SDSL library.
- [SIMD B-Tree](https://github.com/mattiaodorisio/SIMD-B-Tree) (standard index): a new implementation of a BTree and a Sampled BTree using SIMD operations.
- std::vector (baseline): lower\_bound operation (binary search) is used as a baseline
- [FAST](https://github.com/curtis-sun/TLI/blob/main/competitors/fast/src/fast.h) (standard index): implementation of a SIMD-BTree by Viktor Leis based on the paper from Kim et. al for SIGMOD 2010.
- [static-search-tree](https://curiouscoding.nl/posts/static-search-tree/) (standard index): This Static-Search Tree is a highly optimized RUST implementation of what was described in the Algorithmica article. For this reason, the index is not directly integrated in our benchmark as of now, but we provided the results of its own benchmark execution in output/static-search-tree-bm-results.json

**Experiments:**
- _build time_: measures, for each dataset, the time required to construct the index.
- _existing lookups_: measures the time per query for items in the dataset.
-  _missing lookups_: measures the time per query for items not in the dataset.
-  _scan_: ONLY FOR COMPRESSED INDEXES, measures the time per element by accessing a random element and then scanning the next 10/100/1000/10000 items.
-  _memory footprint_: measures the required RAM for that index on that dataset, expressed as a ratio to the std::vector one. 

## Requirements
On vanilla Ubuntu 20.04 LTS:
```
sudo apt -y update 
sudo apt -y install zstd python3-pip cmake clang
pip3 install --user numpy scipy matplotlib pandas tabulate
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

## Run everything, including the experiments
```
git clone --recurse-submodules https://github.com/LorenzoBellomo/SortedStaticIndexBenchmark.git
cd SortedStaticBenchmark
./scripts/install.sh
./scripts/download_datasets.sh
./scripts/run_rmi.sh
./scripts/run_experiments.sh
python3 plotter.py
```

## Scripts
The provided scripts are the following:
- _./scripts/install.sh_ installs Google Benchmark, CPM-cmake, SDSL, and the python3 libraries needed for plotting
- _./scripts/download\_datasets.sh_ downloads all the datasets, and creates all the synthetic ones. 
- _./scripts/run\_rmi\_optimizer.sh_ computes the optimal RMI configurations on all datasets, and dumps the results into the _output/optimizer\_output/_ folder. This folder contains the results of a run on the optimizers on our machines. 
- _./scripts/run\_rmi.sh_ runs RMI on each dataset with two configurations (the first and sixth largest models according to the optimizer). 
- _./scripts/run\_experiments.sh_ runs all the experiments . 
- _python3 plotter.py_ plots all the results of the experiments. Plots are stored in _output/plots/_
- _python3 scripts/plot\_dataset\_distribution.py_ plots all the results of the experiments. Plots are stored in _output/plots/_

## Building the Benchmark
To build the project, use the following commands:
```
cmake -DCMAKE_BUILD_TYPE=Release .. # uses O3 as optimization flag + huge page
 # or 
cmake -DCMAKE_BUILD_TYPE=Debug .. # to also run asserts
```

## Understanding the results and the plots
All the plots with the detailed results of the benchmark are stored in the output/plots folder. This repository also provides a TeX file (**plot\_recap.tex**) that has a blueprint to create a PDF full report of the plots. 
If you are not familiar with LaTeX, simply go on [Overleaf](https://www.overleaf.com/), create an account, then click on **New Project -> Blank Project**. Inside the project, paste all the content of **plot\_recap.tex** into the main file. 
Then take all the content of the **output/plots** folder and put it inside the project (naming this folder "plots"). In the end, you should have a TeX file and a plots folder.
Then click on Recompile and this will create the PDF report. 

## Adding a new index
To add an index to the benchmark you must follow the following steps:
- add the project in the CMakeLists.txt file;
- create, under "include/index_ops" the interface file for that index, implementing the required methods (prepare, build, next_geq, access, size_in_bytes, to_string);
- register the benchmark in the files buildtime.cpp, existing.cpp, and missing.cpp. In case the new one is a compressed index, also add register the benchmark in the scan.cpp file. Finally, in these files, add the #include<index_ops/YOURFILE.hpp> line at the beginning.

The methods to implement (in the .hpp file to include in "include/index_ops") are:
- _prepare_: a function that is run before _"build"_, but is not timed. It is used for some indexes if they need extra work that is not considered build time (for example, to copy a vector);
- _build_: builds the index;
- _next\_geq_: returns the first item that is greater or equal than the query, or the max possible item represented in the number of bits used for representing data;
- _access_: **MUST BE IMPLEMENTED only for compressed indexes**, and is used to access the i-th element in the collection;
- _size\_in\_bytes_: provides the size in bytes of the model.
- _to\_string_: string version of the index, must be coherent with the one specified when registering a benchmark (e.g., PGM8, std::vector...)

## License

This project is licensed under the terms of the Apache License 2.0.

If you use the library please cite the following paper:

> L. Bellomo, G. Cianci, L. de Rosa, P. Ferragina, and M. Odorisio. A Comparative Study of Compressed, Learned, and Traditional Indexing Methods for Integer Data. Proceedings of the 23rd Symposium on Experimental Algorithms (SEA), 2025.

```tex
Bibtex available soon
```