use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::collections::HashMap;
use std::ffi;
use std::fmt;
use std::fmt::{Display, Debug};
use std::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};
use tungstenite::{connect, Message};
use url::Url;

mod clib;

/// A position on a chessboard
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct BoardPos(pub clib::board_pos);

impl BoardPos {
    /// Construct a board position from an x and y
    pub fn new(x: i32, y: i32) -> BoardPos {
        BoardPos(unsafe { clib::board_pos_from_xy(x, y) })
    }

    /// Get the x value of the position
    pub fn x(self) -> i32 {
        unsafe { clib::board_pos_to_x(self.0) }
    }

    /// Get the y value of the position
    pub fn y(self) -> i32 {
        unsafe { clib::board_pos_to_y(self.0) }
    }

    /// An invalid board position (outside the chessboard)
    pub const INVALID: BoardPos = BoardPos(clib::BOARD_POS_INVALID as u8);

    /// Construct a board pos from a u8, and convert INVALID to None
    fn from_u8(pos: u8) -> Option<BoardPos> {
        if pos == clib::BOARD_POS_INVALID as u8 {
            None
        } else {
            Some(BoardPos(pos))
        }
    }
}

impl Display for BoardPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res: [i8; 3] = [0; 3];
        unsafe { clib::board_pos_to_str(self.0, res.as_mut_ptr()) };
        let c_str = unsafe { ffi::CStr::from_ptr(res.as_ptr()) };
        write!(f, "{}", c_str.to_str().unwrap())
    }
}

/// A structure with a bit corresponding to each square
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Bitboard(pub clib::bitboard);

impl Bitboard {
    /// Construct a bitboard from a bitboard value
    pub fn new(value: u64) -> Bitboard {
        Bitboard(value)
    }

    /// Check if the given square is set
    pub fn check_square(self, square: BoardPos) -> bool {
        (unsafe { clib::bitboard_check_square(self.0, square.0) }) != 0
    }

    /// Return self with the given square set to 1
    pub fn set_square(self, square: BoardPos) -> Bitboard {
        Bitboard(unsafe { clib::bitboard_set_square(self.0, square.0) })
    }

    /// Return self with the given square set to 0
    pub fn clear_square(self, square: BoardPos) -> Bitboard {
        Bitboard(unsafe { clib::bitboard_clear_square(self.0, square.0) })
    }

    /// Return self with the given square flipped
    pub fn flip_square(self, square: BoardPos) -> Bitboard {
        Bitboard(unsafe { clib::bitboard_flip_square(self.0, square.0) })
    }

    /// Return the number of bits set to 1 in the bitboard
    pub fn count(self) -> usize {
        (unsafe { clib::bitboard_popcount(self.0) }) as usize
    }

    /// Return the index of the first set bit, starting at lsb
    pub fn scan_lsb(self) -> BoardPos {
        BoardPos(unsafe { clib::bitboard_scan_lsb(self.0) } as u8)
    }

    /// Return true if any bits in the bitboard are set
    pub fn any_set(self) -> bool {
        self.0 != 0
    }

    /// Print the bitboard on stdout
    pub fn print(self) {
        unsafe { clib::bitboard_print(self.0) }
    }

    /// Pretty print the bitboard on stdout
    pub fn print_pretty(self) {
        unsafe { clib::bitboard_print_pretty(self.0) }
    }
}

/// An iterator over the set bits in a bitboard
pub struct BitboardSetIterator {
    val: Bitboard,
}

impl Iterator for BitboardSetIterator {
    type Item = BoardPos;

    fn next(&mut self) -> Option<BoardPos> {
        if !self.val.any_set() {
            None
        } else {
            let pos = self.val.scan_lsb();
            self.val = self.val.clear_square(pos);
            Some(pos)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.val.count();
        (size, Some(size))
    }
}

impl IntoIterator for Bitboard {
    type Item = BoardPos;
    type IntoIter = BitboardSetIterator;

