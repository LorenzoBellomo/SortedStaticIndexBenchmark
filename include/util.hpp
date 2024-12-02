#include <fstream>
#include <vector>
#include <json.hpp>
#include "index_ops/std_vector.hpp"
#include "index_ops/pgm.hpp"
// Add headers for other vectors

using json = nlohmann::json;

template <typename T>
std::vector<T> read_bin_file(const std::string& path_to_file) {
    std::ifstream ifs(path_to_file, std::ios::binary);
    uint64_t N = 0;
    ifs.read(reinterpret_cast<char*>(&N), sizeof(uint64_t));
    std::vector<T> buf(N);
    ifs.read(reinterpret_cast<char*>(buf.data()), N * sizeof(T));
    return buf;
}

json LoadConfigJSON(const std::string& config_path) {
    std::ifstream file(config_path);
    return json::parse(file);
}

template <typename T>
IndexInterface<T>* CreateIndex(const std::string& name, const std::vector<std::string>& params) {
    // farsi restituire il nome della classe dal wrapper, e "variant" che restituisce una versione "string" dei template. 
    // far diventare plurale, questo instanzia tutti gli indici con tutte le varianti e restituisce una lista. Poi con ognuna genero il benchmark usando i metodi dei wrapper per tirare fuori i nomi.
    if (name == "std_vector") {
        return new StdVectorInterface<T>();
    } else if (name == "pgm-index") {
        //const size_t eps = atoi(params[0]);
        return new PGMinterface<T>();
    }
    // Add other index creations
    throw std::runtime_error("Unknown index name: " + name);
}
