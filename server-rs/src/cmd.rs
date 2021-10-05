use crate::apikey::ApiKey;
use crate::db::GameTimeMs;
use crate::error::Error;
use crate::games::GameState;
use crate::models::{GameId, TournamentId, TournamentPlayer, UserId};
use lazy_static;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// Protocol Versions
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum ProtocolVersion {
    Legacy,
    Current,
}

impl Display for ProtocolVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolVersion::Legacy => write!(f, "1"),
            ProtocolVersion::Current => write!(f, "2"),
        }
    }
}

impl TryFrom<i32> for ProtocolVersion {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ProtocolVersion::Legacy),
            2 => Ok(ProtocolVersion::Current),
            _ => Err(Error::InvalidProtocolVersion),
        }
    }
}

/// A command that can be sent from server to client
#[derive(PartialEq, Debug)]
pub enum ServerCommand {
    /// Report an error to the client
    Error(Error),
    /// Report that a command succeeded
    Okay,
    /// Report the current user's newly generated api key
    GenApikey(ApiKey),
    /// Report information for the current user
    SelfUserInfo {
        id: UserId,
        name: String,
        email: Option<String>,
    },
    /// Return a new game's id
    NewGame(GameId),
    /// Report a new game with temp user
    NewGameTmpUsers { id: GameId, users: Vec<ApiKey> },
    /// Report a game's state to clients
    Game {
        id: GameId,
        game_type: String,
        owner: UserId,
        started: bool,
        finished: bool,
        winner: GameState,
        time_dur: GameTimeMs,
        current_move_start: Option<i64>,
        current_player: Option<UserId>,
        players: Vec<(UserId, String, Option<f64>, i64)>,
        state: Option<String>,
    },
    /// Report a new tournament's id
    NewTournament(TournamentId),
    /// Report a tournament's state to clients
    Tournament {
        id: TournamentId,
        tourney_type: String,
        owner: UserId,
        game_type: String,
        started: bool,
        finished: bool,
        winner: GameState,
        players: Vec<(UserId, String, i32, i32, i32)>,
        games: String,
    },
    /// Send a game to the client to make a move on
    Go {
        id: GameId,
        game_type: String,
        time_ms: i64,
        time_for_turn_ms: i64,
        state: Option<String>,
    },
    /// Send a game to the client to make a move on (legacy)
    Position { state: Option<String> },
}

