#include "chess-util.h"
#include "found_magics.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

/**
 * knight and king move generation lookup table.
 * for each square on the board, the lookup table contains a bitboard with bits
 * set for each valid move for a knight/king on that square */
static bitboard move_gen_knights[64];
static bitboard move_gen_kings[64];

/**
 * pre-generate the knight lookup table */
static void move_gen_init_knights() {
  for (board_pos pos = 0; pos < 64; pos++) {
    int x, y;
    board_pos_to_xy(pos, &x, &y);
    bitboard moves = 0;
    for (int i = -1; i <= 1; i += 2) {
      for (int j = -1; j <= 1; j += 2) {
        // generate move offset by (1,2) and (2,1) and add to moves if square is
        // in bounds
        board_pos s1 = board_pos_from_xy(x + i * 1, y + j * 2);
        board_pos s2 = board_pos_from_xy(x + i * 2, y + j * 1);

        if (s1 != BOARD_POS_INVALID) {
          moves = bitboard_set_square(moves, s1);
        }
        if (s2 != BOARD_POS_INVALID) {
          moves = bitboard_set_square(moves, s2);
        }
      }
      move_gen_knights[pos] = moves;
    }
  }
}

/**
 * pre-generate the king lookup table */
static void move_gen_init_kings() {
  for (board_pos pos = 0; pos < 64; pos++) {
    int x, y;
    board_pos_to_xy(pos, &x, &y);
    bitboard moves = 0;
    for (int i = -1; i <= 1; i += 1) {
      for (int j = -1; j <= 1; j += 1) {
        if (i == 0 && j == 0)
          continue;

        board_pos square = board_pos_from_xy(x + i, y + j);
        if (square != BOARD_POS_INVALID) {
          moves = bitboard_set_square(moves, square);
        }
      }
      move_gen_kings[pos] = moves;
    }
  }
}

/**
 * given a occupancy bitboard, return valid sliding moves from (x,y) in (dx,dy)
 * direction */
static bitboard gen_ray_moves(bitboard occupancy, int x, int y, int dx,
                              int dy) {
  bitboard moves = 0;
  x = x + dx;
  y = y + dy;
  while (x <= 7 && y <= 7 && x >= 0 && y >= 0) {
    moves = bitboard_set_square(moves, board_pos_from_xy(x, y));
    if (bitboard_check_square(occupancy, board_pos_from_xy(x, y))) {
      break;
    }
    x += dx;
    y += dy;
  }

  return moves;
}

/**
 * given a occupancy bitboard and a bishop/rook square, generate valid moves
 * this is used to pre-generate these moves. see fast lookups below */
static bitboard gen_rook_moves(bitboard occupancy, board_pos square) {
  int x, y;
  board_pos_to_xy(square, &x, &y);
  bitboard moves = 0;
  moves |= gen_ray_moves(occupancy, x, y, 1, 0);
  moves |= gen_ray_moves(occupancy, x, y, -1, 0);
  moves |= gen_ray_moves(occupancy, x, y, 0, 1);
  moves |= gen_ray_moves(occupancy, x, y, 0, -1);
  return moves;
}

static bitboard gen_bishop_moves(bitboard occupancy, board_pos square) {
  int x, y;
  board_pos_to_xy(square, &x, &y);
  bitboard moves = 0;
  moves |= gen_ray_moves(occupancy, x, y, 1, 1);
  moves |= gen_ray_moves(occupancy, x, y, -1, 1);
  moves |= gen_ray_moves(occupancy, x, y, 1, -1);
  moves |= gen_ray_moves(occupancy, x, y, -1, -1);
  return moves;
}

/**
 * sliding pieces (bishop and rook) move lookup table */
#define SLIDING_TABLE_SIZE                                                     \
  107648 // sum of 1 << each entry in rook_magic_bits and bishop_magic_bits
static bitboard sliding_magic_moves[SLIDING_TABLE_SIZE];
// starting indexes for each position in the table
static bitboard *rook_magic_table_ptr[64];
static bitboard *bishop_magic_table_ptr[64];

/**
 * given an occupancy bitboard + square with rook/bishop, generate the index
 * within that square's section of the sliding table */
static uint64_t magic_index_rook(bitboard occupancy, board_pos square) {
  return ((occupancy & rook_magic_masks[square]) *
          rook_magic_factors[square]) >>
         (64 - rook_magic_bits[square]);
}

static uint64_t magic_index_bishop(bitboard occupancy, board_pos square) {
  return ((occupancy & bishop_magic_masks[square]) *
          bishop_magic_factors[square]) >>
         (64 - bishop_magic_bits[square]);
}

/**
 * given an occupancy bitboard, lookup valid moves for a rook/bishop at square
 * because occupancy doesn't distinguish between the players, the result
 * includes capturing all occupied squares
 * to get only valid attacks against the opponent, do (result & ~us_occupancy)
 */
static bitboard rook_move_lookup(bitboard occupancy, board_pos square) {
  assert(square < 64);
  return rook_magic_table_ptr[square][magic_index_rook(occupancy, square)];
}

static bitboard bishop_move_lookup(bitboard occupancy, board_pos square) {
  assert(square < 64);
  return bishop_magic_table_ptr[square][magic_index_bishop(occupancy, square)];
}

/**
 * lookup valid moves for a queen (just rook | bishop) */
