// Windows MSVC fixes
#ifdef _MSC_VER
#include <intrin.h>
#include <nmmintrin.h>
#define __builtin_popcountll _mm_popcnt_u64
static inline int __builtin_ctzll(unsigned long long x) {
  unsigned long ret;
  _BitScanForward64(&ret, x);
  return (int)ret;
}
#endif

#include "chess-util.h"
#include <assert.h>
#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>

int bitboard_popcount(bitboard board) { return __builtin_popcountll(board); }

int bitboard_scan_lsb(bitboard board) { return __builtin_ctzll(board); }

int bitboard_check_square(bitboard board, board_pos square) {
  assert(square < 64);
  return (board >> square) & 1;
}

bitboard bitboard_set_square(bitboard board, board_pos square) {
  assert(square < 64);
  return board | (1ULL << square);
}

bitboard bitboard_clear_square(bitboard board, board_pos square) {
  assert(square < 64);
  return board & ~(1ULL << square);
}

bitboard bitboard_flip_square(bitboard board, board_pos square) {
  assert(square < 64);
  return board ^ (1ULL << square);
}

/* --- bitboard shifting methods --- */
bitboard bitboard_shift_n(bitboard board) { return board << 8ULL; }

bitboard bitboard_shift_s(bitboard board) { return board >> 8ULL; }

// for east and west shifting, mask needed to prevent edges pieces from moving
// up or down a row
#define shift_w_mask 0xfefefefefefefefeULL
#define shift_e_mask 0x7f7f7f7f7f7f7f7fULL

bitboard bitboard_shift_w(bitboard board) {
  return (board << 1ULL) & shift_w_mask;
}

bitboard bitboard_shift_e(bitboard board) {
  return (board >> 1ULL) & shift_e_mask;
}

bitboard bitboard_shift_nw(bitboard board) {
  return bitboard_shift_w(bitboard_shift_n(board));
}

bitboard bitboard_shift_ne(bitboard board) {
  return bitboard_shift_e(bitboard_shift_n(board));
}

bitboard bitboard_shift_sw(bitboard board) {
  return bitboard_shift_w(bitboard_shift_s(board));
}

bitboard bitboard_shift_se(bitboard board) {
  return bitboard_shift_e(bitboard_shift_s(board));
}

static void bitboard_print_col_header() {
  printf("   ");
  for (int x = 0; x < 8; x++) {
    printf(" %c", 'a' + x);
  }
  printf("    \n");
}

static void bitboard_print_col_seperator() {
  printf("   -----------------   \n");
}

void bitboard_print(bitboard board) {
  bitboard_print_col_header();
  bitboard_print_col_seperator();
  for (int y = 7; y >= 0; y--) {
    printf("%c |", '1' + y);
    for (int x = 0; x < 8; x++) {
      board_pos pos = board_pos_from_xy(x, y);
      printf(" %c", bitboard_check_square(board, pos) ? '1' : '0');
    }
    printf(" | %c\n", '1' + y);
  }
  bitboard_print_col_seperator();
  bitboard_print_col_header();
}

void bitboard_print_pretty(bitboard board) {
  bitboard_print_col_header();
  printf("  ┌─────────────────┐\n");
  for (int y = 7; y >= 0; y--) {
    printf("%c │", '1' + y);
    for (int x = 0; x < 8; x++) {

      board_pos pos = board_pos_from_xy(x, y);
      if (bitboard_check_square(board, pos)) {
        printf("\x1b[1;31m 1\x1b[m");
      } else {
        printf(" 0");
      }
    }
    printf(" │ %c\n", '1' + y);
  }
  printf("  └─────────────────┘\n");
  bitboard_print_col_header();
}

board_pos board_pos_from_xy(int x, int y) {
  if (x < 0 || y < 0 || x >= 8 || y >= 8) {
    return BOARD_POS_INVALID;
  } else {
    return x + (y << 3);
  }
}

void board_pos_to_xy(board_pos pos, int *x, int *y) {
  assert(pos < 64);
  *x = pos & 0x07;
  *y = (pos >> 3) & 0x07;
}

int board_pos_to_x(board_pos pos) { return pos & 0x07; }

int board_pos_to_y(board_pos pos) { return (pos >> 3) & 0x07; }

void board_pos_to_str(board_pos pos, char *str) {
  assert(pos < 64);
  int x, y;
  board_pos_to_xy(pos, &x, &y);
  str[0] = x + 'a';
  str[1] = y + '1';
  str[2] = '\0';
}