    fn into_iter(self) -> BitboardSetIterator {
        BitboardSetIterator { val: self }
    }
}

impl BitAnd for Bitboard {
    type Output = Bitboard;

    fn bitand(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 & rhs.0)
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;

    fn bitor(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitXor for Bitboard {
    type Output = Bitboard;

    fn bitxor(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl Not for Bitboard {
    type Output = Bitboard;

    fn not(self) -> Bitboard {
        Bitboard(!self.0)
    }
}

impl Shl<usize> for Bitboard {
    type Output = Bitboard;

    fn shl(self, rhs: usize) -> Bitboard {
        return Bitboard(self.0 << rhs);
    }
}

impl Shr<usize> for Bitboard {
    type Output = Bitboard;

    fn shr(self, rhs: usize) -> Bitboard {
        return Bitboard(self.0 >> rhs);
    }
}

/// Player color (white or black)
#[derive(FromPrimitive, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Player {
    White = 0,
    Black = 1,
}

impl Player {
    /// Convert an int (0 or 1) to a player
    pub fn from_integral(int: u8) -> Player {
        if int == 0 {
            Player::White
        } else if int == 1 {
            Player::Black
        } else {
            panic!("int should be 0 or 1")
        }
    }
}

/// Piece type -- pawn, rook, knight, etc (not color)
#[derive(FromPrimitive, PartialEq, Eq, Clone, Copy, Debug)]
pub enum PieceType {
    King = 0,
    Pawn = 1,
    Knight = 2,
    Rook = 3,
    Bishop = 4,
    Queen = 5,
}

impl PieceType {
    /// Convert an int (0 - 5) to a piece type
    pub fn from_integral(int: u8) -> PieceType {
        if int == 0 {
            PieceType::King
        } else if int == 1 {
            PieceType::Pawn
        } else if int == 2 {
            PieceType::Knight
        } else if int == 3 {
            PieceType::Rook
        } else if int == 4 {
            PieceType::Bishop
        } else if int == 5 {
            PieceType::Queen
        } else {
            panic!("int must be 0 - 5")
        }
    }
}

/// A move that can be made on a [`Board`]
#[derive(Clone, Copy, Debug)]
pub struct Move(pub clib::move_);

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        (unsafe { clib::moves_equal(self.0, other.0) }) != 0
    }
}

impl Eq for Move {}

impl Move {
    /// Get the source square of the move
    pub fn src(self) -> BoardPos {
        BoardPos(unsafe { clib::move_source_square(self.0) })
    }

    /// Get the destination square of the move
    pub fn dst(self) -> BoardPos {
        BoardPos(unsafe { clib::move_destination_square(self.0) })
    }

    /// If the move is a pawn promotion, return what type the pawn if being promoted to
    pub fn promote(self) -> Option<PieceType> {
        FromPrimitive::from_i32(unsafe { clib::move_promotion_piece(self.0) })
    }

    /// If the move is a capture, return what piece type is being captured
    pub fn capture_piece(self) -> Option<PieceType> {
        FromPrimitive::from_i32(unsafe { clib::move_capture_piece(self.0) })
    }

    /// If the move is a capture, return what square is captured
    pub fn capture_square(self) -> Option<BoardPos> {
        if (unsafe { clib::move_is_capture(self.0) }) != 0 {
            Some(BoardPos(unsafe { clib::move_capture_square(self.0) }))
        } else {
            None
        }
    }

    /// Check if the move is a castle
    pub fn castle(self) -> bool {
        (unsafe { clib::move_is_castle(self.0) }) != 0
    }

    /// Get the board flags stored in the move
    pub fn board_flags(self) -> u16 {
        (self.0 & 0xffff) as u16
    }

    /// Construct a move from a string in long algebraic notation, or return None if the move string is malformed
    pub fn from_str(move_str: &str, board: &Board) -> Option<Move> {
        unsafe {
            if clib::move_str_is_wellformed(ffi::CString::new(move_str)
                .expect("Cstring::new failed")
                .as_ptr()) != 0 {
                Some(Move(clib::move_from_str(ffi::CString::new(move_str)
                                                  .expect("Cstring::new failed")
                                                  .as_ptr(), &board.0)))
            } else {
                None
            }
        }
    }

