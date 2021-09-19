use crate::models::UserId;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;

pub mod chess_game;
pub mod ended_game;
pub mod three_mens_morris;

/// A type of game that can be played by the server.
/// `GameType` represents the type of game, not a specific instance of that game.
pub trait GameType: Send + Sync {
    /// Create an instance of this game from it's serialized representation.
    fn deserialize(&self, data: &str, players: &[UserId]) -> Option<Box<dyn GameInstance>>;

    /// Create a new instance of this game with the given number of players. If a game cannot be created with this number of players, return None.
    fn new(&self, players: &[UserId]) -> Option<Box<dyn GameInstance>>;
}

/// Whose turn it is in a game
#[derive(Debug, PartialEq, Eq)]
pub enum GameTurn {
    /// A player's turn
    Turn(UserId),
    /// The game is done
    Finished,
}

/// State information about a game
#[derive(Debug, PartialEq, Eq)]
pub enum GameState {
    InProgress,
    Win(UserId),
    Tie,
}

pub type GameScore = HashMap<UserId, f64>;

/// An instance of a particular game, storing all of its state.
pub trait GameInstance {
    /// Serialize this game's entire state. This is the serialization used for storing and loading the game from database, and sending the game to observing clients. This serialization should include move history, scoring, etc. You do not need to serialize information about players' ids.
    fn serialize(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
    /// Serialize the portion of this game's state needed for a client to decide what move to make. This is probably just the current state of the game, and doesn't need to include information not needed to make move decisions (such as history/scoring/etc).
    fn serialize_current(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.serialize(f)
    }
    /// Check whose' turn it is.
    fn turn(&self) -> GameTurn;
    /// Make a move, or return an error describing why that move is illegal.
    fn make_move(&mut self, player: UserId, move_to_make: &str) -> Result<(), String>;
    /// Get the end state of the game. If the game doesn't have a specific win/loss/tie result, return None.
    fn end_state(&self) -> Option<GameState>;
    /// Get the scores for the game. If the game doesn't have score results, return None. May return None while the game is in progress and Some when scores are available.
    fn scores(&self) -> Option<GameScore>;
}

/// mapping from game type string to GameType
pub type GameTypeMap = HashMap<&'static str, Box<dyn GameType>>;

// Utility to be able to use serialize methods
pub struct Fmt<F>(pub F)
where
    F: Fn(&mut fmt::Formatter) -> fmt::Result;

impl<F> fmt::Display for Fmt<F>
where
    F: Fn(&mut fmt::Formatter) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (self.0)(f)
    }
}