static bitboard queen_magic_lookup(bitboard occupancy, board_pos square) {
  return rook_move_lookup(occupancy, square) |
         bishop_move_lookup(occupancy, square);
}

/**
 * given a bitboard mask, generate all permutations of setting/clearing bits in
 * current that are set in maks call func on each permutation */
static void permute_mask(bitboard mask, bitboard current,
                         void (*func)(bitboard, board_pos, int),
                         board_pos func_arg0, int func_arg1) {
  if (!mask) {
    func(current, func_arg0, func_arg1);
  } else {
    // find first set bit in mask, clear it, and generate permutations on
    // current with that bit set + cleared
    board_pos set_i = bitboard_scan_lsb(mask);
    bitboard new_mask = bitboard_clear_square(mask, set_i);
    permute_mask(new_mask, bitboard_set_square(current, set_i), func, func_arg0,
                 func_arg1);
    permute_mask(new_mask, bitboard_clear_square(current, set_i), func,
                 func_arg0, func_arg1);
  }
}

/**
 * callbacks to be passed to permute_mask
 * for this board occupancy, generate rook/bishop moves and put in
 * sliding_magic_moves lookup table */
static void move_gen_init_sliders_rook(bitboard occupancy, board_pos pos,
                                       int pos_tbl_start) {
  bitboard moves = gen_rook_moves(occupancy, pos);
  uint64_t index = pos_tbl_start + magic_index_rook(occupancy, pos);
  assert(index < SLIDING_TABLE_SIZE);
  assert(sliding_magic_moves[index] == 0xffffffffffffffffULL ||
         sliding_magic_moves[index] == moves);
  sliding_magic_moves[index] = moves;
}

static void move_gen_init_sliders_bishop(bitboard occupancy, board_pos pos,
                                         int pos_tbl_start) {
  bitboard moves = gen_bishop_moves(occupancy, pos);
  uint64_t index = pos_tbl_start + magic_index_bishop(occupancy, pos);
  assert(index < SLIDING_TABLE_SIZE);
  assert(sliding_magic_moves[index] == 0xffffffffffffffffULL ||
         sliding_magic_moves[index] == moves);
  sliding_magic_moves[index] = moves;
}

/**
 * pre-generate the sliding_magic_moves array */
static void move_gen_init_sliders() {
  memset(sliding_magic_moves, 0xff, sizeof(sliding_magic_moves));
  int tbl_index = 0;
  for (board_pos pos = 0; pos < 64; pos++) {
    rook_magic_table_ptr[pos] = sliding_magic_moves + tbl_index;
    permute_mask(rook_magic_masks[pos], 0, &move_gen_init_sliders_rook, pos,
                 tbl_index);
    tbl_index += (1 << rook_magic_bits[pos]);
  }
  for (board_pos pos = 0; pos < 64; pos++) {
    bishop_magic_table_ptr[pos] = sliding_magic_moves + tbl_index;
    permute_mask(bishop_magic_masks[pos], 0, &move_gen_init_sliders_bishop, pos,
                 tbl_index);
    tbl_index += (1 << bishop_magic_bits[pos]);
  }
  assert(tbl_index == SLIDING_TABLE_SIZE);
}

// indexed [player][double_rank_ahead][rank_ahead][square]
static bitboard move_gen_pawns[2][2][8][64];

/**
 * pre-generate move_gen_pawns */
static void move_gen_init_pawns() {
  for (int player = 0; player < 2; player++) {
    for (board_pos pos = 0; pos < 64; pos++) {
      int x, y;
      board_pos_to_xy(pos, &x, &y);
      for (unsigned int ahead = 0; ahead < 8; ahead++) {
        for (unsigned int double_ahead = 0; double_ahead < 2; double_ahead++) {
          bitboard moves = 0;
          int dir = player == WHITE ? 1 : -1;
          int ahead_present = (y + dir >= 0 && y + dir < 8);
          // check move directly ahead
          if (!(ahead & 2U) && ahead_present) {
            moves = bitboard_set_square(moves, board_pos_from_xy(x, y + dir));
            // check move two ahead
            if (!double_ahead &&
                ((player == WHITE && y == 1) || (player == BLACK && y == 6))) {
              moves =
                  bitboard_set_square(moves, board_pos_from_xy(x, y + 2 * dir));
            }
          }
          // check captures
          if (x >= 1 && (ahead & 1U) && ahead_present) {
            moves =
                bitboard_set_square(moves, board_pos_from_xy(x - 1, y + dir));
          }
          if (x <= 6 && (ahead & 4U) && ahead_present) {
            moves =
                bitboard_set_square(moves, board_pos_from_xy(x + 1, y + dir));
          }
          // set lookup table
          move_gen_pawns[player][double_ahead][ahead][pos] = moves;
        }
      }
    }
  }
}

/**
 * given an occupancy bitboard, lookup valid moves for a pawn on square with
 * color player */
static bitboard pawn_move_lookup(bitboard occupancy, board_pos square,
                                 int player) {
  int dir = player == WHITE ? 1 : -1;
  // get the three squares forward of the pawn, and the square two squares ahead
  int forward_shift = square - 1 + dir * 8;
  uint8_t forward_rank = forward_shift >= 0
                             ? (occupancy >> forward_shift) & 0x07
                             : (occupancy << -forward_shift) & 0x07;
  int double_forward_shift = square + dir * 16;
  uint8_t double_forward_rank =
      double_forward_shift > 0 ? (occupancy >> double_forward_shift) & 0x01
                               : (occupancy << -double_forward_shift) & 0x01;

  return move_gen_pawns[player][double_forward_rank][forward_rank][square];
}

