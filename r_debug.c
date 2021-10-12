#include <link.h>
#include <inttypes.h>
#include <stdio.h>

uint64_t base_addr() {
    return _r_debug.r_map->l_addr;
}