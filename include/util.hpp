#include <fstream>
#include <vector>
#include <json.hpp>
#include "index_ops/std_vector.hpp"
// Add headers for other vectors

using json = nlohmann::json;

template <typename T>
std::vector<T> LoadDataset(const std::string& filepath) {
    std::ifstream in(filepath, std::ios::binary);
    std::vector<T> data((std::istreambuf_iterator<char>(in)), {});
    return data;
}

json LoadConfig(const std::string& config_path) {
    std::ifstream file(config_path);
    return json::parse(file);
}

IndexInterface* CreateIndex(const std::string& name, const std::vector<std::string>& params) {
    if (name == "std_vector") {
        return new StdVector();
    }
    // Add other index creations
    throw std::runtime_error("Unknown index name: " + name);
}
