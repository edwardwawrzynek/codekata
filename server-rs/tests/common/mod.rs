use dotenv::dotenv;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use diesel::{Connection, PgConnection, RunQueryDsl};
use diesel_migrations::embed_migrations;
use server_rs::games::GameTypeMap;
use server_rs::tournament::TournamentTypeMap;
use server_rs::*;
use std::net::TcpListener;
use std::time::Duration;
use tungstenite::client::AutoStream;
use tungstenite::error::UrlError::UnableToConnect;
use tungstenite::{connect, Message, WebSocket};
use url::Url;

embed_migrations!("migrations/");

// postgres database test helper
// the helper creates a new database for tests and drops it once done
struct PgTestContext {
    default_url: String,
    db_name: String,
}

impl PgTestContext {
    fn new(base_url: &str, default_url: &str, db_name: &str) -> Self {
        // connect to default db and create test db
        let conn =
            PgConnection::establish(default_url).expect("cannot connect to default pg database");
        diesel::sql_query(format!("CREATE DATABASE {}", db_name))
            .execute(&conn)
            .expect("couldn't create test database");

        // connect to test db and run migrations
        let conn_test = PgConnection::establish(&format!("{}/{}", base_url, db_name))
            .expect("cannot connect to test database");
        embedded_migrations::run(&conn_test).expect("running migrations failed");

        PgTestContext {
            default_url: default_url.to_string(),
            db_name: db_name.to_string(),
        }
    }

    fn remove(&mut self) {
        let conn = PgConnection::establish(&self.default_url)
            .expect("cannot connect to default pg database");
        diesel::sql_query(format!(
            "SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE datname = '{}';",
            self.db_name
        ))
        .execute(&conn)
        .expect("cannot disconnect db users");
        diesel::sql_query(format!("DROP DATABASE {}", self.db_name))
            .execute(&conn)
            .expect("cannot drop test database");
    }
}

#[derive(Debug, PartialEq, Eq)]
enum SessionTestLine {
    Client { id: usize, cmd: String },
    Server { id: usize, cmd: String },
}

/// Parse a session test case.
/// Return (lines, number of client/server connections)
fn parse_session_test(test: &str) -> Result<(Vec<SessionTestLine>, usize), String> {
    let mut lines = Vec::new();
    let mut max_id = 0;
    for line in test.split('\n') {
        let line = line.trim();
        // ignore black lines
        if line.len() == 0 {
            continue;
        }
        // ignore comment lines
        if line.len() >= 2
            && line.chars().nth(0).unwrap() == '/'
            && line.chars().nth(1).unwrap() == '/'
        {
            continue;
        }
        if line.len() < 4 {
            return Err(format!(
                "invalid test line: {}: line does not begin with sender specification",
                line
            ));
        }
        if line.chars().nth(0).unwrap() != '[' || line.chars().nth(3).unwrap() != ']' {
            return Err(format!(
                "invalid test line: {}: line does not begin with sender specification",
                line
            ));
        }
        let id = match line.chars().nth(2).unwrap().to_string().parse::<usize>() {
            Ok(id) => id,
            Err(_) => return Err(format!("invalid test line: {}: sender specification should contain server/client id, contains {} instead", line, line.chars().nth(2).unwrap()))
        };
        let parsed = match line.chars().nth(1).unwrap() {
            'C' => SessionTestLine::Client {
                id,
                cmd: line[4..].trim().to_string(),
            },
            'S' => SessionTestLine::Server {
                id,
                cmd: line[4..].trim().to_string(),
            },
            _ => {
                return Err(format!(
                    "invalid test line: {}: sender specification should begin with C or S, not {}",
                    line,
                    line.chars().nth(1).unwrap()
                ))
            }
        };
        lines.push(parsed);
        if id > max_id {
            max_id = id;
        }
    }

    Ok((lines, max_id))
}

