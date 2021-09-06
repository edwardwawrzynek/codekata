CREATE TABLE games (
    id SERIAL PRIMARY KEY,
    owner_id INTEGER NOT NULL,
    game_type TEXT NOT NULL,
    state TEXT,
    finished BOOLEAN NOT NULL,
    winner INTEGER,
    is_tie BOOLEAN
)