/**
 * generate moves for the piece at the given square on board
 * this is just regular moves -- does not include castles */
static bitboard move_gen_reg_moves_mask(bitboard occupancy_for_sliders,
                                        bitboard occupancy_for_pawns, int piece,
                                        int player, board_pos square) {
  switch (piece) {
  case KING:
    return move_gen_kings[square];
  case KNIGHT:
    return move_gen_knights[square];
  case PAWN:
    return pawn_move_lookup(occupancy_for_pawns, square, player);
  case ROOK:
    return rook_move_lookup(occupancy_for_sliders, square);
  case BISHOP:
    return bishop_move_lookup(occupancy_for_sliders, square);
  case QUEEN:
    return queen_magic_lookup(occupancy_for_sliders, square);
  default:
    assert(0);
  }
}

/**
 * generate the occupancy mask for slider/pawn lookups */
static bitboard board_occupancy_for_sliders_lookups(const board *board) {
  return board->players[WHITE] | board->players[BLACK];
}

static bitboard board_occupancy_for_pawns_lookups(const board *board) {
  bitboard occupancy = board->players[WHITE] | board->players[BLACK];
  if (board->flags & BOARD_FLAGS_EP_PRESENT) {
    occupancy =
        bitboard_set_square(occupancy, board->flags & BOARD_FLAGS_EP_SQUARE);
  }
  return occupancy;
}

#define MOVE_GEN_MODE_NORMAL 0
#define MOVE_GEN_MODE_CASTLE_KING 1
#define MOVE_GEN_MODE_CASTLE_QUEEN 2
#define MOVE_GEN_MODE_END 3

void move_gen_init(move_gen *move_gen, board *board) {
  int player = board_player_to_move(board);
  move_gen->board = board;
  move_gen->occupancy_for_sliders = board_occupancy_for_sliders_lookups(board);
  move_gen->occupancy_for_pawns = board_occupancy_for_pawns_lookups(board);
  move_gen->final_moves_mask = ~board->players[player];
  move_gen->cur_mode = MOVE_GEN_MODE_NORMAL;
  move_gen->cur_square = 0;
  move_gen->cur_moves = 0;
  move_gen->cur_promotion = KNIGHT;
  move_gen->cur_piece_type = 0;
  move_gen->done = 0;
  move_gen->hit_move = 0;
}

#define MOVE_FLAGS_PREV_FLAGS 0x0000000ffffULL
#define MOVE_FLAGS_SRC 0x000003f0000ULL
#define MOVE_SHIFT_SRC 16
#define MOVE_FLAGS_DST 0x0000fc00000ULL
#define MOVE_SHIFT_DST 22
#define MOVE_FLAGS_IS_PROMOTE 0x00010000000ULL
#define MOVE_SHIFT_IS_PROMOTE 28
#define MOVE_FLAGS_PROMOTE_PIECE 0x000e0000000ULL
#define MOVE_SHIFT_PROMOTE_PIECE 29
#define MOVE_FLAGS_IS_CAPTURE 0x00100000000ULL
#define MOVE_SHIFT_IS_CAPTURE 32
#define MOVE_FLAGS_CAPTURE_PIECE 0x00e00000000ULL
#define MOVE_SHIFT_CAPTURE_PIECE 33
#define MOVE_FLAGS_CAPTURE_SQUARE 0x3f000000000ULL
#define MOVE_SHIFT_CAPTURE_SQUARE 36
#define MOVE_FLAGS_IS_CASTLE 0x40000000000ULL
#define MOVE_SHIFTS_IS_CASTLE 42

board_pos move_source_square(move move) {
  return (move & MOVE_FLAGS_SRC) >> MOVE_SHIFT_SRC;
}

board_pos move_destination_square(move move) {
  return (move & MOVE_FLAGS_DST) >> MOVE_SHIFT_DST;
}

int move_is_promotion(move move) {
  return move & MOVE_FLAGS_IS_PROMOTE ? 1 : 0;
}

int move_promotion_piece(move move) {
  if (!move_is_promotion(move))
    return -1;
  else
    return (move & MOVE_FLAGS_PROMOTE_PIECE) >> MOVE_SHIFT_PROMOTE_PIECE;
}

int move_is_capture(move move) { return move & MOVE_FLAGS_IS_CAPTURE ? 1 : 0; }

int move_capture_piece(move move) {
  if (!move_is_capture(move))
    return -1;
  else
    return (move & MOVE_FLAGS_CAPTURE_PIECE) >> MOVE_SHIFT_CAPTURE_PIECE;
}

board_pos move_capture_square(move move) {
  if (!move_is_capture(move))
    return BOARD_POS_INVALID;
  else
    return (move & MOVE_FLAGS_CAPTURE_SQUARE) >> MOVE_SHIFT_CAPTURE_SQUARE;
}

int move_is_castle(move move) {
  return (move & MOVE_FLAGS_IS_CASTLE) >> MOVE_SHIFTS_IS_CASTLE;
}

/**
 * create a move from the given components */
