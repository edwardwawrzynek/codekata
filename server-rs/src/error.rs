use bcrypt;
use diesel;
use r2d2;

use crate::cmd::ProtocolVersion;
use futures_channel::mpsc;
use std::fmt;
use std::num::ParseIntError;
use tungstenite::protocol::Message;

#[derive(Debug)]
pub enum Error {
    DBError(diesel::result::Error),
    R2D2Error(r2d2::Error),
    BCryptError(bcrypt::BcryptError),
    NoSuchUser,
    MalformedApiKey,
    InvalidApiKey,
    IncorrectCredentials,
    EmailAlreadyTaken,
    InvalidCommand(String),
    InvalidNumberOfArguments {
        cmd: String,
        expected: usize,
        actual: usize,
    },
    NoSuchConnectedClient,
    ClientTxChannelClosed(mpsc::TrySendError<Message>),
    MessageParseError,
    NotLoggedIn,
    NoSuchGame,
    AlreadyInGame,
    GameAlreadyStarted,
    NotTurn,
    DontOwnGame,
    InvalidNumberOfPlayers,
    NotInGame,
    InvalidNumberId,
    NoSuchGameType(String),
    InvalidProtocolVersion,
    InvalidMove(String),
    InvalidProtocolForCommand {
        proto: ProtocolVersion,
        expected: ProtocolVersion,
    },
    NoSuchTournament,
    NoSuchTournamentType,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        use Error::*;
        match self {
            NoSuchUser => match other {
                NoSuchUser => true,
                _ => false,
            },
            MalformedApiKey => match other {
                MalformedApiKey => true,
                _ => false,
            },
            InvalidApiKey => match other {
                InvalidApiKey => true,
                _ => false,
            },
            IncorrectCredentials => match other {
                IncorrectCredentials => true,
                _ => false,
            },
            EmailAlreadyTaken => match other {
                EmailAlreadyTaken => true,
                _ => false,
            },
            InvalidCommand(self_cmd) => match other {
                InvalidCommand(other_cmd) => *self_cmd == *other_cmd,
                _ => false,
            },
            InvalidNumberOfArguments {
                cmd,
                expected,
                actual,
            } => match other {
                InvalidNumberOfArguments {
                    cmd: cmd_other,
                    expected: expected_other,
                    actual: actual_other,
                } => *cmd == *cmd_other && *expected == *expected_other && *actual == *actual_other,
                _ => false,
            },
            DBError(_) => match other {
                DBError(_) => true,
                _ => false,
            },
            BCryptError(_) => match other {
                BCryptError(_) => true,
                _ => false,
            },
            NoSuchConnectedClient => match other {
                NoSuchConnectedClient => true,
                _ => false,
            },
            ClientTxChannelClosed(_) => match other {
                ClientTxChannelClosed(_) => true,
                _ => false,
            },
            R2D2Error(_) => match other {
                R2D2Error(_) => true,
                _ => false,
            },
            MessageParseError => match other {
                MessageParseError => true,
                _ => false,
            },
            NotLoggedIn => match other {
                NotLoggedIn => true,
                _ => false,
            },
            NoSuchGame => match other {
                NoSuchGame => true,
                _ => false,
            },
            AlreadyInGame => match other {
                AlreadyInGame => true,
                _ => false,
            },
            DontOwnGame => match other {
                DontOwnGame => true,
                _ => false,
            },
            GameAlreadyStarted => match other {
                GameAlreadyStarted => true,
                _ => false,
            },
            NotTurn => match other {
                NotTurn => true,
                _ => false,
            },
            InvalidNumberOfPlayers => match other {
                InvalidNumberOfPlayers => true,
                _ => false,
            },
            NotInGame => match other {
                NotInGame => true,
                _ => false,
            },
            InvalidNumberId => match other {
                InvalidNumberId => true,
                _ => false,
            },
            NoSuchGameType(game_type) => match other {
                NoSuchGameType(other_type) => *game_type == *other_type,
                _ => false,
            },
            InvalidProtocolVersion => match other {
                InvalidProtocolVersion => true,
                _ => false,
            },
            InvalidMove(error) => match other {
                InvalidMove(other_error) => *error == *other_error,
                _ => false,
            },
            InvalidProtocolForCommand { proto, expected } => match other {
                InvalidProtocolForCommand {
                    proto: other_proto,
                    expected: other_expected,
                } => proto == other_proto && expected == other_expected,
                _ => false,
            },
            NoSuchTournament => match other {
                NoSuchTournament => true,
                _ => false,
            },
            NoSuchTournamentType => match other {
                NoSuchTournamentType => true,
                _ => false,
            },
        }
    }
}

impl Eq for Error {}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Error {
        Error::DBError(e)
    }
}

impl From<bcrypt::BcryptError> for Error {
    fn from(e: bcrypt::BcryptError) -> Error {
        Error::BCryptError(e)
    }
}

impl From<r2d2::Error> for Error {
    fn from(e: r2d2::Error) -> Error {
        Error::R2D2Error(e)
    }
}

impl From<ParseIntError> for Error {
    fn from(_e: ParseIntError) -> Error {
        Error::InvalidNumberId
    }
}

impl From<Error> for fmt::Error {
    fn from(_: Error) -> Self {
        fmt::Error
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            DBError(e) => write!(f, "database error: {}", *e),
            BCryptError(e) => write!(f, "bcrypt error: {}", *e),
            NoSuchUser => write!(f, "no such user"),
            MalformedApiKey => write!(f, "malformed api key"),
            InvalidApiKey => write!(f, "invalid api key"),
            IncorrectCredentials => write!(f, "incorrect login credentials"),
            EmailAlreadyTaken => write!(f, "email is already taken"),
            InvalidCommand(cmd) => write!(f, "unrecognized command: {}", cmd),
            InvalidNumberOfArguments {
                cmd,
                expected,
                actual,
            } => write!(
                f,
                "invalid number of arguments for command {} - expected {}, found {}",
                cmd, expected, actual
            ),
            NoSuchConnectedClient => write!(f, "no such connected client"),
            ClientTxChannelClosed(_) => write!(f, "client transmit channel is closed"),
            R2D2Error(_) => write!(
                f,
                "database pool error: could not establish database connection"
            ),
            MessageParseError => write!(
                f,
                "couldn't parse client command as text (make sure to use utf-8 encoded messages)"
            ),
            NotLoggedIn => write!(f, "you are not logged in"),
            NoSuchGame => write!(f, "no such game"),
            AlreadyInGame => write!(f, "you are already in that game"),
            GameAlreadyStarted => write!(f, "that game has already started"),
            DontOwnGame => write!(f, "you aren't the owner of that game"),
            InvalidNumberOfPlayers => write!(f, "invalid number of players joined to start game"),
            NotInGame => write!(f, "you aren't a player in that game"),
            InvalidNumberId => write!(f, "malformed id or number"),
            NoSuchGameType(game_type) => write!(f, "unsupported game type: {}", *game_type),
            InvalidProtocolVersion => write!(f, "invalid protocol version"),
            NotTurn => write!(f, "it is not your turn to move in that game"),
            InvalidMove(error) => write!(f, "invalid move: {}", *error),
            InvalidProtocolForCommand { proto, expected } => write!(
                f,
                "that command is only available in protocol version {} (you are in version {})",
                expected, proto
            ),
            NoSuchTournament => write!(f, "no such tournament"),
            NoSuchTournamentType => write!(f, "no such tournament type"),
        }
    }
}