/// A command sent to the server from the client
#[derive(PartialEq, Eq, Debug)]
pub enum ClientCommand<'a> {
    /// Set the protocol version
    Version(ProtocolVersion),
    /// Create a new user with login credentials and login
    NewUser {
        name: &'a str,
        email: &'a str,
        password: &'a str,
    },
    /// Create a new user without login credentials and login
    NewTmpUser {
        name: &'a str,
    },
    /// Login with an apikey
    Apikey(ApiKey),
    /// Login with an email and password
    Login {
        email: &'a str,
        password: &'a str,
    },
    /// Lgout of the current session
    Logout,
    /// Set the current user's name
    Name(&'a str),
    /// Set the current user's password
    Password(&'a str),
    /// Generate an apikey for the current user (ServerCommand::GenApiKey response)
    GenApikey,
    /// Get info on the current user (ServerCommand::UserInfo response)
    SelfUserInfo,
    /// Create a new game of the given type
    NewGame {
        game_type: &'a str,
        total_time: i64,
        time_per_move: i64,
    },
    /// Create a new game with temporary users
    NewGameTmpUsers {
        game_type: &'a str,
        total_time: i64,
        time_per_move: i64,
        num_tmp_users: i32,
    },
    /// Observe a game with the given id
    ObserveGame(GameId),
    /// End observation of a game with the given id
    StopObserveGame(GameId),
    /// Join a game with the given id
    JoinGame(GameId),
    /// Leave a game with the given id
    LeaveGame(GameId),
    /// Start a game with the given id
    StartGame(GameId),
    /// Create a new tournament
    NewTournament {
        tourney_type: &'a str,
        game_type: &'a str,
        total_time: i64,
        time_per_move: i64,
        options: &'a str,
    },
    /// Become a player in a tournament
    JoinTournament(TournamentId),
    /// Stop being a player in a tournament
    LeaveTournament(TournamentId),
    /// Start a tournament
    StartTournament(TournamentId),
    /// Get updates on a tournament
    ObserveTournament(TournamentId),
    // stop getting updates on a tournament
    StopObserveTournament(TournamentId),
    /// Make a move in a game
    Play {
        id: GameId,
        play: &'a str,
    },
    /// Make a move in a game (legacy)
    Move(&'a str),
}

impl ServerCommand {
    fn write_game_state(f: &mut fmt::Formatter<'_>, winner: &GameState) -> fmt::Result {
        match winner {
            &GameState::InProgress => write!(f, "-"),
            &GameState::Win(uid) => write!(f, "{}", uid),
            &GameState::Tie => write!(f, "tie"),
        }
    }
}

impl fmt::Display for ServerCommand {
    /// Serialize the command into the textual representation expected by the client
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ServerCommand::*;
        let dash_str = "-".to_string();

        match self {
            &Okay => write!(f, "okay"),
            &Error(ref e) => write!(f, "error {}", e.to_string()),
            &GenApikey(ref key) => write!(f, "gen_apikey {}", key.to_string()),
            &SelfUserInfo {
                id,
                ref name,
                ref email,
            } => {
                let email_str = email.as_ref().unwrap_or(&dash_str);
                write!(f, "self_user_info {}, {}, {}", id, *name, *email_str)
            }
            &NewGame(id) => write!(f, "new_game {}", id),
            &NewGameTmpUsers { id, ref users } => {
                write!(f, "new_game_tmp_users {}", id)?;
                for user in users {
                    write!(f, ", {}", *user)?;
                }
                Ok(())
            }
            &Game {
                id,
                ref game_type,
                started,
                finished,
                ref winner,
                ref players,
                owner,
                ref state,
                ref time_dur,
                current_player,
                current_move_start,
            } => {
                write!(
                    f,
                    "game {}, {}, {}, {}, {}, ",
                    id, *game_type, owner, started, finished
                )?;
                ServerCommand::write_game_state(f, winner)?;
                write!(
                    f,
                    ", {}, {}, ",
                    time_dur.sudden_death_ms, time_dur.per_move_ms
                )?;
                match current_move_start {
                    Some(t) => write!(f, "{}", t)?,
                    None => write!(f, "-")?,
                };
                write!(f, ", ")?;
                match current_player {
                    Some(i) => write!(f, "{}", i)?,
                    None => write!(f, "-")?,
                }
                write!(f, ", [")?;
                for (i, player) in players.iter().enumerate() {
                    write!(
                        f,
                        "[{}, {}, {}, {}]",
                        (*player).0,
                        (*player).1,
                        (*player).2.unwrap_or(0.0),
                        (*player).3
                    )?;
                    if i < players.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "], {}", *state.as_ref().unwrap_or(&dash_str))
            }
            &NewTournament(id) => write!(f, "new_tournament {}", id),
            &Tournament {
                id,
                ref tourney_type,
                owner,
                ref game_type,
                started,
                finished,
                ref winner,
                ref players,
                ref games,
            } => {
                write!(
                    f,
                    "tournament {}, {}, {}, {}, {}, {}, ",
                    id, tourney_type, owner, game_type, started, finished
                )?;
                ServerCommand::write_game_state(f, winner)?;
                write!(f, ", [")?;
                for (i, player) in players.iter().enumerate() {
                    write!(
                        f,
                        "[{}, {}, {}, {}, {}]",
                        player.0, player.1, player.2, player.3, player.4
                    )?;
                    if i < players.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "], {}", games)
            }
            &Go {
                id,
                ref game_type,
                time_ms,
                time_for_turn_ms,
                ref state,
            } => write!(
                f,
                "go {}, {}, {}, {}, {}",
                id,
                *game_type,
                time_ms,
                time_for_turn_ms,
                *state.as_ref().unwrap_or(&dash_str)
            ),
            &Position { ref state } => {
                write!(f, "position {}", *state.as_ref().unwrap_or(&dash_str))
            }
        }
    }
}

/// Parse a command from a client into a command and arguments
fn parse_cmd(msg: &str) -> (&str, Vec<&str>) {
    let mut cmd = msg;
    let mut args = Vec::new();

    let mut cmd_end_index = msg.len();
    for (i, c) in msg.chars().enumerate() {
        if char::is_whitespace(c) {
            cmd = &msg[0..i];
            cmd_end_index = i;
            break;
        }
    }

    if cmd_end_index < msg.len() {
        for el in msg[cmd_end_index..].split(',') {
            args.push(el.trim());
        }
    }

    (cmd, args)
}

lazy_static! {
    // number of arguments expected for each command
    static ref NUM_ARGS: HashMap<&'static str, usize> = {
        let mut m = HashMap::new();
        m.insert("new_user", 3);
        m.insert("new_tmp_user", 1);
        m.insert("apikey", 1);
        m.insert("login", 2);
        m.insert("name", 1);
        m.insert("password", 1);
        m.insert("gen_apikey", 0);
        m.insert("self_user_info", 0);
        m.insert("logout", 0);
        m.insert("new_game", 3);
        m.insert("new_game_tmp_users", 4);
        m.insert("observe_game", 1);
        m.insert("stop_observe_game", 1);
        m.insert("join_game", 1);
        m.insert("leave_game", 1);
        m.insert("start_game", 1);
        m.insert("new_tournament", 5);
        m.insert("join_tournament", 1);
        m.insert("leave_tournament", 1);
        m.insert("start_tournament", 1);
        m.insert("observe_tournament", 1);
        m.insert("stop_observe_tournament", 1);
        m.insert("version", 1);
        m.insert("play", 2);
        m.insert("move", 1);
        m
    };
}