static move construct_move(uint16_t board_flags, board_pos src, board_pos dst,
                           int is_promotion, int promote_piece, int is_capture,
                           int capture_piece, board_pos capture_pos,
                           int is_castle) {
  return ((uint64_t)board_flags & MOVE_FLAGS_PREV_FLAGS) +
         (((uint64_t)src << MOVE_SHIFT_SRC) & MOVE_FLAGS_SRC) +
         (((uint64_t)dst << MOVE_SHIFT_DST) & MOVE_FLAGS_DST) +
         ((uint64_t)(is_promotion != 0) << MOVE_SHIFT_IS_PROMOTE) +
         (((uint64_t)promote_piece << MOVE_SHIFT_PROMOTE_PIECE) &
          MOVE_FLAGS_PROMOTE_PIECE) +
         ((uint64_t)(is_capture != 0) << MOVE_SHIFT_IS_CAPTURE) +
         (((uint64_t)capture_piece << MOVE_SHIFT_CAPTURE_PIECE) &
          MOVE_FLAGS_CAPTURE_PIECE) +
         (((uint64_t)capture_pos << MOVE_SHIFT_CAPTURE_SQUARE) &
          MOVE_FLAGS_CAPTURE_SQUARE) +
         (((uint64_t)(is_castle != 0) << MOVE_SHIFTS_IS_CASTLE) &
          MOVE_FLAGS_IS_CASTLE);
}

/**
 * given an en passant target square, get the pawn square to which it
 * cooresponds */
static board_pos en_passant_target_to_pawn_pos(board_pos ep_target) {
  int x, y;
  board_pos_to_xy(ep_target, &x, &y);
  // en passant must be on rank 3 or 6
  if (y == 2) {
    return board_pos_from_xy(x, y + 1);
  } else if (y == 5) {
    return board_pos_from_xy(x, y - 1);
  } else {
    assert(0);
  }
}

static char promote_codes[6] = {'k', 'p', 'n', 'r', 'b', 'q'};

void move_to_str(move move, char *res_str) {
  board_pos_to_str(move_source_square(move), res_str);
  board_pos_to_str(move_destination_square(move), res_str + 2);
  if (move_is_promotion(move)) {
    res_str[4] = promote_codes[move_promotion_piece(move)];
    res_str[5] = '\0';
  } else {
    res_str[4] = '\0';
  }
}

move move_new(board_pos src, board_pos dst, int is_promote, int promote_piece, const board *board) {
  // check for capturing
  int is_capture = 0;
  board_pos capture_pos = BOARD_POS_INVALID;
  int capture_piece = board_piece_on_square(board, dst);
  if (capture_piece != -1) {
    if (board_player_on_square(board, dst) == board_player_to_move(board)) {
      return MOVE_END;
    }
    is_capture = 1;
    capture_pos = dst;
  }
  // check for en passant
  if (dst == board_get_en_passant_target(board) &&
      board_piece_on_square(board, src) == PAWN) {
    is_capture = 1;
    capture_pos =
        en_passant_target_to_pawn_pos(board_get_en_passant_target(board));
    capture_piece = board_piece_on_square(board, capture_pos);
    if (capture_piece != PAWN) {
      return MOVE_END;
    }
  }
  int is_castle = 0;
  // check for castling
  if (((src == board_pos_from_xy(4, 0) &&
        (dst == board_pos_from_xy(2, 0) || dst == board_pos_from_xy(6, 0))) ||
       (src == board_pos_from_xy(4, 7) &&
        (dst == board_pos_from_xy(2, 7) || dst == board_pos_from_xy(6, 7)))) &&
      board_piece_on_square(board, src) == KING) {
    is_castle = 1;
  }

  return construct_move(board->flags, src, dst, is_promote, promote_piece,
                        is_capture, capture_piece, capture_pos, is_castle);
}

static int file_wellformed(char file) {
  return (file >= 'a' && file <= 'h') || (file >= 'A' && file <= 'H');
}

static int rank_wellformed(char rank) {
  return rank >= '1' && rank <= '8';
}

static int promote_wellformed(char promote) {
  return promote == 'n' || promote == 'r' || promote == 'b' || promote == 'q';
}

int move_str_is_wellformed(const char *move_str) {
  if(strlen(move_str) != 4 && strlen(move_str) != 5) {
    return 0;
  }
  if(!file_wellformed(move_str[0]) || !rank_wellformed(move_str[1])) {
    return 0;
  }
  if(!file_wellformed(move_str[2]) || !rank_wellformed(move_str[3])) {
    return 0;
  }
  if (strlen(move_str) == 5 && !promote_wellformed(move_str[4])) {
    return 0;
  }
  return 1;
}

move move_from_str(const char *move_str, const board *board) {
  // separate src, dst, and promotion parts of string
  char src_str[3];
  memcpy(src_str, move_str, 2 * sizeof(char));
  src_str[2] = '\0';
  char dst_str[3];
  memcpy(dst_str, move_str + 2, 2 * sizeof(char));
  dst_str[2] = '\0';
  // handle promotion char if present
  int is_promote = 0;
  char promote_char;
  int promote_piece = -1;
  if (move_str[4] != '\0') {
    is_promote = 1;
    promote_char = move_str[4];
    for (int p = 0; p < 6; p++) {
      if (promote_codes[p] == promote_char)
        promote_piece = p;
    }
    if (promote_piece == -1) {
      return MOVE_END;
    }
  }
  // parse src + dst
  board_pos src = board_pos_from_str(src_str);
  board_pos dst = board_pos_from_str(dst_str);

  return move_new(src, dst, is_promote, promote_piece, board);
}

