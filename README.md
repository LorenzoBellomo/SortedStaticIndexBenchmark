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


```
cmake -DCMAKE_BUILD_TYPE=Release .. # uses O2 as optimization flag
 # or 
cmake -DCMAKE_BUILD_TYPE=Debug .. # to also run asserts, uses O2 as optimization flag
```



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