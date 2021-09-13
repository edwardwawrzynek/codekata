table! {
    game_players (id) {
        id -> Int4,
        user_id -> Int4,
        game_id -> Int4,
        score -> Nullable<Float8>,
        waiting_for_move -> Bool,
        time_ms -> Int8,
    }
}

table! {
    games (id) {
        id -> Int4,
        owner_id -> Int4,
        game_type -> Text,
        state -> Nullable<Text>,
        finished -> Bool,
        winner -> Nullable<Int4>,
        is_tie -> Nullable<Bool>,
        dur_per_move_ms -> Int8,
        dur_sudden_death_ms -> Int8,
        current_move_start_ms -> Nullable<Int8>,
        turn_id -> Nullable<Int8>,
        tournament_id -> Nullable<Int4>,
    }
}

table! {
    tournament_players (id) {
        id -> Int4,
        user_id -> Int4,
        tournament_id -> Int4,
        win -> Int4,
        loss -> Int4,
        tie -> Int4,
    }
}

table! {
    tournaments (id) {
        id -> Int4,
        owner_id -> Int4,
        tournament_type -> Text,
        game_type -> Text,
        dur_per_move_ms -> Int8,
        dur_sudden_death_ms -> Int8,
        started -> Bool,
        finished -> Bool,
        winner -> Nullable<Int4>,
        options -> Text,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Nullable<Text>,
        name -> Text,
        is_admin -> Bool,
        password_hash -> Nullable<Text>,
        api_key_hash -> Text,
    }
}

allow_tables_to_appear_in_same_query!(game_players, games, tournament_players, tournaments, users,);