bitboard board_is_square_attacked(const board *board, board_pos square,
                                  int attacking_player) {
  assert(attacking_player == WHITE || attacking_player == BLACK);
  int defending_player = !attacking_player;
  // hits on attacking pieces
  bitboard attack_hits = 0;
  bitboard attackers_mask = board->players[attacking_player];
  bitboard occ_slide = board_occupancy_for_sliders_lookups(board);
  bitboard occ_pawn = board_occupancy_for_pawns_lookups(board);
  // in order to find attacks, treat the square as a piece of each type and
  // check the intersection of legal moves and the opponent
  for (int piece = KING; piece <= KNIGHT; piece++) {
    assert(piece == KING || piece == PAWN || piece == KNIGHT);
    attack_hits |= move_gen_reg_moves_mask(occ_slide, occ_pawn, piece,
                                           defending_player, square) &
                   (board->pieces[piece]);
  }
  for (int piece = ROOK; piece <= BISHOP; piece++) {
    assert(piece == ROOK || piece == BISHOP);
    attack_hits |= move_gen_reg_moves_mask(occ_slide, occ_pawn, piece,
                                           defending_player, square) &
                   (board->pieces[piece] | board->pieces[QUEEN]);
  }
  attack_hits &= attackers_mask;
  return attack_hits;
}

/**
 * clear the castling rights for player on piece side for board */
static void board_clear_castling(board *board, int player, int side) {
  assert(side == QUEEN || side == KING);
  int flag = player == WHITE ? (side == QUEEN ? BOARD_FLAGS_W_CASTLE_QUEEN
                                              : BOARD_FLAGS_W_CASTLE_KING)
                             : (side == QUEEN ? BOARD_FLAGS_B_CASTLE_QUEEN
                                              : BOARD_FLAGS_B_CASTLE_KING);
  board->flags &= ~(flag);
}

static void move_gen_make_castle(board *board, move move) {
  board_pos dst = move_destination_square(move);
  board_pos src = move_source_square(move);
  int player = board_player_to_move(board);
  int side = board_pos_to_x(dst) == 2 ? QUEEN : KING;
  assert(board_pos_to_x(dst) == 2 || board_pos_to_x(dst) == 6);
  assert(board_piece_on_square(board, src) == KING);
  assert(!move_is_capture(move));
  assert(!move_is_promotion(move));

  int y = player == WHITE ? 0 : 7;
  assert(board_pos_to_y(dst) == y);
  // move king
  board->players[player] = bitboard_clear_square(board->players[player], src);
  board->pieces[KING] = bitboard_clear_square(board->pieces[KING], src);
  board->players[player] = bitboard_set_square(board->players[player], dst);
  board->pieces[KING] = bitboard_set_square(board->pieces[KING], dst);
  // move rook
  board_pos rook_src =
      side == QUEEN ? board_pos_from_xy(0, y) : board_pos_from_xy(7, y);
  board_pos rook_dst =
      side == QUEEN ? board_pos_from_xy(3, y) : board_pos_from_xy(5, y);
  board->players[player] =
      bitboard_clear_square(board->players[player], rook_src);
  board->pieces[ROOK] = bitboard_clear_square(board->pieces[ROOK], rook_src);
  board->players[player] =
      bitboard_set_square(board->players[player], rook_dst);
  board->pieces[ROOK] = bitboard_set_square(board->pieces[ROOK], rook_dst);

  board_clear_castling(board, player, QUEEN);
  board_clear_castling(board, player, KING);
}