    /// Check if this move is legal to be applied on a board
    pub fn is_legal(self, board: &mut Board) -> bool {
        unsafe { clib::move_is_legal(self.0, &mut board.0) != 0 }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res: [i8; 5] = [0; 5];
        unsafe { clib::move_to_str(self.0, res.as_mut_ptr()) };
        let c_str = unsafe { ffi::CStr::from_ptr(res.as_ptr()) };
        write!(f, "{}", c_str.to_str().unwrap())
    }
}

/// The state of a chess game. Board contains the layout of pieces and whose turn it is to move.
pub struct Board(clib::board);

impl PartialEq for Board {
    fn eq(&self, other: &Board) -> bool {
        // TODO: this can be checked more efficiently
        self.to_string() == other.to_string()
    }
}

impl Eq for Board {}

impl Board {
    /// Create a board from FEN
    pub fn new(fen: &str) -> Board {
        let mut board: Board = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        unsafe {
            clib::board_from_fen_str(
                &mut board.0,
                ffi::CString::new(fen)
                    .expect("Cstring::new failed")
                    .as_ptr(),
            )
        };
        board
    }

    /// Get the piece type on the given square
    pub fn piece_on_square(&self, square: BoardPos) -> Option<PieceType> {
        FromPrimitive::from_i32(unsafe { clib::board_piece_on_square(&self.0, square.0) })
    }

    /// Get the player on the given square
    pub fn player_on_square(&self, square: BoardPos) -> Option<Player> {
        FromPrimitive::from_i32(unsafe { clib::board_player_on_square(&self.0, square.0) })
    }

    /// Check which player's turn it is to move
    pub fn player_to_move(&self) -> Player {
        FromPrimitive::from_i32(unsafe { clib::board_player_to_move(&self.0) }).unwrap()
    }

    /// Get a bitboard with bits set for squares controlled by player
    pub fn player_bb(&self, player: Player) -> Bitboard {
        Bitboard(self.0.players[player as usize])
    }

    /// Get a bitboard with bits set for squares controlled by player and of type piece
    pub fn piece_bb(&self, player: Player, piece: PieceType) -> Bitboard {
        Bitboard(self.0.players[player as usize] & self.0.pieces[piece as usize])
    }

    /// Get the en passant target square
    pub fn en_passant_target(&self) -> Option<BoardPos> {
        BoardPos::from_u8(unsafe { clib::board_get_en_passant_target(&self.0) })
    }

    /// Check if the given player has castling rights on side
    pub fn has_castling_rights(&self, player: Player, side: PieceType) -> bool {
        (unsafe { clib::board_can_castle(&self.0, player as i32, side as i32) }) != 0
    }

    /// Print the board on stdout
    pub fn print(&self) {
        unsafe { clib::board_print(&self.0) }
    }

    /// Pretty print the board on stdout
    pub fn print_pretty(&self) {
        unsafe { clib::board_print_pretty(&self.0) }
    }

    /// Return a bitboard with bits set for all squares attacking square
    pub fn square_attackers(&self, square: BoardPos, attacking_player: Player) -> Bitboard {
        Bitboard(unsafe {
            clib::board_is_square_attacked(&self.0, square.0, attacking_player as i32)
        })
    }

    /// Check if the given player is in check
    pub fn in_check(&self, player: Player) -> bool {
        (unsafe { clib::board_player_in_check(&self.0, player as i32) }) != 0
    }

    /// Apply a [`Move`] to the board
    pub fn make_move(&mut self, m: Move) {
        unsafe { clib::board_make_move(&mut self.0, m.0) }
    }

    /// Unapply a [`Move`] to the board
    pub fn unmake_move(&mut self, m: Move) {
        unsafe { clib::board_unmake_move(&mut self.0, m.0) }
    }

