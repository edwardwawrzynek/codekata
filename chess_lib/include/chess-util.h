#ifndef H_CHESS_INCL
#define H_CHESS_INCL

#include "stdint.h"

#ifndef H_CHESS_UTIL_TYPEDEFS_INCL
#define H_CHESS_UTIL_TYPEDEFS_INCL

typedef uint64_t __chess_util_bitboard;
typedef uint8_t __chess_util_board_pos;

struct __chess_util_board {
  __chess_util_bitboard players[2];
  __chess_util_bitboard pieces[6];
  uint32_t flags;
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

// bitboard definition + utilities

/**
 * A bitboard is a data structure mapping one bit to one square on the board.
 * For chess, a bitboard stores a single boolean for each square.
 *
 * Implemented as a single 64-bit word. Squares map to bits in little endian
 * order:
 *    a  b  c  d  e  f  g  h
 *   ------------------------
 * 8| 56 57 58 59 60 61 62 63
 * 7| 48 49 50 51 52 53 54 55
 * 6| 40 41 42 43 44 45 46 47
 * 5| 32 33 34 35 36 37 38 39
 * 4| 24 25 26 27 28 29 30 31
 * 3| 16 17 18 19 20 21 22 23
 * 2| 08 09 10 11 12 13 14 15
 * 1| 00 01 02 03 04 05 06 07
 *
 * Bitboards are efficient because we can apply bitwise operations on them,
 * which the cpu can perform quickly.
 */
typedef __chess_util_bitboard bitboard;

/**
 * A position on a bitboard -- a single square.
 *
 * Implemented as a bitboard index */
typedef __chess_util_board_pos board_pos;

/**
 * Check if the bit is set for the given square */
int bitboard_check_square(bitboard board, board_pos square);

/**
 * Return the bitboard with the bit for square set */
bitboard bitboard_set_square(bitboard board, board_pos square);

/**
 * Return the bitboard with the bit for square cleared */
bitboard bitboard_clear_square(bitboard board, board_pos square);

/**
 * Return the bitboard with the bit for square flipped */
bitboard bitboard_flip_square(bitboard board, board_pos square);

/**
 * Population count: Count the numbers of bits set (== 1) in the bitboard */
int bitboard_popcount(bitboard board);

/**
 * Get the index of the first bit set in the bitboard (starting from the least
 * significant bit) */
int bitboard_scan_lsb(bitboard board);

bitboard bitboard_shift_n(bitboard board);

bitboard bitboard_shift_s(bitboard board);

bitboard bitboard_shift_w(bitboard board);

bitboard bitboard_shift_e(bitboard board);

bitboard bitboard_shift_nw(bitboard board);

bitboard bitboard_shift_ne(bitboard board);

bitboard bitboard_shift_sw(bitboard board);

bitboard bitboard_shift_se(bitboard board);

/**
 * Print a bitboard on stdout using only ascii */
void bitboard_print(bitboard board);

/**
 * Print a bitboard on stdout using ansi escapes + unicode */
void bitboard_print_pretty(bitboard board);

#define BOARD_POS_INVALID 255

/**
 * Convert x and y coordinates to a board_pos
 * if x and y describe an invalid board position, BOARD_POS_INVALID is returned
 */
board_pos board_pos_from_xy(int x, int y);

/**
 * Convert a board_pos to x and y coordinates
 * Sets *x and *y to the resulting position */
void board_pos_to_xy(board_pos pos, int *x, int *y);

int board_pos_to_x(board_pos pos);

int board_pos_to_y(board_pos pos);

/**
 * Convert a board_pos to algebraic notation (eg a1, h6, etc)
 * The result will be stored in str, which must be at least 3 bytes */
void board_pos_to_str(board_pos pos, char *str);

/**
 * Convert a pos in algebraic notation to a board_pos */
board_pos board_pos_from_str(const char *str);

/**
 * A board represents the state of a game, including:
 * - piece placement
 * - castling rights
 * - en passant target square
 * - player to move
 * - turn counter
 *
 * Implementation:
 * A bitboard for each player indicating what squares they have pieces on
 * A bitboard for pawns/rooks/knights/bishops/queens/kings
 * A value with flags for next player to move and en passant target square */
#define WHITE 0
#define BLACK 1

#define KING 0
#define PAWN 1
#define KNIGHT 2
#define ROOK 3
#define BISHOP 4
#define QUEEN 5

#define BOARD_FLAGS_EP_SQUARE 63
#define BOARD_FLAGS_EP_PRESENT 64
#define BOARD_FLAGS_TURN 128

#define BOARD_FLAGS_W_CASTLE_KING 256
#define BOARD_FLAGS_W_CASTLE_QUEEN 512
#define BOARD_FLAGS_B_CASTLE_KING 1024
#define BOARD_FLAGS_B_CASTLE_QUEEN 2048

#define BOARD_FLAGS_TURN_NUM 0xffff0000
#define BOARD_FLAGS_TURN_NUM_SHIFT 16

#define BOARD_FLAGS_LOW 0x0000ffff

typedef struct __chess_util_board board;

/**
 * check that board is in a consistent state */
void board_invariants(const board *board);

/**
 * initialize a board from a board in FEN notation */
void board_from_fen_str(board *board, const char *fen_string);

/**
 * express a board in FEN notation (minus halfmove + turn counters)
 * res_str must have 90 bytes allocated */
void board_to_fen_str(const board *board, char *res_str);

/**
 * get the piece at the given square, or -1 if no piece is on the square */
int board_piece_on_square(const board *board, board_pos square);

/**
 * get the player controlling the piece at the given square, or -1 if no piece
 * is on the square */
int board_player_on_square(const board *board, board_pos square);

/**
 * get which player's turn it is for the board */
int board_player_to_move(const board *board);

/**
 * get the turn number for the board
 * each turn represents two halfmoves (ie both white and black moving)
 * The move counter starts at one, and is incremented after black makes a move
 */
int board_get_full_turn_number(const board *board);

/**
 * get the en passant target square for the board, or BOARD_POS_INVALID if there
 * is no target square
 *
 * the en passant target square is the square which a pawn skipped while moving
 * forwards 2 squares last turn the en passant target square is directly behind
 * the pawn this turn, that pawn can be capture by moving onto the en passant
 * target square
 *
 * if no pawn was moves forwards 2 squares last turn, then there is no en
 * passant target square
 */
board_pos board_get_en_passant_target(const board *board);

/**
 * check if player can castle (side should be either KING or QUEEN) */
int board_can_castle(const board *board, int player, int side);

/**
 * print a board on stdout using ony ascii characters */
void board_print(const board *board);

/**
 * print a board on stdout using ansi escapes + unicode */
void board_print_pretty(const board *board);

/**
 * Convert a piece character to a piece index
 * pawn: P + p, knight: N + n, bishop: B + b, rook: R + r, queen: Q + q, king: K
 * + k
 */
int board_piece_char_to_piece(char c);

/**
 * Convert a piece character to a player index
 * white: P, N, B, R, Q, K
 * black: p, n, b, r, q, k
 */
int board_piece_char_to_player(char c);

/**
 * convert a piece and player index into a character representing that piece
 * white: P, N, B, R, Q, K
 * black: p, n, b, r, q, k
 */
char board_piece_char_from_piece_player(int piece, int player);

/**
 * pre generate move generation lookup tables */
void move_gen_pregenerate();

/**
 * move represents a single piece move on a board
 *
 * Implementation:
 * move has to be reversible
 * bits 0-15  : board's previous flags
 * bits 16-21 : move source
 * bits 22-27 : move destination
 * bit  28    : set if the move is a promotion
 * bits 31-29 : promotion piece value
 * bit  32    : set if the move is a capture
 * bits 33-35 : type of captured piece
 * bits 36-41 : square capture piece was on (may be different from move
 * destination due to en passant)
 */
typedef __chess_util_move move;

#define MOVE_END 0xffffffffffffffffULL

/**
 * get the source square of a move (where a piece is being moved from ) */
board_pos move_source_square(move move);

/**
 * get the destination square of a move (where a piece is being moved to) */
board_pos move_destination_square(move move);

/**
 * check if a move is a promotion */
int move_is_promotion(move move);

/**
 * if a move is a promotion, return what piece it is being promoted to
 * return -1 otherwise */
int move_promotion_piece(move move);

/**
 * check if a move is a capture */
int move_is_capture(move move);

/**
 * if a move is a capture, return what piece type is being captured
 * return -1 otherwise */
int move_capture_piece(move move);

/**
 * return 1 if a move is a castle, 0 otherwise */
int move_is_castle(move move);

/**
 * if a move is a capture, return what square is being captured
 * return BOARD_POS_INVALID otherwise
 * in most cases, this is the same as the move's destination
 * for en passant, it is different */
board_pos move_capture_square(move move);

/**
 * convert a move to a string in pure algebraic notation, which is of the form:
 * <src><dst><promote?>
 * for example, moving a piece (of any type) from e2 to e4 is:
 * e2e4
 * moving a pawn from a7 to a8 and promoting to a queen is:
 * a7a8q
 * a7a8n (promoting to knight)
 * a7a8b (promoting to bishop)
 * a7a8r (promoting to rook)
 *
 * res_str must have 6 bytes allocated */
void move_to_str(move move, char *res_str);

/**
 * construct a move from a string and a board
 * board must be a legal board for the move to be applied on (but the move will
 * not actually be made on it) */
move move_from_str(const char *move_str, const board *board);

/**
 * check if a move string is wellformed (corresponds to a chess move)
 * this does not check if the move is legal
 */
int move_str_is_wellformed(const char *move_str);

/**
 * construct a move from a source square, destination square, and promotion info
 * board is a legal board for the move to be later appiled on (but the move will not
 * actually be made on it)
 */
move move_new(board_pos src, board_pos dst, int is_promote, int promote_piece, const board* board);

/**
 * move_gen contains the state of the move generation algorithm */
typedef struct __chess_util_move_gen move_gen;

/**
 * initialize a move_gen structure for a given board */
void move_gen_init(move_gen *move_gen, board *board);

/**
 * check if a square on the board is attacked by a certain player
 * return a bitboard with each square set that is threatening the square
 * if the square is not attacked, the bitboard is 0 */
bitboard board_is_square_attacked(const board *board, board_pos square,
                                  int attacking_player);

/**
 * check if player is in check (ie -- player's king is attacked by opponent) */
bitboard board_player_in_check(const board *board, int player);

/**
 * get the next move from the move generator
 * if no more moves are available, the method returns MOVE_END */
move move_gen_next_move(move_gen *generator);

/**
 * get the next move from the move generator and apply it to board
 * if no more moves are available, the method returns MOVE_END
 * you MUST undo the move before calling move_gen_make_next_move or
 * move_gen_next_move again */
move move_gen_make_next_move(move_gen *generator);

/**
 * make the given move on the given board
 * mutates board */
void board_make_move(board *board, move move);

/**
 * undo the given move on the given board
 * the move must have been previously made on the board
 * mutates board */
void board_unmake_move(board *board, move move);

/**
 * check if the player to move on the board the move_gen is associated with is
 * in checkmate/stalemate this must be called after the move_gen has been
 * exhausted (all moves generated, MOVE_END returned)
 */
int move_gen_is_checkmate(move_gen *const move_gen);
int move_gen_is_stalemate(move_gen *const move_gen);

/**
 * check if a board is checkmated or stalemated for the player to move
 * move_gen_is_checkmate and move_gen_is_stalemate are faster if you have
 * already run a move_gen
 */
int board_is_checkmate(board *const board);
int board_is_stalemate(board *const board);

/**
 * check if two moves are the same move
 */
int moves_equal(move move0, move move1);

/**
 * check if a move is legal on the board
 * note: this is slow -- it runs through move generation for the board
 * moves generated by move_gen_next_move and move_gen_make_next_move are always
 * legal
 */
int move_is_legal(move move_to_check, board *board);

#endif
