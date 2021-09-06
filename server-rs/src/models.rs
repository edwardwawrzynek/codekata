use super::schema::{game_players, games, tournament_players, tournaments, users};

pub type UserId = i32;
pub type GameId = i32;
pub type GamePlayerId = i32;
pub type TournamentId = i32;
pub type TournamentPlayerId = i32;

#[derive(Queryable, AsChangeset)]
#[table_name = "users"]
pub struct User {
    pub id: UserId,
    pub email: Option<String>,
    pub name: String,
    pub is_admin: bool,
    pub password_hash: Option<String>,
    pub api_key_hash: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub email: Option<&'a str>,
    pub name: &'a str,
    pub is_admin: bool,
    pub password_hash: Option<&'a str>,
    pub api_key_hash: &'a str,
}

#[derive(Queryable, AsChangeset)]
#[table_name = "games"]
pub struct DBGame {
    pub id: GameId,
    pub owner_id: UserId,
    pub game_type: String,
    pub state: Option<String>,
    pub finished: bool,
    pub winner: Option<UserId>,
    pub is_tie: Option<bool>,
    pub dur_per_move_ms: i64,
    pub dur_sudden_death_ms: i64,
    pub current_move_start_ms: Option<i64>,
    pub turn_id: Option<i64>,
    pub tournament_id: Option<TournamentId>,
}

#[derive(Insertable)]
#[table_name = "games"]
pub struct NewDBGame<'a> {
    pub owner_id: UserId,
    pub game_type: &'a str,
    pub state: Option<&'a str>,
    pub finished: bool,
    pub winner: Option<UserId>,
    pub is_tie: Option<bool>,
    pub dur_per_move_ms: i64,
    pub dur_sudden_death_ms: i64,
    pub current_move_start_ms: Option<i64>,
    pub turn_id: Option<i64>,
    pub tournament_id: Option<TournamentId>,
}

#[derive(Queryable, AsChangeset)]
#[table_name = "game_players"]
pub struct GamePlayer {
    pub id: GamePlayerId,
    pub user_id: UserId,
    pub game_id: GameId,
    pub score: Option<f64>,
    pub waiting_for_move: bool,
    pub time_ms: i64,
}

#[derive(Insertable)]
#[table_name = "game_players"]
pub struct NewGamePlayer {
    pub user_id: UserId,
    pub game_id: GameId,
    pub score: Option<f64>,
    pub waiting_for_move: bool,
    pub time_ms: i64,
}

#[derive(Queryable, AsChangeset)]
#[table_name = "tournaments"]
pub struct DBTournament {
    pub id: TournamentId,
    pub owner_id: UserId,
    pub tournament_type: String,
    pub game_type: String,
    pub dur_per_move_ms: i64,
    pub dur_sudden_death_ms: i64,
    pub started: bool,
    pub finished: bool,
    pub winner: Option<UserId>,
    pub options: String,
}

#[derive(Insertable)]
#[table_name = "tournaments"]
pub struct NewDBTournament<'a> {
    pub owner_id: UserId,
    pub tournament_type: &'a str,
    pub game_type: &'a str,
    pub dur_per_move_ms: i64,
    pub dur_sudden_death_ms: i64,
    pub started: bool,
    pub finished: bool,
    pub winner: Option<UserId>,
    pub options: &'a str,
}

#[derive(Queryable, AsChangeset, PartialEq, Eq, Hash, Debug, Copy, Clone)]
#[table_name = "tournament_players"]
pub struct TournamentPlayer {
    pub id: TournamentPlayerId,
    pub user_id: UserId,
    pub tournament_id: TournamentId,
    pub win: i32,
    pub loss: i32,
    pub tie: i32,
}

#[derive(Insertable)]
#[table_name = "tournament_players"]
pub struct NewTournamentPlayer {
    pub user_id: UserId,
    pub tournament_id: TournamentId,
    pub win: i32,
    pub loss: i32,
    pub tie: i32,
}
