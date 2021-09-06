CREATE TABLE tournaments (
    id SERIAL PRIMARY KEY,
    owner_id INTEGER NOT NULL,
    tournament_type TEXT NOT NULL,
    game_type TEXT NOT NULL,
    dur_per_move_ms BIGINT NOT NULL,
    dur_sudden_death_ms BIGINT NOT NULL,
    started BOOLEAN NOT NULL,
    finished BOOLEAN NOT NULL,
    winner INTEGER,
    options TEXT NOT NULL
)