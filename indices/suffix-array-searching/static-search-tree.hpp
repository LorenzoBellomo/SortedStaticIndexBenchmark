#ifndef STATICSEARCHTREE_H
#define STATICSEARCHTREE_H

#include <stdint.h>
#include <stddef.h>

// Opaque struct representing the STree with branching factor 16 and node size 16
typedef struct STree16 STree16;

extern "C" {

STree16* stree16_new(const uint32_t* vals, size_t len);

uint32_t stree16_search(const STree16* tree, uint32_t query);

void stree16_free(STree16* tree);

size_t stree16_size(const STree16* tree);
}

#endif // STATICSEARCHTREE_H
