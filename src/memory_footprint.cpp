
#include <iostream>
#include <string>
#include <util.hpp>

#include <index_ops/std_vector.hpp>
#include <index_ops/pgm.hpp>
#include <index_ops/pgmpp.hpp>
#include <index_ops/sdsl_eliasfano.hpp>
#include <index_ops/roaring.hpp>
#include <index_ops/lavec.hpp>
#include <index_ops/rrrgammadelta.hpp>
#include <index_ops/csstree.hpp>
#include <index_ops/alex.hpp>
#include <index_ops/plex.hpp>
#include <index_ops/rmi.hpp>

template <class IDX, typename T>
void build_index(std::string dataset_name) {
    std::vector<T> data = LoadDataset<T>("../data/" + dataset_name);
    IDX idx;
    idx.prepare(data);
    idx.build(data);
}

int main(int argc, char *argv[]) {

    std::string libr = argv[1];
    std::string dataset = argv[2];

    if (dataset.back() == '2') {
        if (libr == "PLEX8")
            build_index<PLEXInterface<uint32_t, 8>, uint32_t>(dataset);
        else if (libr == "PLEX32") 
            build_index<PLEXInterface<uint32_t, 32>, uint32_t>(dataset);
        else if (libr == "PLEX128") 
            build_index<PLEXInterface<uint32_t, 128>, uint32_t>(dataset);
        else if (libr == "ALEX") 
            build_index<ALEXInterface<uint32_t>, uint32_t>(dataset);
        else if (libr == "CSS-Btree") 
            build_index<CSSInterface<uint32_t, 32>, uint32_t>(dataset);
        else if (libr == "RRR-Vector15") 
            build_index<RRRInterface<uint32_t, 15>, uint32_t>(dataset);
        else if (libr == "RRR-Vector31") 
            build_index<RRRInterface<uint32_t, 31>, uint32_t>(dataset);
        else if (libr == "RRR-Vector127") 
            build_index<RRRInterface<uint32_t, 127>, uint32_t>(dataset);
        else if (libr == "GammaCode16") 
            build_index<GammaInterface<uint32_t, 16>, uint32_t>(dataset);
        else if (libr == "GammaCode32") 
            build_index<GammaInterface<uint32_t, 32>, uint32_t>(dataset);
        else if (libr == "DeltaCode16") 
            build_index<DeltaInterface<uint32_t, 16>, uint32_t>(dataset);
        else if (libr == "DeltaCode32") 
            build_index<GammaInterface<uint32_t, 32>, uint32_t>(dataset);
        else if (libr == "LA-vectoropt") 
            build_index<LaVectorOptInterface<uint32_t>, uint32_t>(dataset);
        else if (libr == "LA-vector6") 
            build_index<LaVectorInterface<uint32_t, 6>, uint32_t>(dataset);
        else if (libr == "LA-vector8") 
            build_index<LaVectorInterface<uint32_t, 8>, uint32_t>(dataset);
        else if (libr == "LA-vector10") 
            build_index<LaVectorInterface<uint32_t, 10>, uint32_t>(dataset);
        else if (libr == "LA-vector12") 
            build_index<LaVectorInterface<uint32_t, 12>, uint32_t>(dataset);
        else if (libr == "Roaring") 
            build_index<RoaringInterface<uint32_t>, uint32_t>(dataset);
        else if (libr == "std::vector") 
            build_index<StdVectorInterface<uint32_t>, uint32_t>(dataset);
        else if (libr == "EliasFano") 
            build_index<SDSLEliasFanoInterface<uint32_t>, uint32_t>(dataset);
        else if (libr == "PGM8") 
            build_index<PGMInterface<uint32_t, 8>, uint32_t>(dataset);
        else if (libr == "PGM32") 
            build_index<PGMInterface<uint32_t, 32>, uint32_t>(dataset);
        else if (libr == "PGM128") 
            build_index<PGMInterface<uint32_t, 128>, uint32_t>(dataset);
        else if (libr == "PGM++8") 
            build_index<PGMPPInterface<uint32_t, 8>, uint32_t>(dataset);
        else if (libr == "PGM++32") 
            build_index<PGMPPInterface<uint32_t, 32>, uint32_t>(dataset);
        else if (libr == "PGM++128") 
            build_index<PGMPPInterface<uint32_t, 128>, uint32_t>(dataset);
    } else {
        if (libr == "PLEX8")
            build_index<PLEXInterface<uint64_t, 8>, uint64_t>(dataset);
        else if (libr == "PLEX32") 
            build_index<PLEXInterface<uint64_t, 32>, uint64_t>(dataset);
        else if (libr == "PLEX128") 
            build_index<PLEXInterface<uint64_t, 128>, uint64_t>(dataset);
        else if (libr == "ALEX") 
            build_index<ALEXInterface<uint64_t>, uint64_t>(dataset);
        else if (libr == "CSS-Btree") 
            build_index<CSSInterface<uint64_t, 32>, uint64_t>(dataset);
        else if (libr == "RRR-Vector15") 
            build_index<RRRInterface<uint64_t, 15>, uint64_t>(dataset);
        else if (libr == "RRR-Vector31") 
            build_index<RRRInterface<uint64_t, 31>, uint64_t>(dataset);
        else if (libr == "RRR-Vector127") 
            build_index<RRRInterface<uint64_t, 127>, uint64_t>(dataset);
        else if (libr == "GammaCode16") 
            build_index<GammaInterface<uint64_t, 16>, uint64_t>(dataset);
        else if (libr == "GammaCode32") 
            build_index<GammaInterface<uint64_t, 32>, uint64_t>(dataset);
        else if (libr == "DeltaCode16") 
            build_index<DeltaInterface<uint64_t, 16>, uint64_t>(dataset);
        else if (libr == "DeltaCode32") 
            build_index<DeltaInterface<uint64_t, 32>, uint64_t>(dataset);
        else if (libr == "LA-vectoropt") 
            build_index<LaVectorOptInterface<uint64_t>, uint64_t>(dataset);
        else if (libr == "LA-vector6") 
            build_index<LaVectorInterface<uint64_t, 6>, uint64_t>(dataset);
        else if (libr == "LA-vector8") 
            build_index<LaVectorInterface<uint64_t, 8>, uint64_t>(dataset);
        else if (libr == "LA-vector10") 
            build_index<LaVectorInterface<uint64_t, 10>, uint64_t>(dataset);
        else if (libr == "LA-vector12") 
            build_index<LaVectorInterface<uint64_t, 12>, uint64_t>(dataset);
        else if (libr == "Roaring") 
            build_index<RoaringInterface<uint64_t>, uint64_t>(dataset);
        else if (libr == "std::vector") 
            build_index<StdVectorInterface<uint64_t>, uint64_t>(dataset);
        else if (libr == "EliasFano") 
            build_index<SDSLEliasFanoInterface<uint64_t>, uint64_t>(dataset);
        else if (libr == "PGM8") 
            build_index<PGMInterface<uint64_t, 8>, uint64_t>(dataset);
        else if (libr == "PGM32") 
            build_index<PGMInterface<uint64_t, 32>, uint64_t>(dataset);
        else if (libr == "PGM128") 
            build_index<PGMInterface<uint64_t, 128>, uint64_t>(dataset);
        else if (libr == "PGM++8") 
            build_index<PGMPPInterface<uint64_t, 8>, uint64_t>(dataset);
        else if (libr == "PGM++32") 
            build_index<PGMPPInterface<uint64_t, 32>, uint64_t>(dataset);
        else if (libr == "PGM++128") 
            build_index<PGMPPInterface<uint64_t, 128>, uint64_t>(dataset);
    }
    return 0;
}