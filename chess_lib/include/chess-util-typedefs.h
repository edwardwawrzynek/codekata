#include <stdint.h>

#ifndef H_CHESS_UTIL_TYPEDEFS_INCL
#define H_CHESS_UTIL_TYPEDEFS_INCL

typedef uint64_t __chess_util_bitboard;
typedef uint8_t __chess_util_board_pos;

struct __chess_util_board {
  __chess_util_bitboard players[2];
  __chess_util_bitboard pieces[6];
  uint16_t flags;
};

typedef uint64_t __chess_util_move;

struct __chess_util_move_gen {
  // board having move generated for
  struct __chess_util_board *board;
  // occupancy bitboards for sliders + pawns
  // the occupancy for pawns includes the en passant target, occupancy for
  // sliders doesn't
  __chess_util_bitboard occupancy_for_sliders;
  __chess_util_bitboard occupancy_for_pawns;
  // final mask to & with moves
  // for all moves, this should be ~us
  // for attacks, this should be them
  __chess_util_bitboard final_moves_mask;
  // current move being generated
  uint8_t cur_mode;
  uint8_t cur_piece_type;
  uint8_t cur_square;
  uint8_t cur_promotion;
  __chess_util_bitboard cur_moves;
  // if all moves have been generated
  uint8_t done;
  // if at least one move was generated
  uint8_t hit_move;
};

#endif
