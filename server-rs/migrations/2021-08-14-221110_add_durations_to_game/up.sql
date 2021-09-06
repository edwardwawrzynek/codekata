ALTER TABLE games
    ADD COLUMN dur_per_move_ms BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN dur_sudden_death_ms BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN current_move_start_ms BIGINT DEFAULT NULL
