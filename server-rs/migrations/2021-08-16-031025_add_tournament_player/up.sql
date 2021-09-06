CREATE TABLE tournament_players (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    tournament_id INTEGER NOT NULL,
    win INTEGER NOT NULL,
    loss INTEGER NOT NULL,
    tie INTEGER NOT NULL
)