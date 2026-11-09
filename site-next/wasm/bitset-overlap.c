#include <stdint.h>

/* Educational bitset intersection, NOT a cryptographically private PSI protocol. */
__attribute__((visibility("default")))
uint32_t overlap_count(uint32_t alice, uint32_t bob) {
    uint32_t x = alice & bob;
    x = x - ((x >> 1u) & UINT32_C(0x55555555));
    x = (x & UINT32_C(0x33333333)) + ((x >> 2u) & UINT32_C(0x33333333));
    x = (x + (x >> 4u)) & UINT32_C(0x0f0f0f0f);
    return (x * UINT32_C(0x01010101)) >> 24u;
}
