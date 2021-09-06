use crate::games::{Fmt, GameInstance, GameScore, GameState, GameTurn, GameType};
use crate::models::UserId;
use std::fmt::Formatter;

/// A game that has ended abnormally (such as through time expiration, resignation, etc)
#[derive(Debug)]
pub struct EndedGame();

impl GameType for EndedGame {
    fn deserialize(&self, data: &str, _: &[UserId]) -> Option<Box<dyn GameInstance>> {
        let mut components = data.split(',');
        components.next()?;
        let winner = components.next()?;
        let reason = components.next()?;
        let game_type = components.next()?;
        let prev_state = components.next()?;

        let winner = match winner {
            "-" => None,
            id => id.parse::<UserId>().ok(),
        };

        Some(Box::new(EndedGameInstance {
            winner,
            reason: reason.to_string(),
            game_type: game_type.to_string(),
            prev_state: prev_state.to_string(),
        }))
    }

    fn new(&self, _: &[UserId]) -> Option<Box<dyn GameInstance>> {
        Some(Box::new(EndedGameInstance {
            winner: None,
            reason: "".to_string(),
            game_type: "".to_string(),
            prev_state: "".to_string(),
        }))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct EndedGameInstance {
    winner: Option<UserId>,
    reason: String,
    game_type: String,
    prev_state: String,
}

impl EndedGameInstance {
    pub fn from_current_state(
        state: Option<&dyn GameInstance>,
        game_type: String,
        winner: Option<UserId>,
        reason: String,
    ) -> EndedGameInstance {
        EndedGameInstance {
            winner,
            reason,
            game_type,
            prev_state: match state {
                Some(state) => format!("{}", Fmt(|f| state.serialize(f))),
                None => "-".to_string(),
            },
        }
    }
}

impl GameInstance for EndedGameInstance {
    fn serialize(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "__ENDED_GAME, {}, {}, {}, {}",
            self.winner.map_or("-".to_string(), |i| i.to_string()),
            self.reason,
            self.game_type,
            self.prev_state
        )
    }

    fn serialize_current(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.serialize(f)
    }

    fn turn(&self) -> GameTurn {
        GameTurn::Finished
    }

    fn make_move(&mut self, _: UserId, _: &str) -> Result<(), String> {
        Err("invalid move".to_string())
    }

    fn end_state(&self) -> Option<GameState> {
        match self.winner {
            None => Some(GameState::Tie),
            Some(uid) => Some(GameState::Win(uid)),
        }
    }

    fn scores(&self) -> Option<GameScore> {
        None
    }
}