fn parse_val<F: FromStr>(str: &str) -> Result<F, Error> {
    match str.parse::<F>() {
        Ok(id) => Ok(id),
        Err(_) => Err(Error::InvalidNumberId),
    }
}

fn parse_protocol(str: &str) -> Result<ProtocolVersion, Error> {
    let num = parse_val::<i32>(str)?;
    ProtocolVersion::try_from(num)
}

impl ClientCommand<'_> {
    /// Parse a command from the textual representation sent by a client
    pub fn deserialize(message: &str) -> Result<ClientCommand, Error> {
        use ClientCommand::*;

        let msg = message.trim();
        let (cmd, args) = parse_cmd(msg);
        // check for command existence + correct number of arguments
        let expected_args = NUM_ARGS.get(cmd);
        match expected_args {
            None => return Err(Error::InvalidCommand(cmd.to_string())),
            Some(expected) => {
                if args.len() != *expected {
                    return Err(Error::InvalidNumberOfArguments {
                        cmd: cmd.to_string(),
                        expected: *expected,
                        actual: args.len(),
                    });
                }
            }
        }
        // command is correct, so deserialize
        match cmd {
            "version" => Ok(Version(parse_protocol(args[0])?)),
            "new_user" => Ok(NewUser {
                name: args[0],
                email: args[1],
                password: args[2],
            }),
            "new_tmp_user" => Ok(NewTmpUser { name: args[0] }),
            "apikey" => Ok(Apikey(ApiKey::try_from(args[0])?)),
            "login" => Ok(Login {
                email: args[0],
                password: args[1],
            }),
            "name" => Ok(Name(args[0])),
            "password" => Ok(Password(args[0])),
            "gen_apikey" => Ok(GenApikey),
            "self_user_info" => Ok(SelfUserInfo),
            "logout" => Ok(Logout),
            "new_game" => Ok(NewGame {
                game_type: args[0],
                total_time: parse_val(args[1])?,
                time_per_move: parse_val(args[2])?,
            }),
            "new_game_tmp_users" => Ok(NewGameTmpUsers {
                game_type: args[0],
                total_time: parse_val(args[1])?,
                time_per_move: parse_val(args[2])?,
                num_tmp_users: parse_val(args[3])?,
            }),
            "observe_game" => Ok(ObserveGame(parse_val(args[0])?)),
            "stop_observe_game" => Ok(StopObserveGame(parse_val(args[0])?)),
            "join_game" => Ok(JoinGame(parse_val(args[0])?)),
            "leave_game" => Ok(LeaveGame(parse_val(args[0])?)),
            "start_game" => Ok(StartGame(parse_val(args[0])?)),
            "play" => Ok(Play {
                id: parse_val(args[0])?,
                play: args[1],
            }),
            "move" => Ok(Move(args[0])),
            "new_tournament" => Ok(NewTournament {
                tourney_type: args[0],
                game_type: args[1],
                total_time: parse_val(args[2])?,
                time_per_move: parse_val(args[3])?,
                options: args[4],
            }),
            "join_tournament" => Ok(JoinTournament(parse_val(args[0])?)),
            "leave_tournament" => Ok(LeaveTournament(parse_val(args[0])?)),
            "start_tournament" => Ok(StartTournament(parse_val(args[0])?)),
            "observe_tournament" => Ok(ObserveTournament(parse_val(args[0])?)),
            "stop_observe_tournament" => Ok(StopObserveTournament(parse_val(args[0])?)),
            _ => Err(Error::InvalidCommand(cmd.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cmd_serialize_test() {
        assert_eq!(ServerCommand::Okay.to_string(), "okay");
        assert_eq!(
            ServerCommand::SelfUserInfo {
                id: 5,
                name: "user".to_string(),
                email: Some("sample@example.com".to_string()),
            }
            .to_string(),
            "self_user_info 5, user, sample@example.com"
        );
        assert_eq!(
            ServerCommand::Error(Error::InvalidApiKey).to_string(),
            "error invalid api key"
        );
        assert_eq!(
            ServerCommand::GenApikey(
                ApiKey::try_from("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                    .expect("failed to parse api key")
            )
            .to_string(),
            "gen_apikey aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert_eq!(ServerCommand::NewGame(1).to_string(), "new_game 1");
        assert_eq!(
            ServerCommand::Game {
                id: 1,
                game_type: "some_game".to_string(),
                owner: 2,
                started: true,
                finished: true,
                winner: GameState::Tie,
                time_dur: GameTimeMs { sudden_death_ms: 200, per_move_ms: 100 },
                current_move_start: Some(150),
                current_player: Some(3),
                players: vec![(3, "Name1".to_string(), Some(0.5), 1), (4, "Name2".to_string(), Some(4.5), 2), (5, "Name3".to_string(), None, 3)],
                state: Some("STATE".to_string()),
            }
            .to_string(),
            "game 1, some_game, 2, true, true, tie, 200, 100, 150, 3, [[3, Name1, 0.5, 1], [4, Name2, 4.5, 2], [5, Name3, 0, 3]], STATE"
        );
        assert_eq!(
            ServerCommand::Go {
                id: 1,
                game_type: "some_game".to_string(),
                time_ms: 1234,
                time_for_turn_ms: 321,
                state: Some("STATE".to_string())
            }
            .to_string(),
            "go 1, some_game, 1234, 321, STATE"
        );
        assert_eq!(
            ServerCommand::Position {
                state: Some("STATE".to_string())
            }
            .to_string(),
            "position STATE"
        );
        assert_eq!(
            ServerCommand::NewTournament(1).to_string(),
            "new_tournament 1"
        );
        assert_eq!(
            ServerCommand::Tournament {
                id: 1,
                tourney_type: "type".to_string(),
                owner: 2,
                game_type: "game".to_string(),
                started: true,
                finished: true,
                winner: GameState::Tie,
                players: vec![
                    (3, "Name1".to_string(), 4, 5, 6),
                    (7, "Name2".to_string(), 8, 9, 10)
                ],
                games: "GAMES".to_string()
            }
            .to_string(),
            "tournament 1, type, 2, game, true, true, tie, [[3, Name1, 4, 5, 6], [7, Name2, 8, 9, 10]], GAMES"
        );
    }

    #[test]
    fn cmd_parse_test() {
        assert_eq!(
            ClientCommand::deserialize("version 2"),
            Ok(ClientCommand::Version(ProtocolVersion::Current))
        );
        assert_eq!(
            ClientCommand::deserialize("version 1"),
            Ok(ClientCommand::Version(ProtocolVersion::Legacy))
        );
        assert_eq!(
            ClientCommand::deserialize("version 0"),
            Err(Error::InvalidProtocolVersion)
        );
        assert_eq!(
            ClientCommand::deserialize("random_cmd"),
            Err(Error::InvalidCommand("random_cmd".to_string()))
        );

        assert_eq!(
            ClientCommand::deserialize("new_user test, hi"),
            Err(Error::InvalidNumberOfArguments {
                expected: 3,
                actual: 2,
                cmd: "new_user".to_string()
            })
        );
        assert_eq!(
            ClientCommand::deserialize("new_user User Name , user@sample.com,password  "),
            Ok(ClientCommand::NewUser {
                name: "User Name",
                email: "user@sample.com",
                password: "password"
            })
        );

        assert_eq!(
            ClientCommand::deserialize("new_tmp_user   Hi  "),
            Ok(ClientCommand::NewTmpUser { name: "Hi" })
        );

        assert_eq!(
            ClientCommand::deserialize("apikey hello"),
            Err(Error::MalformedApiKey)
        );
        assert_eq!(
            ClientCommand::deserialize("apikey 0123456789abcdef0123456789abcdef"),
            Ok(ClientCommand::Apikey(
                ApiKey::try_from("0123456789abcdef0123456789abcdef")
                    .expect("failed to parse api key")
            ))
        );

        assert_eq!(
            ClientCommand::deserialize("login sample@example.com,password"),
            Ok(ClientCommand::Login {
                email: "sample@example.com",
                password: "password"
            })
        );
        assert_eq!(
            ClientCommand::deserialize("logout"),
            Ok(ClientCommand::Logout)
        );

        assert_eq!(
            ClientCommand::deserialize("gen_apikey   "),
            Ok(ClientCommand::GenApikey)
        );
        assert_eq!(
            ClientCommand::deserialize("self_user_info"),
            Ok(ClientCommand::SelfUserInfo)
        );

        assert_eq!(
            ClientCommand::deserialize("new_game chess, 1000, 500"),
            Ok(ClientCommand::NewGame {
                game_type: "chess",
                total_time: 1000,
                time_per_move: 500
            })
        );
        assert_eq!(
            ClientCommand::deserialize("new_game_tmp_users chess, 1000, 500, 5"),
            Ok(ClientCommand::NewGameTmpUsers {
                game_type: "chess",
                total_time: 1000,
                time_per_move: 500,
                num_tmp_users: 5
            })
        );
        assert_eq!(
            ClientCommand::deserialize("observe_game game"),
            Err(Error::InvalidNumberId)
        );
        assert_eq!(
            ClientCommand::deserialize("observe_game 1"),
            Ok(ClientCommand::ObserveGame(1))
        );
        assert_eq!(
            ClientCommand::deserialize("stop_observe_game 2"),
            Ok(ClientCommand::StopObserveGame(2))
        );
        assert_eq!(
            ClientCommand::deserialize("start_game 3"),
            Ok(ClientCommand::StartGame(3))
        );
        assert_eq!(
            ClientCommand::deserialize("join_game 4"),
            Ok(ClientCommand::JoinGame(4))
        );
        assert_eq!(
            ClientCommand::deserialize("leave_game 5"),
            Ok(ClientCommand::LeaveGame(5))
        );
        assert_eq!(
            ClientCommand::deserialize("play 1, e2e4"),
            Ok(ClientCommand::Play {
                id: 1,
                play: "e2e4"
            })
        );
        assert_eq!(
            ClientCommand::deserialize("move e2e4"),
            Ok(ClientCommand::Move("e2e4"))
        );

        assert_eq!(
            ClientCommand::deserialize("new_tournament type, game, 100, 200, 2"),
            Ok(ClientCommand::NewTournament {
                tourney_type: "type",
                game_type: "game",
                total_time: 100,
                time_per_move: 200,
                options: "2"
            })
        );
        assert_eq!(
            ClientCommand::deserialize("join_tournament 1"),
            Ok(ClientCommand::JoinTournament(1))
        );
        assert_eq!(
            ClientCommand::deserialize("leave_tournament 1"),
            Ok(ClientCommand::LeaveTournament(1))
        );
        assert_eq!(
            ClientCommand::deserialize("start_tournament 1"),
            Ok(ClientCommand::StartTournament(1))
        );
        assert_eq!(
            ClientCommand::deserialize("observe_tournament 1"),
            Ok(ClientCommand::ObserveTournament(1))
        );
        assert_eq!(
            ClientCommand::deserialize("stop_observe_tournament 1"),
            Ok(ClientCommand::StopObserveTournament(1))
        );
    }
}
