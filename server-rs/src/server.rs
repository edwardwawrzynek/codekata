use crate::apikey::ApiKey;
use crate::cmd::{ClientCommand, ProtocolVersion, ServerCommand};
use crate::db::{init_db_pool, DBWrapper, Game, GameTimeCfg, PgPool, PlayerTimeExpiry, Tournament};
use crate::error::Error;
use crate::games::{Fmt, GameState, GameTurn, GameTypeMap};
use crate::models::{GameId, GamePlayer, TournamentId, TournamentPlayer, User, UserId};
use crate::tournament::{TournamentCfg, TournamentTypeMap};
use futures_channel::mpsc;
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use std::future::Future;
use std::sync::MutexGuard;
use std::time::{Duration, UNIX_EPOCH};
use std::{
    collections::HashMap,
    collections::HashSet,
    hash::Hash,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::net::{TcpListener, TcpStream};
use tungstenite::protocol::Message;

/// Topics that a client is interested in receiving messages about
#[derive(PartialEq, Eq, Hash, Debug)]
enum Topic {
    /// Messages for all clients
    Global,
    /// Messages for all clients logged in as a particular user
    UserPrivate(UserId),
    /// Messages for all clients logged in as a particular user using a certain protocol version
    UserPrivateProtocolVersion(UserId, ProtocolVersion),
    /// Messages about a particular game
    Game(GameId),
    /// Message about a particular tournament
    Tournament(TournamentId),
}

impl Default for Topic {
    fn default() -> Self {
        Topic::Global
    }
}

type ClientTxChannel = mpsc::UnboundedSender<Message>;

#[derive(Debug)]
struct ClientConnInfo {
    tx: ClientTxChannel,
    protocol: ProtocolVersion,
}

/// A collection of connected clients. PeerMap contains a mapping of topics to clients addresses, and client addresses to a communication channel.
#[derive(Debug, Default)]
struct ClientMap {
    // map client -> client transmit channel, protocol version
    channels: HashMap<SocketAddr, ClientConnInfo>,
    // map topic -> interested clients
    topics: HashMap<Topic, HashSet<SocketAddr>>,
    // map client -> logged in user
    users: HashMap<SocketAddr, UserId>,
}

type ClientMapLock = Arc<Mutex<ClientMap>>;

impl ClientMap {
    /// Insert a client connection
    pub fn insert_client(&mut self, client: SocketAddr, tx: ClientTxChannel) {
        self.channels.insert(
            client,
            ClientConnInfo {
                tx,
                protocol: ProtocolVersion::Legacy,
            },
        );
    }

    /// Add a client to a topic, creating that topic if it doesn't exist.
    fn add_to_priv_topic(&mut self, topic: Topic, client: SocketAddr) {
        let topic_map = self.topics.entry(topic).or_insert(HashSet::new());
        topic_map.insert(client);
    }

    /// Add a client to a topic, creating that topic if it doesn't exist.
    /// In order to register for UserPrivate topics, add_as_user must be used instead.
    pub fn add_to_topic(&mut self, topic: Topic, client: SocketAddr) {
        // clients must register on private topics through add_as_user (to lessen accidental registration for private messages)
        match topic {
            Topic::UserPrivate(_) | Topic::UserPrivateProtocolVersion(_, _) => return,
            _ => {}
        }
        self.add_to_priv_topic(topic, client);
    }

    /// Check if a client is registered as a logged in user
    pub fn is_user(&self, client: &SocketAddr) -> Option<UserId> {
        self.users.get(client).map(|u| *u)
    }

    /// Unregister a client as a user
    pub fn remove_as_user(&mut self, client: &SocketAddr) {
        if let Some(old_user) = self.is_user(&client) {
            self.remove_from_topic(Topic::UserPrivate(old_user), client);
            self.remove_from_topic(
                Topic::UserPrivateProtocolVersion(old_user, ProtocolVersion::Current),
                client,
            );
            self.remove_from_topic(
                Topic::UserPrivateProtocolVersion(old_user, ProtocolVersion::Legacy),
                client,
            );
        }
        self.users.remove(client);
    }

    /// Register a client as a user and add them to the UserPrivate topic for that user.
    /// If the client had been previously registered as a different user, unregister them.
    pub fn add_as_user(&mut self, user_id: UserId, client: SocketAddr) {
        self.remove_as_user(&client);
        self.users.insert(client, user_id);

        self.add_to_priv_topic(Topic::UserPrivate(user_id), client);
        self.add_to_priv_topic(
            Topic::UserPrivateProtocolVersion(user_id, self.protocol_ver(&client)),
            client,
        );
    }

    /// Remove a client from a topic (if the client is in that topic)
    pub fn remove_from_topic(&mut self, topic: Topic, client: &SocketAddr) {
        let topic_map = self.topics.get_mut(&topic);
        if let Some(topic_map) = topic_map {
            topic_map.remove(client);
        }
    }

    /// Remove a client connection
    pub fn remove_client(&mut self, client: &SocketAddr) {
        self.channels.remove(client);
        for (_, topic) in &mut self.topics {
            topic.remove(client);
        }
    }

    /// Send a message to a client
    pub fn send(&self, client: &SocketAddr, msg: Message) -> Result<(), Error> {
        let tx = self.channels.get(client);
        match tx {
            Some(ClientConnInfo { tx, .. }) => {
                tx.unbounded_send(msg).unwrap_or_else(|e| {
                    eprintln!(
                        "Can't send message to client -- receiving channel was closed, {}",
                        e
                    )
                });
                Ok(())
            }
            None => Err(Error::NoSuchConnectedClient),
        }
    }

    /// Send a message to all clients in a topic
    pub fn publish(&self, topic: Topic, msg: &Message) -> Result<(), Error> {
        let topic_map = self.topics.get(&topic);
        if let Some(topic_map) = topic_map {
            for client in topic_map {
                self.send(client, msg.clone())?;
            }
        }

        Ok(())
    }

    /// Get a connection's protocol version
    pub fn protocol_ver(&self, client: &SocketAddr) -> ProtocolVersion {
        self.channels[client].protocol
    }

    /// Set a connection's protocol version
    pub fn set_protocol_ver(&mut self, client: &SocketAddr, ver: ProtocolVersion) {
        let user = self.is_user(client);
        match self.channels.get_mut(client) {
            Some(conn) => {
                let old_ver = conn.protocol;
                conn.protocol = ver;

                if let Some(user_id) = user {
                    // remove from old protocol topic
                    self.remove_from_topic(
                        Topic::UserPrivateProtocolVersion(user_id, old_ver),
                        client,
                    );
                    // add to new protocol topic
                    self.add_to_priv_topic(
                        Topic::UserPrivateProtocolVersion(user_id, ver),
                        *client,
                    );
                }
            }
            None => {}
        }
    }
}

/// Convert a game and its players to a game command
fn serialize_game_state(game: &Game, players: &[GamePlayer]) -> ServerCommand {
    let (finished, winner, state, current_player) = match &game.instance {
        &None => (false, GameState::InProgress, None, None),
        Some(inst) => {
            let state = format!("{}", Fmt(|f| inst.serialize(f)));
            match inst.turn() {
                GameTurn::Finished => (
                    true,
                    inst.end_state().unwrap_or(GameState::InProgress),
                    Some(state),
                    None,
                ),
                GameTurn::Turn(uid) => (false, GameState::InProgress, Some(state), Some(uid)),
            }
        }
    };

    ServerCommand::Game {
        id: game.id,
        game_type: game.game_type.clone(),
        owner: game.owner_id,
        started: game.instance.is_some(),
        finished,
        winner,
        time_dur: game.time.to_ms(),
        current_move_start: game.current_move_start.map(|f| {
            f.duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_millis() as i64
        }),
        current_player,
        players: players
            .iter()
            .map(|p| (p.user_id, p.score, p.time_ms))
            .collect::<Vec<(UserId, Option<f64>, i64)>>(),
        state,
    }
}

/// Convert a tournament into a tournament command
fn serialize_tournament_state(
    tourney: &Tournament,
    players: Vec<TournamentPlayer>,
    db: &DBWrapper,
) -> Result<ServerCommand, Error> {
    let state =
        tourney
            .instance
            .end_state(tourney.started, tourney.id, &tourney.cfg, &*players, db)?;
    let games = format!(
        "{}",
        Fmt(|f| tourney
            .instance
            .serialize_games(tourney.id, &tourney.cfg, f, db))
    );

    Ok(ServerCommand::Tournament {
        id: tourney.id,
        owner: tourney.owner_id,
        tourney_type: tourney.tournament_type.clone(),
        game_type: tourney.cfg.game_type.clone(),
        started: tourney.started,
        finished: match state {
            GameState::InProgress => false,
            _ => true,
        },
        winner: state,
        players,
        games,
    })
}

/// Convert all games in a tournament to commands
fn serialize_tournament_games(
    id: TournamentId,
    db: &DBWrapper,
) -> Result<Vec<ServerCommand>, Error> {
    let mut res = vec![];
    let games = db.find_tournament_games(id)?;
    for dbgame in games.into_iter() {
        let (game, players) = db.dbgame_to_game_and_players(dbgame)?;
        res.push(serialize_game_state(&game, &*players));
    }
    Ok(res)
}

fn find_user_in_players(players: &[GamePlayer], user_id: UserId) -> Option<&GamePlayer> {
    let index = players.iter().position(|p| p.user_id == user_id);
    index.map(|i| &players[i])
}

/// Convert a game to a go or board command for its active player
fn serialize_game_for_player(
    game: &Game,
    players: &[GamePlayer],
    protocol: ProtocolVersion,
) -> Option<(UserId, ServerCommand)> {
    match &game.instance {
        &None => None,
        Some(inst) => match inst.turn() {
            GameTurn::Finished => None,
            GameTurn::Turn(user_id) => {
                let state = format!("{}", Fmt(|f| inst.serialize_current(f)));

                let player_sudden_death_start = find_user_in_players(players, user_id)
                    .expect("active user not in players")
                    .time_ms;
                let time_remaining = game
                    .current_player_time(Duration::from_millis(player_sudden_death_start as u64))
                    .to_ms();
                Some((
                    user_id,
                    match protocol {
                        ProtocolVersion::Current => ServerCommand::Go {
                            id: game.id,
                            game_type: game.game_type.clone(),
                            time_for_turn_ms: time_remaining.per_move_ms,
                            time_ms: time_remaining.sudden_death_ms,
                            state: Some(state),
                        },
                        ProtocolVersion::Legacy => ServerCommand::Position { state: Some(state) },
                    },
                ))
            }
        },
    }
}

/// Return a list of go commands for all the active games a player is in that are waiting on that player to move
fn serialize_waiting_games_for_user(
    user_id: UserId,
    db: &DBWrapper,
    protocol: ProtocolVersion,
) -> Result<Vec<ServerCommand>, Error> {
    let games = db.find_waiting_games_for_user(user_id)?;
    let mut res = Vec::new();
    // if in legacy mode, only send oldest game
    let games = match protocol {
        ProtocolVersion::Current => &*games,
        ProtocolVersion::Legacy => {
            if games.len() >= 1 {
                &games[..1]
            } else {
                &*games
            }
        }
    };
    for game_id in games {
        let (game, players) = db.find_game(*game_id)?;
        if let Some((uid, cmd)) = serialize_game_for_player(&game, &*players, protocol) {
            assert_eq!(uid, user_id);
            res.push(cmd);
        }
    }
    Ok(res)
}

/// Check if a user connected on a specific protocol version should receive go or board commands for a given game.
/// For `ProtocolVersion::Current`, this is always true. For `ProtocolVersion::Legacy`, this is only true if the game is the oldest game a player has to make a move in (since the legacy protocol only allows clients to consider one game at once).
fn user_should_receive_game_update(
    user_id: UserId,
    game_id: GameId,
    db: &DBWrapper,
    protocol: ProtocolVersion,
) -> Result<bool, Error> {
    match protocol {
        ProtocolVersion::Current => Ok(true),
        ProtocolVersion::Legacy => match db.find_oldest_waiting_game_for_user(user_id)? {
            Some(gid) => Ok(gid == game_id),
            None => Ok(false),
        },
    }
}

/// Handle a change in game state
fn handle_game_update(
    game: &Game,
    players: &[GamePlayer],
    db: &DBWrapper,
    clients: &Mutex<ClientMap>,
) {
    let state_cmd = serialize_game_state(game, players);
    let state_msg = Message::from(state_cmd.to_string());
    let clients = clients.lock().unwrap();
    // send game to all observers
    clients
        .publish(Topic::Game(game.id), &state_msg)
        .unwrap_or_else(|e| eprintln!("Can't send game state to game observers, {}", e));
    // send game to tournament observers
    if let Some(tourney_id) = game.tournament_id {
        clients
            .publish(Topic::Tournament(tourney_id), &state_msg)
            .unwrap_or_else(|e| eprintln!("Can't send game state to tournament observers, {}", e));
    }
    // send game to player whose turn it is
    for protocol in [ProtocolVersion::Current, ProtocolVersion::Legacy] {
        if let Some((user_id, cmd)) = serialize_game_for_player(game, &*players, protocol) {
            if user_should_receive_game_update(user_id, game.id, db, protocol).unwrap_or(false) {
                clients
                    .publish(
                        Topic::UserPrivateProtocolVersion(user_id, protocol),
                        &Message::from(cmd.to_string()),
                    )
                    .unwrap_or_else(|e| eprintln!("Can't send game to client, {}", e));
            }
        }
    }
}

/// Handle a change in tournament state
fn handle_tournament_update(
    tournament: &Tournament,
    players: &[TournamentPlayer],
    db: &DBWrapper,
    clients: &Mutex<ClientMap>,
) {
    // serialize tournament + send to observers
    let state_cmd = serialize_tournament_state(tournament, players.to_vec(), db)
        .unwrap_or_else(|e| ServerCommand::Error(e));

    let clients = clients.lock().unwrap();
    clients
        .publish(
            Topic::Tournament(tournament.id),
            &Message::from(state_cmd.to_string()),
        )
        .unwrap_or_else(|e| eprintln!("Can't send tournament to client, {}", e));
}

/// Handle the potential expiry of a player's time
fn handle_player_expiry(
    expiry: PlayerTimeExpiry,
    client_map: &Mutex<ClientMap>,
    db_pool: &PgPool,
    game_type_map: &GameTypeMap,
    tournament_type_map: &TournamentTypeMap,
    time_expiry_tx: mpsc::UnboundedSender<PlayerTimeExpiry>,
) -> Result<(), Error> {
    let game_update_callback = |game: &Game, players: &[GamePlayer], db: &DBWrapper| {
        handle_game_update(game, players, db, client_map);
    };
    let tournament_update_callback =
        |tourney: &Tournament, players: &[TournamentPlayer], db: &DBWrapper| {
            handle_tournament_update(tourney, players, db, client_map);
        };
    let db = DBWrapper::from_pg_pool(
        db_pool,
        game_type_map,
        tournament_type_map,
        game_update_callback,
        tournament_update_callback,
        time_expiry_tx,
    )?;
    // load game and check turn_id
    let (mut game, mut players) = db.find_game(expiry.game_id)?;
    if game.turn_id == Some(expiry.turn_id) {
        // TODO: handle winners for >2 player games
        if players.len() == 2 {
            // make player whose time did not expire winner
            let mut winner = None;
            for player in players.iter() {
                if player.user_id != expiry.user_id {
                    winner = Some(player.user_id);
                    break;
                }
            }

            db.end_game(&mut game, &mut *players, winner, "Time Expired".to_string())?;
        }
    }
    Ok(())
}

/// Apply a command sent by a client and return a response (if necessary)
fn handle_cmd(
    cmd: &ClientCommand,
    client_map: &Mutex<ClientMap>,
    client_addr: &SocketAddr,
    db_pool: &PgPool,
    game_type_map: &GameTypeMap,
    tournament_type_map: &TournamentTypeMap,
    player_expiry_tx: mpsc::UnboundedSender<PlayerTimeExpiry>,
) -> Result<Option<ServerCommand>, Error> {
    use ClientCommand::*;

    // lock the client map
    let clients = || client_map.lock().unwrap();

    // callback when a game's state changes
    let game_update = |game: &Game, players: &[GamePlayer], db: &DBWrapper| {
        handle_game_update(game, players, db, client_map);
    };
    let tournament_update = |tourney: &Tournament, players: &[TournamentPlayer], db: &DBWrapper| {
        handle_tournament_update(tourney, players, db, client_map);
    };

    // get a database connection
    let db = || {
        DBWrapper::from_pg_pool(
            db_pool,
            game_type_map,
            tournament_type_map,
            game_update,
            tournament_update,
            player_expiry_tx,
        )
    };
    // load the current user
    fn user(
        db: &DBWrapper,
        client_addr: &SocketAddr,
        clients: MutexGuard<ClientMap>,
    ) -> Result<User, Error> {
        if let Some(user_id) = clients.is_user(client_addr) {
            db.find_user(user_id)
        } else {
            Err(Error::NotLoggedIn)
        }
    }

    // send waiting games for user
    fn send_waiting_games(
        user_id: UserId,
        db: &DBWrapper,
        client_addr: &SocketAddr,
        clients: MutexGuard<ClientMap>,
    ) -> Result<(), Error> {
        let cmds =
            serialize_waiting_games_for_user(user_id, db, clients.protocol_ver(client_addr))?;
        for cmd in &cmds {
            clients.send(client_addr, Message::from(cmd.to_string()))?;
        }
        Ok(())
    }

    // login as a user
    fn login(
        user_id: UserId,
        client_addr: &SocketAddr,
        db: &DBWrapper,
        mut clients: MutexGuard<ClientMap>,
    ) -> Result<(), Error> {
        clients.add_as_user(user_id, *client_addr);
        send_waiting_games(user_id, db, client_addr, clients)?;
        Ok(())
    }

    // expect a specific protocol version
    let expect_proto = |expected: ProtocolVersion| {
        let proto = clients().protocol_ver(client_addr);
        if proto != expected {
            Err(Error::InvalidProtocolForCommand { proto, expected })
        } else {
            Ok(())
        }
    };

    match cmd {
        Version(ver) => {
            clients().set_protocol_ver(client_addr, *ver);
            Ok(None)
        }
        // --- User Authentication ---
        NewUser {
            name,
            email,
            password,
        } => {
            let db = db()?;
            let user = db.new_user(*name, *email, *password)?;
            login(user.id, client_addr, &db, clients())?;
            Ok(None)
        }
        NewTmpUser { name } => {
            let db = db()?;
            let user = db.new_tmp_user(*name)?;
            login(user.id, client_addr, &db, clients())?;
            Ok(None)
        }
        Apikey(key) => {
            let db = db()?;
            let user = db.find_user_by_apikey(key)?;
            login(user.id, client_addr, &db, clients())?;
            Ok(None)
        }
        Login { email, password } => {
            let db = db()?;
            let user = db.find_user_by_credentials(*email, *password)?;
            login(user.id, client_addr, &db, clients())?;
            Ok(None)
        }
        Logout => {
            clients().remove_as_user(client_addr);
            Ok(None)
        }
        // --- User Info / Edit ---
        Name(name) => {
            let db = db()?;
            db.save_user(&User {
                name: name.to_string(),
                ..user(&db, client_addr, clients())?
            })?;
            Ok(None)
        }
        Password(pass) => {
            let db = db()?;
            let hashed = bcrypt::hash(pass, bcrypt::DEFAULT_COST)?;
            db.save_user(&User {
                password_hash: Some(hashed),
                ..user(&db, client_addr, clients())?
            })?;
            Ok(None)
        }
        GenApikey => {
            let db = db()?;
            let key = ApiKey::new();
            db.save_user(&User {
                api_key_hash: key.hash().to_string(),
                ..user(&db, client_addr, clients())?
            })?;
            Ok(Some(ServerCommand::GenApikey(key)))
        }
        SelfUserInfo => {
            let user = user(&db()?, client_addr, clients())?;
            Ok(Some(ServerCommand::SelfUserInfo {
                id: user.id,
                name: user.name,
                email: user.email,
            }))
        }
        // --- Game Creation / Management --
        NewGame {
            game_type,
            total_time,
            time_per_move,
        } => {
            let db = &db()?;
            let user = user(db, client_addr, clients())?;
            let game = db.new_game(
                *game_type,
                user.id,
                GameTimeCfg::from_ms(*time_per_move, *total_time),
                None,
            )?;
            Ok(Some(ServerCommand::NewGame(game.id)))
        }
        NewGameTmpUsers {
            game_type,
            total_time,
            time_per_move,
            num_tmp_users,
        } => {
            if *num_tmp_users <= 0 {
                return Err(Error::InvalidNumberOfPlayers);
            }

            let db = &db()?;
            // create users
            let mut keys = vec![];
            let mut users = vec![];
            for i in 0..(*num_tmp_users) {
                // create user
                let name = format!("Temporary User #{}", i);
                let user = db.new_tmp_user(&*name)?;
                // create + set apikey
                let key = ApiKey::new();
                db.save_user(&User {
                    api_key_hash: key.hash().to_string(),
                    ..user
                })?;
                keys.push(key);
                users.push(user.id);
            }
            // create game
            let game = db.new_game(
                *game_type,
                users[0],
                GameTimeCfg::from_ms(*time_per_move, *total_time),
                None,
            )?;
            // join game
            for id in &users {
                db.join_game(game.id, *id)?;
            }
            // start game
            db.start_game(game.id, users[0])?;

            Ok(Some(ServerCommand::NewGameTmpUsers {
                id: game.id,
                users: keys,
            }))
        }
        ObserveGame(game_id) => {
            let (game, players) = db()?.find_game(*game_id)?;
            clients().add_to_topic(Topic::Game(*game_id), *client_addr);
            Ok(Some(serialize_game_state(&game, &players)))
        }
        StopObserveGame(game_id) => {
            clients().remove_from_topic(Topic::Game(*game_id), client_addr);
            Ok(None)
        }
        JoinGame(game_id) => {
            let db = &db()?;
            db.join_game(*game_id, user(db, client_addr, clients())?.id)?;
            Ok(None)
        }
        LeaveGame(game_id) => {
            let db = &db()?;
            db.leave_game(*game_id, user(db, client_addr, clients())?.id)?;
            Ok(None)
        }
        StartGame(game_id) => {
            let db = &db()?;
            db.start_game(*game_id, user(db, client_addr, clients())?.id)?;
            Ok(None)
        }
        Play { id, play } => {
            expect_proto(ProtocolVersion::Current)?;
            let db = &db()?;
            let user = user(db, client_addr, clients())?;
            db.make_move(*id, user.id, *play)?;
            Ok(None)
        }
        Move(play) => {
            expect_proto(ProtocolVersion::Legacy)?;
            let db = &db()?;
            let user = user(db, client_addr, clients())?;
            let game_id = db.find_oldest_waiting_game_for_user(user.id)?;
            match game_id {
                None => Err(Error::NotTurn),
                Some(game_id) => {
                    db.make_move(game_id, user.id, *play)?;
                    Ok(None)
                }
            }
        }
        NewTournament {
            tourney_type,
            game_type,
            total_time,
            time_per_move,
            options,
        } => {
            let db = &db()?;
            let user = user(db, client_addr, clients())?;
            let tourney = db.new_tournament(
                *tourney_type,
                user.id,
                &TournamentCfg {
                    game_type: game_type.to_string(),
                    time_cfg: GameTimeCfg::from_ms(*time_per_move, *total_time),
                },
                *options,
            )?;
            Ok(Some(ServerCommand::NewTournament(tourney.id)))
        }
        JoinTournament(id) => {
            let db = &db()?;
            let user = user(db, client_addr, clients())?;
            db.join_tournament(*id, user.id)?;
            Ok(None)
        }
        LeaveTournament(id) => {
            let db = &db()?;
            let user = user(db, client_addr, clients())?;
            db.leave_tournament(*id, user.id)?;
            Ok(None)
        }
        StartTournament(id) => {
            let db = &db()?;
            let user = user(db, client_addr, clients())?;
            db.start_tournament(*id, user.id)?;
            Ok(None)
        }
        ObserveTournament(id) => {
            // load tournament
            let db = &db()?;
            let mut clients = clients();
            let tourney = db.find_tournament(*id)?;
            let players = db.find_tournament_players(*id)?;
            // send games in tournament
            let games = serialize_tournament_games(*id, db)?;
            for cmd in games {
                clients.send(client_addr, Message::from(cmd.to_string()))?;
            }
            // add to topic
            clients.add_to_topic(Topic::Tournament(*id), *client_addr);
            // send tournament
            Ok(Some(serialize_tournament_state(&tourney, players, db)?))
        }
        StopObserveTournament(id) => {
            clients().remove_from_topic(Topic::Tournament(*id), client_addr);
            Ok(None)
        }
    }
}

/// Parse a message sent by a client, perform the necessary action, and send any needed response back
fn handle_message(
    msg: &Message,
    client_map: &Mutex<ClientMap>,
    client_addr: &SocketAddr,
    db_pool: &PgPool,
    game_type_map: &GameTypeMap,
    tournament_type_map: &TournamentTypeMap,
    player_expiry_tx: mpsc::UnboundedSender<PlayerTimeExpiry>,
) {
    // reply to ping messages
    let reply = if msg.is_close() || msg.is_ping() {
        Ok(None)
    } else {
        // parse the message
        match msg.to_text() {
            Err(_) => Err(Error::MessageParseError),
            Ok(txt) => {
                let cmd = ClientCommand::deserialize(txt);
                match cmd {
                    Ok(cmd) => handle_cmd(
                        &cmd,
                        client_map,
                        client_addr,
                        db_pool,
                        game_type_map,
                        tournament_type_map,
                        player_expiry_tx,
                    ),
                    Err(e) => Err(e),
                }
            }
        }
    };

    let clients = client_map.lock().unwrap();

    let reply = reply.unwrap_or_else(|e| Some(ServerCommand::Error(e)));

    let reply = match reply {
        Some(c) => Some(c),
        None => match clients.protocol_ver(client_addr) {
            ProtocolVersion::Current => Some(ServerCommand::Okay),
            _ => None,
        },
    };

    if let Some(reply) = reply {
        clients
            .send(client_addr, Message::from(reply.to_string()))
            .unwrap_or_else(|e| eprintln!("Error sending message to client, {}", e));
    }
}

async fn handle_connection(
    client_map: ClientMapLock,
    raw_stream: TcpStream,
    addr: SocketAddr,
    db_pool: Arc<PgPool>,
    game_type_map: Arc<GameTypeMap>,
    tournament_type_map: Arc<TournamentTypeMap>,
    player_expiry_tx: mpsc::UnboundedSender<PlayerTimeExpiry>,
) {
    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");

    // create channel for sending messages to websocket
    let (tx, rx) = mpsc::unbounded();
    client_map.lock().unwrap().insert_client(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    let handle_incoming = incoming.try_for_each(|msg| {
        handle_message(
            &msg,
            &*client_map,
            &addr,
            &db_pool,
            &game_type_map,
            &*tournament_type_map,
            player_expiry_tx.clone(),
        );

        future::ok(())
    });

    let send_outgoing = rx.map(Ok).forward(outgoing);

    pin_mut!(handle_incoming, send_outgoing);
    future::select(handle_incoming, send_outgoing).await;

    client_map.lock().unwrap().remove_client(&addr);
}

fn run_expiry_rx(
    clients: Arc<Mutex<ClientMap>>,
    db_pool: Arc<PgPool>,
    game_type_map: Arc<GameTypeMap>,
    tournament_type_map: Arc<TournamentTypeMap>,
    expiry_tx: mpsc::UnboundedSender<PlayerTimeExpiry>,
    mut expiry_rx: mpsc::UnboundedReceiver<PlayerTimeExpiry>,
) {
    tokio::spawn((|| async move {
        while let Some(expiry) = expiry_rx.next().await {
            handle_player_expiry(
                expiry,
                &*clients,
                &*db_pool,
                &*game_type_map,
                &*tournament_type_map,
                expiry_tx.clone(),
            )
            .unwrap_or_else(|e| eprintln!("failed to handle expiry: {}", e));
        }
    })());
}

pub fn run_server<'a>(
    url: &'a str,
    db_url: &'a str,
    game_type_map: Arc<GameTypeMap>,
    tournament_type_map: Arc<TournamentTypeMap>,
) -> impl Future<Output = ()> + 'a {
    async move {
        // Create application state
        let clients = Arc::new(Mutex::new(ClientMap::default()));
        let db_pool = Arc::new(init_db_pool(db_url).expect("Can't open database"));

        // Setup a tcp server and accept connections
        let try_socket = TcpListener::bind(url).await;
        let listener = try_socket.expect("Failed to bind to port");
        println!("Listening on: {}", url);

        // Setup channel to handle time events
        let (expiry_tx, expiry_rx) = mpsc::unbounded::<PlayerTimeExpiry>();
        run_expiry_rx(
            clients.clone(),
            db_pool.clone(),
            game_type_map.clone(),
            tournament_type_map.clone(),
            expiry_tx.clone(),
            expiry_rx,
        );

        while let Ok((stream, addr)) = listener.accept().await {
            tokio::spawn(handle_connection(
                clients.clone(),
                stream,
                addr,
                db_pool.clone(),
                game_type_map.clone(),
                tournament_type_map.clone(),
                expiry_tx.clone(),
            ));
        }
    }
}
