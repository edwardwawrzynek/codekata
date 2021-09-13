use crate::games::{GameInstance, GameScore, GameState, GameTurn, GameType};
use crate::models::UserId;
use chess;
use std::collections::HashMap;
use std::fmt;

// chess board starting position
static DEFAULT_BOARD: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug)]
pub struct NineHolesGame();

pub struct NineHolesGameInstance {

};

impl GameInstance for NineHolesGameInstance {

}