/// Check if the contents of a server response match an expected format
/// This is a literal comparison, except that the expected format can include a *, which matches against any non whitespace, non comma, non bracket literal
fn response_matches_expected(response: &str, expect: &str) -> bool {
    let globable = |c: char| !c.is_whitespace() && c != ',' && c != '[' && c != ']';

    let mut resp_iter = response.chars();
    let mut next = resp_iter.next();
    for e in expect.chars() {
        if e != '*' {
            match next {
                Some(c) if c == e => {}
                _ => return false,
            }
            next = resp_iter.next();
        } else {
            while let Some(peek) = resp_iter.next() {
                if !globable(peek) {
                    next = Some(peek);
                    break;
                }
            }
        }
    }

    true
}

/// Run a session test case.
/// A session test case is a list of client commands to send, and expected responses from the server.
/// Multiple client/server connections are supported in a test case. Each line of the test case starts with its sender (in brackets), then contains the command to send to/expect from the server. Clients are C1, C2, C3, etc, and server responses are S1, S2, S3, etc.
/// For example,
/// > [C1] play 1, e2e4
/// > [S1] okay
/// > [S2] go 1, chess, ...
/// > [C2] play 1, e7e5
/// > [S2] okay
/// > [S1] go 1, chess, ...
pub async fn session_test(test: &str) {
    dotenv().ok();

    let mut game_type_map: GameTypeMap = HashMap::new();
    game_type_map.insert("chess", Box::new(games::chess_game::ChessGame()));

    let mut tournament_type_map: TournamentTypeMap = HashMap::new();
    tournament_type_map.insert("round_robin", Box::new(tournament::RoundRobin()));

    // find an open port
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let port = addr.port();
    drop(listener);

    let base_url = env::var("DATABASE_TEST_BASE_URL").expect("DATABASE_TEST_BASE_URL must be set");
    let default_url =
        env::var("DATABASE_TEST_DEFAULT_URL").expect("DATABASE_TEST_DEFAULT_URL must be set");
    let db_name = format!("server_rs_test_{}", port);
    let mut db_test_ctx = PgTestContext::new(&*base_url, &*default_url, &*db_name);

    // start the server
    tokio::spawn((|| async move {
        server::run_server(
            &*format!("127.0.0.1:{}", port),
            &format!("{}/{}", base_url, db_name),
            Arc::new(game_type_map),
            Arc::new(tournament_type_map),
        )
        .await;
    })());

    let ws_url = format!("ws://127.0.0.1:{}", port);

    // wait for server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    while let Err(tungstenite::Error::Url(UnableToConnect(_))) =
        connect(Url::parse(&*ws_url).expect("couldn't parse server url"))
    {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // parse the test case
    let (lines, num_clients) = parse_session_test(test).expect("failed to parse session test case");

    // open connections to server
    let mut conns: Vec<WebSocket<AutoStream>> = (0..num_clients)
        .into_iter()
        .map(|_| {
            connect(Url::parse(&*ws_url).expect("couldn't parse server url"))
                .expect("couldn't connect to server")
                .0
        })
        .collect();

    for line in &lines {
        match line {
            SessionTestLine::Client { id, cmd } => {
                conns[*id - 1]
                    .write_message(Message::Text(cmd.clone()))
                    .expect("can't send message to server");
            }
            SessionTestLine::Server { id, cmd } => {
                let response = conns[*id - 1]
                    .read_message()
                    .expect("error reading message from server")
                    .into_text()
                    .expect("response isn't text");
                if !response_matches_expected(&*response, &**cmd) {
                    panic!("response from server doesn't match expected:\nresponse: [S{}] {}\nexpected: [S{}] {}", *id, response, *id, cmd);
                }
            }
        }
    }

    db_test_ctx.remove();
}

mod tests {
    use super::*;

    #[test]
    fn parse_test() {
        assert_eq!(
            parse_session_test("[C1] cmd1\n[S1] cmd2 arg1\n[C2] cmd3"),
            Ok((
                vec![
                    SessionTestLine::Client {
                        id: 1,
                        cmd: "cmd1".to_string()
                    },
                    SessionTestLine::Server {
                        id: 1,
                        cmd: "cmd2 arg1".to_string()
                    },
                    SessionTestLine::Client {
                        id: 2,
                        cmd: "cmd3".to_string()
                    }
                ],
                2 as usize
            ))
        );
    }
}