board_pos board_pos_from_str(const char *str) {
  int x, y;
  if (str[0] >= 'a' && str[0] <= 'h') {
    x = str[0] - 'a';
  } else if (str[0] >= 'A' && str[0] <= 'H') {
    x = str[0] - 'A';
  } else {
    assert(0);
  }

  assert(str[1] >= '1' && str[1] <= '8');
  y = str[1] - '1';

  assert(str[2] == '\0');

  return board_pos_from_xy(x, y);
}

void board_invariants(const board *board) {
#ifndef NDEBUG
  // make sure player occupancy don't overlap
  assert((board->players[0] & board->players[1]) == 0);
  for (int i = 0; i < 6; i++) {
    for (int j = 0; j < 6; j++) {
      if (i == j)
        continue;
      assert((board->pieces[i] & board->pieces[j]) == 0);
    }
  }
  // make sure pieces don't overlap
  assert(bitboard_popcount(board->players[0] & board->pieces[KING]) == 1);
  assert(bitboard_popcount(board->players[1] & board->pieces[KING]) == 1);
  // make sure ep target square is empty if present and on rank 3 or 6
  if (board->flags & BOARD_FLAGS_EP_PRESENT) {
    int player_to_move = board_player_to_move(board);
    board_pos ep_target = board->flags & BOARD_FLAGS_EP_SQUARE;
    int x, y;
    board_pos_to_xy(ep_target, &x, &y);
    assert((player_to_move == WHITE && y == 5) ||
           (player_to_move == BLACK && y == 2));
    assert(!bitboard_check_square(board->players[0] & board->players[1],
                                  ep_target));
  }
#endif
}

int board_piece_char_to_piece(char c) {
  switch (c) {
  case 'P':
  case 'p':
    return PAWN;
  case 'N':
  case 'n':
    return KNIGHT;
  case 'B':
  case 'b':
    return BISHOP;
  case 'R':
  case 'r':
    return ROOK;
  case 'Q':
  case 'q':
    return QUEEN;
  case 'k':
  case 'K':
    return KING;
  default:
    assert(0);
  }
}

int board_piece_char_to_player(char c) {
  switch (c) {
  case 'P':
  case 'N':
  case 'B':
  case 'R':
  case 'Q':
  case 'K':
    return WHITE;
  case 'p':
  case 'n':
  case 'b':
  case 'r':
  case 'q':
  case 'k':
    return BLACK;
  default:
    assert(0);
  }
}

static char piece_str[2][6] = {
    {'K', 'P', 'N', 'R', 'B', 'Q'}, // white
    {'k', 'p', 'n', 'r', 'b', 'q'}  // black
};

char board_piece_char_from_piece_player(int piece, int player) {
  assert(player == WHITE || player == BLACK);
  assert(piece >= 0 && piece < 6);
  return piece_str[player][piece];
}

