# Sorted Static Benchmark

This repository contains the benchmark on Sorted Integers for standard, learned, and compressed indices.

**Contributions:**
- Comprehensive analysis of time/space performance of indexing data structures for static integer sets;
- a highly optimized implementation of a BTree exploiting SIMD instructions;
- an easily extensible benchmark framework consisting of *TOT* compressed indexes, *TOT* standard indexes, and *TOT* learned indexes, selected among the most well-known and best-performing ones;
- a new hybrid index deploying compression, SIMD instructions, and learned models. 

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
git clone --recurse-submodules https://github.com/LorenzoBellomo/SortedStaticBenchmark.git
cd SortedStaticBenchmark
./scripts/install.sh
./scripts/download_datasets.sh
./scripts/run_rmi.sh
./scripts/run_experiments.sh
python3 plotter.py
```

## Scripts
The scripts provided are the following:
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
cmake -DCMAKE_BUILD_TYPE=Release .. # uses O2 as optimization flag
 # or 
cmake -DCMAKE_BUILD_TYPE=Debug .. # to also run asserts, uses O2 as optimization flag
```

## Understanding the results and the plots
All the plots with the detailed results of the benchmark are stored in the output/plots folder. This repository also prevides a TeX file (**plot\_recap.tex**) that has a blueprint to create a PDF full-report of the plots. 
If you are not familiar with LaTeX, simply go on [Overleaf](https://www.overleaf.com/), create an account, then click on **New Project -> Blank Project**. Inside the project, paste all the content of **plot\_recap.tex** into the main file. 
Then take all the content of the **output/plots** folder and put it inside the project. 
In the end, you should have a TeX file and a plots folder. 
Then click on Recompile and this will create the PDF report. 

## Adding a new index
To add an index to the benchmark you must follow the following steps:
- add the project in the CMakeLists.txt file;
- create, under "include/index_ops" the interface file for that index, implementing the required methods (prepare, build, next_geq, access, size_in_bytes, to_string);
- register the benchmark in the files buildtime.cpp, existing.cpp, and missing.cpp. In case the new one is a compressed index, also add register the benchmark in the scan.cpp file. Finally, in these files, add the #include<index_ops/YOURFILE.hpp> line at the beginning.

The methods to implement are:
- _prepare_: a function that is run before _"build"_, but is not timed. It is used for some indexes if they need extra work that is not conisdered build time (for example, to copy a vector);
- _build_: builds the index;
- _next\_geq_: returns the first item that is greater of equal than the query, or the max possible item represented in the number of bits used for representing data;
- _access_: **MUST BE IMPLEMENTED only for compressed indexes**, and is used to access the i-th element in the collection;
- _size\_in\_bytes_: provides the size in bytes of the model.
- _to\_string_: string version of the index, must be coherent with the one specified when registering a benchmark (e.g., PGM8, std::vector...)