#ifndef FOUND_MAGICS_H_INCL
#define FOUND_MAGICS_H_INCL

#include "chess-util.h"
#include <stdint.h>

/**
 * bishop and rook magic bitboard constants
 *
 * the hashing function is ((occupancy & mask) * factor) >> (64 - bits)
 */
extern const bitboard rook_magic_factors[64];
extern const bitboard bishop_magic_factors[64];
extern const bitboard rook_magic_masks[64];
extern const bitboard bishop_magic_masks[64];

extern const unsigned int rook_magic_bits[64];
extern const unsigned int bishop_magic_bits[64];

#endif