void board_from_fen_str(board *board, const char *fen_string) {
  // clear board
  for (int i = 0; i < 2; i++) {
    board->players[i] = 0L;
  }
  for (int i = 0; i < 6; i++) {
    board->pieces[i] = 0L;
  }
  board->flags = 0;

  // read piece locations
  for (int rank = 7; rank >= 0; rank--) {
    int file = 0;
    while (*fen_string != '/' && !isspace(*fen_string)) {
      if (*fen_string >= '1' && *fen_string <= '8') {
        file += *fen_string - '0';
      } else {
        int player = board_piece_char_to_player(*fen_string);
        int piece = board_piece_char_to_piece(*fen_string);
        board->players[player] = bitboard_set_square(
            board->players[player], board_pos_from_xy(file, rank));
        board->pieces[piece] = bitboard_set_square(
            board->pieces[piece], board_pos_from_xy(file, rank));
        file++;
      }
      fen_string++;
    }
    fen_string++;
  }

  while (isspace(*fen_string)) {
    fen_string++;
  }

  // read player to move
  if (*fen_string == 'b') {
    board->flags |= BOARD_FLAGS_TURN;
  } else if (*fen_string == 'w') {
    board->flags &= ~BOARD_FLAGS_TURN;
  } else {
    assert(0);
  }
  fen_string++;

  while (isspace(*fen_string)) {
    fen_string++;
  }

  // read castling availability
  while (*fen_string == 'K' || *fen_string == 'Q' || *fen_string == 'k' ||
         *fen_string == 'q' || *fen_string == '-') {
    switch (*fen_string) {
    case 'K':
      board->flags |= BOARD_FLAGS_W_CASTLE_KING;
      break;
    case 'Q':
      board->flags |= BOARD_FLAGS_W_CASTLE_QUEEN;
      break;
    case 'k':
      board->flags |= BOARD_FLAGS_B_CASTLE_KING;
      break;
    case 'q':
      board->flags |= BOARD_FLAGS_B_CASTLE_QUEEN;
      break;
    case '-':
    default:
      break;
    }
    fen_string++;
  }

  while (isspace(*fen_string)) {
    fen_string++;
  }

  // read en passant target
  if (*fen_string == '-') {
    board->flags &= ~BOARD_FLAGS_EP_PRESENT;
    fen_string++;
  } else {
    char square_str[3];
    square_str[0] = *fen_string++;
    square_str[1] = *fen_string++;
    square_str[2] = '\0';
    board->flags |= BOARD_FLAGS_EP_PRESENT;
    board->flags |= board_pos_from_str(square_str) & BOARD_FLAGS_EP_SQUARE;
  }

  // ignore halfmove
  while(isspace(*fen_string)) {
    fen_string++;
  }
  while(!isspace(*fen_string)) {
    fen_string++;
  }
  while(isspace(*fen_string)) {
    fen_string++;
  }
  // read turn counter
  char turn_count_str[6];
  int turn_count_index = 0;
  while(*fen_string != '\0' && !isspace(*fen_string) && turn_count_index < 5) {
    turn_count_str[turn_count_index] = *fen_string;
    turn_count_index++;
    fen_string++;
  }
  turn_count_str[turn_count_index] = '\0';

  int turn_count = atoi(turn_count_str);
  board->flags |= (turn_count << BOARD_FLAGS_TURN_NUM_SHIFT) & BOARD_FLAGS_TURN_NUM;

  board_invariants(board);
}

/**
 * convert castling rights to a string
 * castling_str must have 5 bytes allocated */
static void board_castling_to_str(const board *board, char *castling_str) {
  char *castling = castling_str;
  if (board->flags & BOARD_FLAGS_W_CASTLE_KING) {
    *castling++ = 'K';
  }
  if (board->flags & BOARD_FLAGS_W_CASTLE_QUEEN) {
    *castling++ = 'Q';
  }
  if (board->flags & BOARD_FLAGS_B_CASTLE_KING) {
    *castling++ = 'k';
  }
  if (board->flags & BOARD_FLAGS_B_CASTLE_QUEEN) {
    *castling++ = 'q';
  }
  if (castling == castling_str) {
    *castling++ = '-';
  }
  *castling = '\0';
}

/**
 * convert en passant square to string
 * ep_str must have three bytes allocated */
static void board_ep_to_str(const board *board, char *ep_str) {
  if (board->flags & BOARD_FLAGS_EP_PRESENT) {
    board_pos_to_str(board->flags & BOARD_FLAGS_EP_SQUARE, ep_str);
  } else {
    *ep_str++ = '-';
    *ep_str++ = '\0';
  }
}

void board_to_fen_str(const board *board, char *res_str) {
  board_invariants(board);
  for (int y = 7; y >= 0; y--) {
    int empty_counter = 0;
    for (int x = 0; x < 8; x++) {
      int player = board_player_on_square(board, board_pos_from_xy(x, y));
      int piece = board_piece_on_square(board, board_pos_from_xy(x, y));
      if (player == -1) {
        empty_counter++;
      } else {
        if (empty_counter > 0) {
          assert(empty_counter <= 8);
          *res_str++ = empty_counter + '0';
          empty_counter = 0;
        }
        *res_str++ = board_piece_char_from_piece_player(piece, player);
      }
    }
    if (empty_counter > 0) {
      assert(empty_counter <= 8);
      *res_str++ = empty_counter + '0';
      empty_counter = 0;
    }
    if (y > 0) {
      *res_str++ = '/';
    }
  }

  *res_str++ = ' ';
  if (board_player_to_move(board) == WHITE)
    *res_str++ = 'w';
  else
    *res_str++ = 'b';

  *res_str++ = ' ';
  board_castling_to_str(board, res_str);
  while (*res_str != '\0') {
    res_str++;
  }
  *res_str++ = ' ';

  board_ep_to_str(board, res_str);
  while (*res_str != '\0') {
    res_str++;
  }
  *res_str++ = ' ';
  // fake halfmove counter
  *res_str++ = '0';
  *res_str++ = ' ';
  // real full turn counter
  snprintf(res_str, 4, "%i", board_get_full_turn_number(board));
}

