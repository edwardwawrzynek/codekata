use crate::games::{GameInstance, GameScore, GameState, GameTurn, GameType};
use crate::models::UserId;
use chess;
use std::collections::HashMap;
use std::fmt;

// chess board starting position
static DEFAULT_BOARD: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug)]
pub struct ChessGame();

impl GameType for ChessGame {
    fn deserialize(&self, data: &str, players: &[UserId]) -> Option<Box<dyn GameInstance>> {
        if players.len() != 2 {
            return None;
        }

        // serialization format: fen,[move0,move1,move2]
        let clean_data = data.replace('[', "").replace(']', "");
        let mut components = clean_data.split(',');
        if let Some(fen) = components.next() {
            let mut moves = Vec::new();
            for move_str in components {
                if move_str.len() > 0 {
                    moves.push(move_str.to_string())
                }
            }

            Some(Box::new(ChessGameInstance {
                board: chess::Board::new(fen),
                moves,
                white: players[0],
                black: players[1],
            }))
        } else {
            None
        }
    }

    fn new(&self, players: &[UserId]) -> Option<Box<dyn GameInstance>> {
        if players.len() != 2 {
            None
        } else {
            Some(Box::new(ChessGameInstance {
                board: chess::Board::new(DEFAULT_BOARD),
                moves: Vec::new(),
                white: players[0],
                black: players[1],
            }))
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ChessGameInstance {
    // current board state
    board: chess::Board,
    // moves made to reach this state
    moves: Vec<String>,
    // players in the game
    white: UserId,
    black: UserId,
}

impl ChessGameInstance {
    fn chess_player_to_user(&self, player: chess::Player) -> UserId {
        match player {
            chess::Player::White => self.white,
            chess::Player::Black => self.black,
        }
    }

    fn other_chess_player(&self, player: chess::Player) -> chess::Player {
        match player {
            chess::Player::White => chess::Player::Black,
            chess::Player::Black => chess::Player::White,
        }
    }

    fn other_user(&self, user: UserId) -> UserId {
        if user == self.white {
            self.black
        } else {
            self.white
        }
    }
}

impl GameInstance for ChessGameInstance {
    fn serialize(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},[", self.board.to_string())?;
        for (i, m) in (&self.moves).into_iter().enumerate() {
            write!(f, "{}", m)?;
            if i < self.moves.len() - 1 {
                write!(f, ",")?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }

    fn serialize_current(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // use FEN representation
        write!(f, "{}", self.board.to_string())
    }

    fn turn(&self) -> GameTurn {
        if self.board.is_stalemate() || self.board.is_checkmate() {
            GameTurn::Finished
        } else {
            GameTurn::Turn(self.chess_player_to_user(self.board.player_to_move()))
        }
    }

    fn make_move(&mut self, player: UserId, move_to_make: &str) -> Result<(), String> {
        if self.chess_player_to_user(self.board.player_to_move()) != player {
            return Err("not player's turn".to_string());
        }

        let chess_move = chess::Move::from_str(move_to_make, &self.board);
        match chess_move {
            Some(chess_move) => {
                if chess_move.is_legal(&mut self.board) {
                    self.board.make_move(chess_move);
                    self.moves.push(move_to_make.to_string());
                    Ok(())
                } else {
                    Err(format!("illegal move: {}", move_to_make))
                }
            }
            None => Err(format!("malformed move: {}", move_to_make)),
        }
    }

    fn end_state(&self) -> Option<GameState> {
        if self.board.is_stalemate() {
            Some(GameState::Tie)
        } else if self.board.is_checkmate() {
            let winner = self.other_chess_player(self.board.player_to_move());
            Some(GameState::Win(self.chess_player_to_user(winner)))
        } else {
            Some(GameState::InProgress)
        }
    }

    fn scores(&self) -> Option<GameScore> {
        let end_state = self.end_state();
        if let Some(end_state) = end_state {
            let mut scores = HashMap::new();
            match end_state {
                GameState::InProgress => None,
                GameState::Tie => {
                    scores.insert(self.white, 0.5);
                    scores.insert(self.black, 0.5);
                    Some(scores)
                }
                GameState::Win(winner) => {
                    scores.insert(winner, 1.0);
                    scores.insert(self.other_user(winner), 0.0);
                    Some(scores)
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::games::Fmt;

    #[test]
    fn chess_create_test() {
        let game = ChessGame();
        let players0 = vec![1];
        let players1 = vec![1, 2];
        if let Some(_) = game.new(&players0[..]) {
            panic!("number of players should be invalid");
        }
        if let None = game.new(&players1[..]) {
            panic!("number of players should be valid");
        }
    }

    #[test]
    fn chess_serialize_test() {
        let game = ChessGame();
        let instance = game.deserialize(
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2,[e2e4,c7c5]",
            &vec![1, 2],
        );
        if let Some(mut instance) = instance {
            assert_eq!(instance.end_state(), Some(GameState::InProgress));
            assert_eq!(instance.scores(), None);
            assert_eq!(instance.turn(), GameTurn::Turn(1));
            assert_eq!(
                format!("{}", Fmt(|f| instance.serialize(f))),
                "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2,[e2e4,c7c5]"
            );
            assert_eq!(
                format!("{}", Fmt(|f| instance.serialize_current(f))),
                "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2"
            );

            assert_eq!(
                instance.make_move(2, "e4e5"),
                Err("not player's turn".to_string())
            );
            assert_eq!(
                instance.make_move(1, "j4e5"),
                Err("malformed move: j4e5".to_string())
            );
            assert_eq!(
                instance.make_move(1, "e4e6"),
                Err("illegal move: e4e6".to_string())
            );
            assert_eq!(instance.make_move(1, "e4e5"), Ok(()));

            assert_eq!(
                format!("{}", Fmt(|f| instance.serialize(f))),
                "rnbqkbnr/pp1ppppp/8/2p1P3/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2,[e2e4,c7c5,e4e5]"
            );
            assert_eq!(
                format!("{}", Fmt(|f| instance.serialize_current(f))),
                "rnbqkbnr/pp1ppppp/8/2p1P3/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2"
            );
        } else {
            panic!("game should have parsed");
        }
    }
}