void board_make_move(board *board, move move) {
  board_invariants(board);
  assert((board->flags & BOARD_FLAGS_LOW) == (move & MOVE_FLAGS_PREV_FLAGS));
  board_pos src = move_source_square(move);
  board_pos dst = move_destination_square(move);
  int piece = board_piece_on_square(board, src);
  int dst_piece = move_is_promotion(move) ? move_promotion_piece(move) : piece;
  int player = board_player_to_move(board);
  int opponent = !player;

  if (move_is_castle(move)) {
    move_gen_make_castle(board, move);
  } else {
    assert(piece != -1);
    assert(!bitboard_check_square(board->players[opponent], dst) ||
           move_is_capture(move));
    // revoke both sides castling rights if king is moved
    if (piece == KING) {
      board_clear_castling(board, player, KING);
      board_clear_castling(board, player, QUEEN);
    }
    // if rooks are moved, revoke castling rights
    if (piece == ROOK) {
      if (player == WHITE && src == board_pos_from_xy(0, 0)) {
        board_clear_castling(board, WHITE, QUEEN);
      } else if (player == WHITE && src == board_pos_from_xy(7, 0)) {
        board_clear_castling(board, WHITE, KING);
      } else if (player == BLACK && src == board_pos_from_xy(0, 7)) {
        board_clear_castling(board, BLACK, QUEEN);
      } else if (player == BLACK && src == board_pos_from_xy(7, 7)) {
        board_clear_castling(board, BLACK, KING);
      }
    }
    // if move is capture, clear dst for opponent
    if (move_is_capture(move)) {
      board_pos cap_square = move_capture_square(move);
      // en passant capture
      board_pos ep_target = board_get_en_passant_target(board);
      if (ep_target != BOARD_POS_INVALID && ep_target == dst) {
        cap_square = en_passant_target_to_pawn_pos(ep_target);
      }
      int cap_piece = board_piece_on_square(board, cap_square);
      assert(cap_piece != -1);
      assert(cap_square != src);
      assert(board_player_on_square(board, cap_square) != player);
      board->players[opponent] =
          bitboard_clear_square(board->players[opponent], cap_square);
      board->pieces[cap_piece] =
          bitboard_clear_square(board->pieces[cap_piece], cap_square);

      // if rooks are captured on initial squares, revoke castling rights
      if (cap_piece == ROOK) {
        if (opponent == WHITE && cap_square == board_pos_from_xy(0, 0)) {
          board_clear_castling(board, WHITE, QUEEN);
        } else if (opponent == WHITE && cap_square == board_pos_from_xy(7, 0)) {
          board_clear_castling(board, WHITE, KING);
        } else if (opponent == BLACK && cap_square == board_pos_from_xy(0, 7)) {
          board_clear_castling(board, BLACK, QUEEN);
        } else if (opponent == BLACK && cap_square == board_pos_from_xy(7, 7)) {
          board_clear_castling(board, BLACK, KING);
        }
      }
    }
    // move piece from src to dst and clear src
    board->pieces[dst_piece] =
        bitboard_set_square(board->pieces[dst_piece], dst);
    board->players[player] = bitboard_set_square(board->players[player], dst);
    board->pieces[piece] = bitboard_clear_square(board->pieces[piece], src);
    board->players[player] = bitboard_clear_square(board->players[player], src);
  }
  // clear en passant target
  board->flags &= ~(BOARD_FLAGS_EP_PRESENT);
  // set en passant target if move is double pawn push
  if (piece == PAWN && ((src - dst) == 16 || (dst - src) == 16)) {
#ifndef NDEBUG
    int x, y;
    board_pos_to_xy(src, &x, &y);
    assert((board_player_to_move(board) == WHITE && y == 1) ||
           (board_player_to_move(board) == BLACK && y == 6));
#endif
    // generate en passant target (one behind square)
    board_pos ep_target;
    if (dst > src) {
      ep_target = src + 8;
    } else {
      ep_target = src - 8;
    }
    board->flags |= BOARD_FLAGS_EP_PRESENT;
    board->flags &= ~BOARD_FLAGS_EP_SQUARE;
    board->flags |= (ep_target & BOARD_FLAGS_EP_SQUARE);
  }

  // if player moving is black, inc turn number
  if(player == BLACK) {
    int prev_turn = board_get_full_turn_number(board);
    // clear turn
    board->flags &= BOARD_FLAGS_LOW;
    // set turn
    board -> flags |= ((prev_turn + 1) << BOARD_FLAGS_TURN_NUM_SHIFT) & BOARD_FLAGS_TURN_NUM;
  }
  // flip player to move
  board->flags ^= BOARD_FLAGS_TURN;
  board_invariants(board);
}

void board_unmake_move(board *board, move move) {
  board_invariants(board);
  // restore flags
  board->flags &= ~BOARD_FLAGS_LOW;
  board->flags |= move & MOVE_FLAGS_PREV_FLAGS;
  // extract info from moves
  board_pos src = move_source_square(move);
  board_pos dst = move_destination_square(move);
  int piece_dst = board_piece_on_square(board, dst);
  int piece_src = move_is_promotion(move) ? PAWN : piece_dst;
  assert(piece_dst != -1);
  // player is the player that made the move
  int player = board_player_to_move(board);
  int opponent = !player;

  // if player making move was black, dec turn number
  if(player == BLACK) {
    int prev_turn = board_get_full_turn_number(board);
    // clear turn
    board->flags &= BOARD_FLAGS_LOW;
    // set turn
    board -> flags |= ((prev_turn - 1) << BOARD_FLAGS_TURN_NUM_SHIFT) & BOARD_FLAGS_TURN_NUM;
  }

  // move dst to src
  board->pieces[piece_dst] =
      bitboard_clear_square(board->pieces[piece_dst], dst);
  board->players[player] = bitboard_clear_square(board->players[player], dst);
  board->pieces[piece_src] = bitboard_set_square(board->pieces[piece_src], src);
  board->players[player] = bitboard_set_square(board->players[player], src);
  // restore captures
  if (move_is_capture(move)) {
    int cap_piece = move_capture_piece(move);
    int cap_square = move_capture_square(move);
    board->pieces[cap_piece] =
        bitboard_set_square(board->pieces[cap_piece], cap_square);
    board->players[opponent] =
        bitboard_set_square(board->players[opponent], cap_square);
  }
  // if castling, move rook back
  if (move_is_castle(move)) {
    int side = board_pos_to_x(dst) == 2 ? QUEEN : KING;
    int rook_y = player == WHITE ? 0 : 7;
    int rook_src_x = side == QUEEN ? 0 : 7;
    int rook_dst_x = side == QUEEN ? 3 : 5;
    board_pos rook_src = board_pos_from_xy(rook_src_x, rook_y);
    board_pos rook_dst = board_pos_from_xy(rook_dst_x, rook_y);
    // set src and clear dst
    board->players[player] =
        bitboard_set_square(board->players[player], rook_src);
    board->pieces[ROOK] = bitboard_set_square(board->pieces[ROOK], rook_src);
    board->players[player] =
        bitboard_clear_square(board->players[player], rook_dst);
    board->pieces[ROOK] = bitboard_clear_square(board->pieces[ROOK], rook_dst);
  }
  board_invariants(board);
}