int board_get_full_turn_number(const board *board) {
  return (board->flags & BOARD_FLAGS_TURN_NUM) >> BOARD_FLAGS_TURN_NUM_SHIFT;
}

int board_player_to_move(const board *board) {
  return board->flags & BOARD_FLAGS_TURN ? 1 : 0;
}

board_pos board_get_en_passant_target(const board *board) {
  if (!(board->flags & BOARD_FLAGS_EP_PRESENT)) {
    return BOARD_POS_INVALID;
  } else {
    return board->flags & BOARD_FLAGS_EP_SQUARE;
  }
}

int board_can_castle(const board *board, int player, int side) {
  assert(player == WHITE || player == BLACK);
  assert(side == KING || side == QUEEN);

  if (player == WHITE && side == KING) {
    return board->flags & BOARD_FLAGS_W_CASTLE_KING ? 1 : 0;
  } else if (player == WHITE && side == QUEEN) {
    return board->flags & BOARD_FLAGS_W_CASTLE_QUEEN ? 1 : 0;
  } else if (player == BLACK && side == KING) {
    return board->flags & BOARD_FLAGS_B_CASTLE_KING ? 1 : 0;
  } else if (player == BLACK && side == QUEEN) {
    return board->flags & BOARD_FLAGS_B_CASTLE_QUEEN ? 1 : 0;
  }

  assert(0);
}

/**
 * print player to move, castling rights, and en passant information */
static void board_print_flags(const board *board) {
  char castling_str[5];
  board_castling_to_str(board, castling_str);
  char ep_str[3];
  board_ep_to_str(board, ep_str);

  printf("move: %s, castling: %s, ep target: %s, turn number (full turns): %i\n",
         board->flags & BOARD_FLAGS_TURN ? "black" : "white", castling_str,
         ep_str, board_get_full_turn_number(board));
  printf("=======================\n");
}

int board_piece_on_square(const board *board, board_pos square) {
  assert(square < 64);
  for (int p = 0; p < 6; p++) {
    if (bitboard_check_square(board->pieces[p], square)) {
      return p;
    }
  }

  return -1;
}

int board_player_on_square(const board *board, board_pos square) {
  assert(square < 64);
  for (int p = 0; p < 2; p++) {
    if (bitboard_check_square(board->players[p], square)) {
      return p;
    }
  }

  return -1;
}

void board_print(const board *board) {
  bitboard_print_col_header();
  bitboard_print_col_seperator();
  for (int y = 7; y >= 0; y--) {
    printf("%c |", '1' + y);
    for (int x = 0; x < 8; x++) {
      board_pos pos = board_pos_from_xy(x, y);
      char piece_char = '.';
      int piece = board_piece_on_square(board, pos);
      int player = board_player_on_square(board, pos);
      if (piece != -1) {
        piece_char = board_piece_char_from_piece_player(piece, player);
      }
      if (player == BLACK) {
        printf("\x1b[31m");
      }
      printf(" %c", piece_char);
      if (player == BLACK) {
        printf("\x1b[m");
      }
    }
    printf(" | %c\n", '1' + y);
  }
  bitboard_print_col_seperator();
  bitboard_print_col_header();
  board_print_flags(board);
}

static char *utf8_pieces[2][6] = {
    {"♚", "♟︎", "♞", "♜", "♝", "♛"}, // black
    {"♔", "♙", "♘", "♖", "♗", "♕"}       // white
};

void board_print_pretty(const board *board) {
  bitboard_print_col_header();
  printf("  ┌─────────────────┐  \n");
  for (int y = 7; y >= 0; y--) {
    printf("%c │\x1b[97m", '1' + y);
    for (int x = 0; x < 8; x++) {
      board_pos pos = board_pos_from_xy(x, y);
      int player = board_player_on_square(board, pos);
      int piece = board_piece_on_square(board, pos);
      if (player == -1) {
        printf(" .");
      } else {
        printf(" %s", utf8_pieces[player][piece]);
      }
    }
    printf(" \x1b[m│ %c\n", '1' + y);
  }
  printf("  └─────────────────┘  \n");
  bitboard_print_col_header();
  board_print_flags(board);
}