    /// Get the flags stored in the board
    pub fn flags(&self) -> u16 {
        self.0.flags
    }

    /// Check if the game is a stalemate
    pub fn is_stalemate(&self) -> bool {
        unsafe { clib::board_is_stalemate(&self.0) != 0}
    }

    /// Check if the game is a checkmate. If so, the player to move is checkmated, and has lost
    pub fn is_checkmate(&self) -> bool {
        unsafe { clib::board_is_checkmate(&self.0) != 0 }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res: [i8; 90] = [0; 90];
        unsafe { clib::board_to_fen_str(&self.0, res.as_mut_ptr()) };
        let c_str = unsafe { ffi::CStr::from_ptr(res.as_ptr()) };
        write!(f, "{}", c_str.to_str().unwrap())
    }
}

impl Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self, f)
    }
}

/// An iterator over all the legal moves that can be made on a [`Board`].
pub struct MoveGenerator(clib::move_gen);

impl MoveGenerator {
    /// Create a move generator from a board
    pub fn new(board: &mut Board) -> MoveGenerator {
        let mut gen: MoveGenerator = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        unsafe { clib::move_gen_init(&mut gen.0, &mut board.0) };
        gen
    }

    /// Get the next move out of the generator
    pub fn next(&mut self, board: &mut Board) -> Option<Move> {
        assert_eq!((&mut board.0 as *mut clib::board), self.0.board);
        let m = unsafe { clib::move_gen_next_move(&mut self.0) };
        if m == clib::MOVE_END {
            None
        } else {
            Some(Move(m))
        }
    }

    /// Get the next move out of the generator and apply it to board
    pub fn make_next(&mut self, board: &mut Board) -> Option<Move> {
        assert_eq!((&mut board.0 as *mut clib::board), self.0.board);
        let m = unsafe { clib::move_gen_make_next_move(&mut self.0) };
        if m == clib::MOVE_END {
            None
        } else {
            Some(Move(m))
        }
    }

    /// Once the generator is exhausted, check if the game is a checkmate
    pub fn is_checkmate(&mut self) -> bool {
        (unsafe { clib::move_gen_is_checkmate(&mut self.0) }) != 0
    }

    /// Once the generator is exhausted, check if the game is a stalemate
    pub fn is_stalemate(&mut self) -> bool {
        (unsafe { clib::move_gen_is_stalemate(&mut self.0) }) != 0
    }
}

/// Initialize the chess library c components
pub fn init() {
    unsafe { clib::move_gen_pregenerate() };
}

/// Connect to a codekata server at host and port, send apikey and name, and call func whenever a move is requested
pub fn connect_to_server<F>(host: &str, port: &str, apikey: &str, name: &str, func: F)
where
    F: Fn(&mut Board) -> (Move, HashMap<String, String>),
{
    init();

    let url = Url::parse(&*format!("ws://{}:{}/", host, port)).unwrap();

    // connect to server
    let (mut socket, _) = connect(url).expect("error connecting to server");

    // send name and apikey
    socket
        .write_message(Message::Text(format!("apikey {}", apikey)))
        .expect("error sending apikey command");
    socket
        .write_message(Message::Text(format!("name {}", name)))
        .expect("error sending name command");

    // wait for position or error command
    loop {
        let msg = socket
            .read_message()
            .expect("error reading msg from server");
        if msg.is_text() {
            let text = msg.into_text().unwrap();

            if text.starts_with("position") {
                let pos_str = &text[9..];
                let mut board = Board::new(pos_str);

                // call func and send move and debug info
                let (move_to_make, debug) = func(&mut board);
                socket
                    .write_message(Message::Text(format!("move {}", move_to_make)))
                    .expect("error sending move command");

                let mut debug_str = String::from("info ");
                for (key, value) in &debug {
                    debug_str += &*format!("{} {}`", key, value);
                }
                socket
                    .write_message(Message::Text(debug_str))
                    .expect("error sending info command");
            } else if text.starts_with("error") {
                println!("error from server: {}", text);
            }
        }
    }
}