/**
 * advance the move generator to the next promotion type */
static void move_gen_next_promote(move_gen *generator) {
  generator->cur_promotion++;
  if (generator->cur_promotion > QUEEN) {
    generator->cur_promotion = KNIGHT;
  }
}

static move move_gen_next_from_cur_moves(move_gen *generator) {
  assert(generator->cur_moves);
  board_pos dst = bitboard_scan_lsb(generator->cur_moves);
  // check for promotion
  int is_promote = 0;
  if (generator->cur_piece_type == PAWN &&
      ((dst >> 3) == 0 || (dst >> 3) == 7)) {
    is_promote = 1;
  }
  // if promotion, only clear board if all promotions have been generated
  if (!is_promote || generator->cur_promotion == QUEEN) {
    generator->cur_moves = bitboard_clear_square(generator->cur_moves, dst);
  }

  int player = board_player_to_move(generator->board);
  int opponent = !player;
  // check if move is a capture
  int is_capture = 0;
  int capture_piece = 0;
  board_pos capture_pos = 0;
  if (bitboard_check_square(generator->board->players[opponent], dst)) {
    is_capture = 1;
    capture_piece = board_piece_on_square(generator->board, dst);
    assert(capture_piece != -1);
    capture_pos = dst;
  }
  // check en passant
  if (generator->cur_piece_type == PAWN) {
    board_pos ep_target = board_get_en_passant_target(generator->board);
    if (ep_target != BOARD_POS_INVALID && dst == ep_target) {
      is_capture = 1;
      capture_piece = PAWN;
      capture_pos = en_passant_target_to_pawn_pos(ep_target);
      assert(board_piece_on_square(generator->board, capture_pos) == PAWN);
    }
  }

  move res = construct_move(generator->board->flags, generator->cur_square, dst,
                            is_promote, generator->cur_promotion, is_capture,
                            capture_piece, capture_pos, 0);
  // move to next promotion type
  if (is_promote) {
    move_gen_next_promote(generator);
  }
  return res;
}

bitboard board_player_in_check(const board *board, int player) {
  assert(player == WHITE || player == BLACK);
  bitboard king_mask = board->pieces[KING] & board->players[player];
  assert(bitboard_popcount(king_mask) == 1);
  board_pos king_pos = bitboard_scan_lsb(king_mask);
  return board_is_square_attacked(board, king_pos, !player);
}

/**
 * generate the castling move for side, or return MOVE_END if side can't castle
 */
static move move_gen_castle(move_gen *generator, int player, int side,
                            int undo_move) {
  if (!board_can_castle(generator->board, player, side)) {
    return MOVE_END;
  }
  int y = player == WHITE ? 0 : 7;
  // direction from king to rook
  int dir = side == QUEEN ? -1 : 1;
  // make sure king and rook positions are in expected
  board_pos king = board_pos_from_xy(4, y);
  board_pos rook =
      side == QUEEN ? board_pos_from_xy(0, y) : board_pos_from_xy(7, y);
  assert(board_piece_on_square(generator->board, king) == KING);
  assert(board_piece_on_square(generator->board, rook) == ROOK);
  assert(board_player_on_square(generator->board, king) == player);
  assert(board_player_on_square(generator->board, rook) == player);

  // check for pieces between king and rook
  for (int x = board_pos_to_x(king) + dir; x != board_pos_to_x(rook);
       x += dir) {
    if (bitboard_check_square(generator->occupancy_for_sliders,
                              board_pos_from_xy(x, y)))
      return MOVE_END;
  }
  // make sure square from king until dest aren't threatened
  int x = board_pos_to_x(king);
  for (int i = 0; i < 3; i++, x += dir) {
    if (board_is_square_attacked(generator->board, board_pos_from_xy(x, y),
                                 !player)) {
      return MOVE_END;
    }
  }

  move move = construct_move(
      generator->board->flags, king,
      board_pos_from_xy(board_pos_to_x(king) + 2 * dir, y), 0, 0, 0, 0, 0, 1);
  // make move if needed
  if (!undo_move) {
    board_make_move(generator->board, move);
    assert(!board_player_in_check(generator->board, player));
  }
  return move;
}

#define MOVE_DONE_NORMAL 1
#define MOVE_DONE_CHECKMATE 2
#define MOVE_DONE_STALEMATE 3

