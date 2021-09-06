CREATE TABLE game_players (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    game_id INTEGER NOT NULL,
    score DOUBLE PRECISION
)