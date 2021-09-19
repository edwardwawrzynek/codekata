#[macro_use]
extern crate diesel_migrations;

mod common;

use common::session_test;

#[tokio::test(flavor = "multi_thread")]
async fn test_version() {
    session_test(
        r#"
[C1] version 2
[S1] okay
    "#,
    )
    .await;
    session_test(
        r#"
[C1] version 1
    "#,
    )
    .await;
    session_test(
        r#"
[C1] version 3
[S1] error invalid protocol version
    "#,
    )
    .await;
    session_test(
        r#"
[C1] version 1
    "#,
    )
    .await;
    session_test(
        r#"
[C1] version 3
[S1] error invalid protocol version
    "#,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_multiple_observe() {
    session_test(
        r#"
[C1] version 2
[S1] okay
[C1] new_tmp_user Test1
[S1] okay
[C1] new_game chess, 100000, 0
[S1] new_game 1
[C1] observe_game 1
[S1] game 1, chess, 1, false, false, -, 100000, 0, -, -, [], -
[C1] observe_game 1
[S1] game 1, chess, 1, false, false, -, 100000, 0, -, -, [], -
[C1] version 2
[S1] okay
    "#,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_user() {
    session_test(
        r#"
[C1] version 2
[S1] okay
[C1] new_tmp_user Test
[S1] okay
[C1] self_user_info
[S1] self_user_info 1, Test, -
[C1] logout
[S1] okay
[C1] self_user_info
[S1] error you are not logged in
[C2] version 2
[S2] okay
[C2] new_tmp_user Test2
[S2] okay
[C2] self_user_info
[S2] self_user_info 2, Test2, -
    "#,
    )
    .await;

    session_test(
        r#"
[C1] version 2
[S1] okay
[C1] new_user Test, test@example.com, password
[S1] okay
[C1] self_user_info
[S1] self_user_info 1, Test, test@example.com
[C2] version 2
[S2] okay
[C2] login test@example.com, password
[S2] okay
[C2] self_user_info
[S2] self_user_info 1, Test, test@example.com
[C2] login test@example.com, random
[S2] error incorrect login credentials
    "#,
    )
    .await;

    session_test(
        r#"
[C1] version 2
[S1] okay
[C1] new_user Test, test@example.com, password
[S1] okay
[C1] self_user_info
[S1] self_user_info 1, Test, test@example.com
[C1] name Name
[S1] okay
[C1] password pass
[S1] okay
[C2] version 2
[S2] okay
[C2] login test@example.com, pass
[S2] okay
[C2] self_user_info
[S2] self_user_info 1, Name, test@example.com
    "#,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_game_create() {
    session_test(
        r#"
[C1] version 2
[S1] okay
[C2] version 2
[S2] okay
[C1] new_tmp_user Test1
[S1] okay
[C2] new_tmp_user Test2
[S2] okay
[C1] new_game chess, 100000, 0
[S1] new_game 1
[C2] new_game chess, 100000, 0
[S2] new_game 2
[C1] join_game 2
[S1] okay
[C2] join_game 2
[S2] okay
[C2] join_game 2
[S2] error you are already in that game
[C1] leave_game 2
[S1] okay
[C1] join_game 2
[S1] okay
[C1] start_game 2
[S1] error you aren't the owner of that game
[C2] start_game 2
[S2] go 2, chess, *, *, rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
[S2] okay
[C1] leave_game 2
[S1] error that game has already started
    "#,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_game_tmp_users_create() {
    session_test(
        r#"
[C1] version 2
[S1] okay
[C1] new_game_tmp_users chess, 100000, 0, 2
[S1] new_game_tmp_users 1, *, *
[C1] version 2
[S1] okay
    "#,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_game_observe() {
    session_test(
        r#"
[C1] version 2
[S1] okay
[C2] version 2
[S2] okay
[C1] new_tmp_user Test1
[S1] okay
[C1] new_game chess, 100000, 0
[S1] new_game 1
[C2] observe_game 1
[S2] game 1, chess, 1, false, false, -, 100000, 0, -, -, [], -
[C1] join_game 1
[S1] okay
[S2] game 1, chess, 1, false, false, -, 100000, 0, -, -, [[1, 0, 100000]], -
[C1] leave_game 1
[S1] okay
[S2] game 1, chess, 1, false, false, -, 100000, 0, -, -, [], -
[C2] stop_observe_game 1
[S2] okay
[C1] join_game 1
[S1] okay
    "#,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_game_play() {
    session_test(r#"
// create a few users to make user id and game player id not match
[C3] version 2
[S3] okay
[C3] new_tmp_user Random1
[S3] okay
[C3] new_tmp_user Random2
[S3] okay
[C3] new_tmp_user Random3
[S3] okay
// create real users
[C1] version 2
[S1] okay
[C2] version 2
[S2] okay
[C1] new_tmp_user Test1
[S1] okay
[C2] new_tmp_user Test2
[S2] okay
[C1] new_game chess, 100000, 0
[S1] new_game 1
[C1] join_game 1
[S1] okay
[C2] join_game 1
[S2] okay
[C1] start_game 1
[S1] go 1, chess, *, *, rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
[S1] okay
[C1] play 1, e2e4
[S1] okay
[S2] go 1, chess, *, *, rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1
[C2] play 1, f7f6
[S2] okay
[S1] go 1, chess, *, *, rnbqkbnr/ppppp1pp/5p2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2
[C1] play 1, a2a3
[S1] okay
[S2] go 1, chess, *, *, rnbqkbnr/ppppp1pp/5p2/8/4P3/P7/1PPP1PPP/RNBQKBNR b KQkq - 0 2
[C2] play 1, g7g5
[S2] okay
[S1] go 1, chess, *, *, rnbqkbnr/ppppp2p/5p2/6p1/4P3/P7/1PPP1PPP/RNBQKBNR w KQkq g6 0 3
[C1] observe_game 1
[S1] game 1, chess, 4, true, false, -, 100000, 0, *, 4, [[4, 0, *], [5, 0, *]], rnbqkbnr/ppppp2p/5p2/6p1/4P3/P7/1PPP1PPP/RNBQKBNR w KQkq g6 0 3,[e2e4,f7f6,a2a3,g7g5]
[C1] play 1, d1h5
[S1] game 1, chess, 4, true, true, 4, 100000, 0, *, -, [[4, 1, *], [5, 0, *]], rnbqkbnr/ppppp2p/5p2/6pQ/4P3/P7/1PPP1PPP/RNB1KBNR b KQkq - 0 3,[e2e4,f7f6,a2a3,g7g5,d1h5]
[S1] okay
[C2] version 2
[S2] okay
[C1] version 2
[S1] okay
    "#).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_game_protocol_versions() {
    session_test(
        r#"
// setup three clients, with C1 & C3 on the same user, but different versions
[C1] version 2
[S1] okay
[C2] version 2
[S2] okay
[C3] version 1
[C1] new_user Test, test@example.com, password
[S1] okay
[C2] new_tmp_user Test2
[S2] okay
[C3] login test@example.com, password
[C1] new_game chess, 100000, 0
[S1] new_game 1
[C3] new_game chess, 100000, 0
[S3] new_game 2
[C1] join_game 1
[S1] okay
[C2] join_game 1
[S2] okay
[C1] join_game 2
[S1] okay
[C2] join_game 2
[S2] okay
[C1] start_game 1
// both C1 & C3 get game 1
[S1] go 1, chess, *, *, rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
[S3] position rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
[S1] okay
[C1] start_game 2
// only C1 gets game 2 (since C3 is in legacy, and only gets one game at a time)
[S1] go 2, chess, *, *, rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
[S1] okay
[C1] play 2, e2e4
[S1] okay
[S2] go 2, chess, *, *, rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1
[C2] play 2, f7f6
[S2] okay
[S1] go 2, chess, *, *, rnbqkbnr/ppppp1pp/5p2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2
// move plays in oldest game (1)
[C3] move e2e4
[S2] go 1, chess, *, *, rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1
[C1] version 1
[C2] play 1, f7f6
[S2] okay
[S1] position rnbqkbnr/ppppp1pp/5p2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2
[S3] position rnbqkbnr/ppppp1pp/5p2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2
[C1] play 1, a1a1
[S1] error that command is only available in protocol version 2 (you are in version 1)
    "#,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_game_expiry() {
    session_test(
        r#"
// setup a game with two players
[C1] version 2
[S1] okay
[C2] version 2
[S2] okay
[C1] new_tmp_user Test1
[S1] okay
[C2] new_tmp_user Test2
[S2] okay
// make game time out quick
[C1] new_game chess, 500, 200
[S1] new_game 1
[C1] join_game 1
[S1] okay
[C2] join_game 1
[S2] okay
[C1] observe_game 1
[S1] game 1, chess, 1, false, false, -, 500, 200, -, -, [[1, 0, 500], [2, 0, 500]], -
[C1] start_game 1
[S1] game 1, chess, 1, true, false, -, 500, 200, *, 1, [[1, 0, 500], [2, 0, 500]], rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1,[]
[S1] go 1, chess, *, *, rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
[S1] okay
// wait for game to time out
[S1] game 1, chess, 1, true, true, 2, 500, 200, *, -, [[1, 0, 0], [2, 0, 500]], __ENDED_GAME, 2, Time Expired, chess, rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1,[]
    "#,
    ).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_tournament_create() {
    session_test(
        r#"
// setup a tournament with three players
[C1] version 2
[S1] okay
[C2] version 2
[S2] okay
[C3] version 2
[S3] okay
[C1] new_tmp_user Test1
[S1] okay
[C2] new_tmp_user Test2
[S2] okay
[C3] new_tmp_user Test3
[S3] okay
[C1] new_tournament round_robin, chess, 100000, 0, 2
[S1] new_tournament 1
[C2] observe_tournament 1
[S2] tournament 1, round_robin, 1, chess, false, false, -, [], []
[C1] join_tournament 1
[S1] okay
[S2] tournament 1, round_robin, 1, chess, false, false, -, [[1, 0, 0, 0]], []
[C1] leave_tournament 1
[S1] okay
[S2] tournament 1, round_robin, 1, chess, false, false, -, [], []
[C1] join_tournament 1
[S1] okay
[S2] tournament 1, round_robin, 1, chess, false, false, -, [[1, 0, 0, 0]], []
[C2] join_tournament 1
[S2] tournament 1, round_robin, 1, chess, false, false, -, [[1, 0, 0, 0], [2, 0, 0, 0]], []
[S2] okay
[C3] join_tournament 1
[S3] okay
[S2] tournament 1, round_robin, 1, chess, false, false, -, [[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0]], []
[C3] start_tournament 1
[S3] error you aren't the owner of that game
[C1] start_tournament 1
// server gives c1 active game
[S1] go 1, chess, *, *, rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
[S1] okay
// server gives c2 observation
[S2] tournament 1, round_robin, 1, chess, true, false, -, [[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0]], []
// games (3 pick 2):
[S2] game 1, chess, 1, false, false, -, 100000, 0, -, -, [[1, 0, 100000], [2, 0, 100000]], -
[S2] game 2, chess, 1, false, false, -, 100000, 0, -, -, [[1, 0, 100000], [3, 0, 100000]], -
[S2] game 3, chess, 1, false, false, -, 100000, 0, -, -, [[2, 0, 100000], [1, 0, 100000]], -
[S2] game 4, chess, 1, false, false, -, 100000, 0, -, -, [[2, 0, 100000], [3, 0, 100000]], -
[S2] game 5, chess, 1, false, false, -, 100000, 0, -, -, [[3, 0, 100000], [1, 0, 100000]], -
[S2] game 6, chess, 1, false, false, -, 100000, 0, -, -, [[3, 0, 100000], [2, 0, 100000]], -
// game 1 starts:
[S2] game 1, chess, 1, true, false, -, 100000, 0, *, 1, [[1, 0, 100000], [2, 0, 100000]], rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1,[]

[C1] version 2
[S1] okay
[C2] version 2
[S2] okay
[C2] version 2
[S2] okay
    "#,
    ).await;
}