static move move_gen_next(move_gen *generator, int undo_moves) {
  board_invariants(generator->board);
  int player = board_player_to_move(generator->board);
  int opponent = !player;
  bitboard player_mask = generator->board->players[player];

  if (generator->cur_mode == MOVE_GEN_MODE_END) {
    // check for checkmate or stalemate
    if (generator->hit_move) {
      generator->done = MOVE_DONE_NORMAL;
    } else {
      if (board_player_in_check(generator->board, player)) {
        generator->done = MOVE_DONE_CHECKMATE;
      } else {
        generator->done = MOVE_DONE_STALEMATE;
      }
    }
    return MOVE_END;
  } else if (generator->cur_mode == MOVE_GEN_MODE_CASTLE_KING) {
    generator->cur_mode = MOVE_GEN_MODE_CASTLE_QUEEN;
    move castle = move_gen_castle(generator, player, KING, undo_moves);
    if (castle != MOVE_END) {
      generator->hit_move = 1;
      return castle;
    } else {
      return move_gen_next(generator, undo_moves);
    }
  } else if (generator->cur_mode == MOVE_GEN_MODE_CASTLE_QUEEN) {
    generator->cur_mode = MOVE_GEN_MODE_END;
    move castle = move_gen_castle(generator, player, QUEEN, undo_moves);
    if (castle != MOVE_END) {
      generator->hit_move = 1;
      return castle;
    } else {
      return move_gen_next(generator, undo_moves);
    }
  } else if (generator->cur_mode == MOVE_GEN_MODE_NORMAL) {
    // if moves are remaining in bitboard, return them
    if (generator->cur_moves) {
      move next_move = move_gen_next_from_cur_moves(generator);
      // make move
      board_make_move(generator->board, next_move);
      if (board_player_in_check(generator->board, player)) {
        board_unmake_move(generator->board, next_move);
        return move_gen_next(generator, undo_moves);
      }

      if (undo_moves) {
        board_unmake_move(generator->board, next_move);
      }
      generator->hit_move = 1;
      return next_move;
    }
    // otherwise, increment to the next square
    do {
      generator->cur_square++;
      if (generator->cur_square >= 64) {
        generator->cur_square = 0;
        generator->cur_piece_type++;
      }
      // if we reached the end of all the pieces, move to castles
      if (generator->cur_piece_type > QUEEN) {
        generator->cur_mode = MOVE_GEN_MODE_CASTLE_KING;
        return move_gen_next(generator, undo_moves);
      }
    } while (!bitboard_check_square(
        generator->board->pieces[generator->cur_piece_type] & player_mask,
        generator->cur_square));

    // generate move mask for the current square and piece type
    generator->cur_moves =
        move_gen_reg_moves_mask(
            generator->occupancy_for_sliders, generator->occupancy_for_pawns,
            generator->cur_piece_type, player, generator->cur_square) &
        generator->final_moves_mask;
    return move_gen_next(generator, undo_moves);
  } else {
    assert(0);
  }
}

move move_gen_next_move(move_gen *generator) {
  return move_gen_next(generator, 1);
}

move move_gen_make_next_move(move_gen *generator) {
  return move_gen_next(generator, 0);
}

int move_gen_is_checkmate(move_gen *const move_gen) {
  if (!move_gen->done) {
    fprintf(stderr, "[chess-util]: error: move_gen_is_checkmate can only be "
                    "called once the move_gen has been fully exhausted\n");
    assert(0);
  }
  return move_gen->done == MOVE_DONE_CHECKMATE;
}

int move_gen_is_stalemate(move_gen *const move_gen) {
  if (!move_gen->done) {
    fprintf(stderr, "[chess-util]: error: move_gen_is_stalemate can only be "
                    "called once the move_gen has been fully exhausted\n");
    assert(0);
  }
  return move_gen->done == MOVE_DONE_STALEMATE;
}

int board_is_checkmate(board *const board_to_check) {
  move_gen gen;
  move_gen_init(&gen, (board *)board_to_check);
  move move;
  while ((move = move_gen_next_move(&gen)) != MOVE_END)
    ;
  return move_gen_is_checkmate(&gen);
}

int board_is_stalemate(board *const board_to_check) {
  move_gen gen;
  move_gen_init(&gen, (board*)board_to_check);
  move move;
  while ((move = move_gen_next_move(&gen)) != MOVE_END)
    ;
  return move_gen_is_stalemate(&gen);
}

int moves_equal(move move0, move move1) {
  if ((move0 & MOVE_FLAGS_PREV_FLAGS) != (move1 & MOVE_FLAGS_PREV_FLAGS))
    return 0;

  if (move_source_square(move0) != move_source_square(move1) ||
      move_destination_square(move0) != move_destination_square(move1))
    return 0;

  if (move_is_promotion(move0) != move_is_promotion(move1) ||
      (move_is_promotion(move0) &&
       move_promotion_piece(move0) != move_promotion_piece(move1)))
    return 0;

  if (move_is_capture(move0) != move_is_capture(move1) ||
      (move_is_capture(move0) &&
       (move_capture_piece(move0) != move_capture_piece(move1) ||
        move_capture_square(move0) != move_capture_square(move1))))
    return 0;

  if (move_is_castle(move0) != move_is_castle(move1))
    return 0;

  return 1;
}

int move_is_legal(move move_to_check, board *board) {
  if (move_to_check == MOVE_END) {
    return 0;
  }
  move_gen gen;
  move_gen_init(&gen, board);
  move cur;
  while ((cur = move_gen_next_move(&gen)) != MOVE_END) {
    if (moves_equal(move_to_check, cur))
      return 1;
  }

  return 0;
}

#if defined __has_attribute
# if __has_attribute(constructor)
  __attribute__ ((constructor))
# endif
#endif
void move_gen_pregenerate() {
  static int run = 0;
  if(run) {
    printf("[chess-util] move generation pre-calculations already ran\n");
    return;
  }
  run = 1;
#ifndef NDEBUG
  printf("[chess-util] chess-util was built with assertions\n");
#else
  printf("[chess-util] chess-util was not built with assertions\n");
#endif
  printf("[chess-util] beginning move generation pre-calculations\n");
  move_gen_init_knights();
  move_gen_init_kings();
  move_gen_init_sliders();
  move_gen_init_pawns();
  printf("[chess-util] move generation pre-calculations done\n");
}
