-- Add migration script here (POSTGRESQL)
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name text NOT NULL,
    discord_id bigint NOT NULL
);

CREATE TABLE IF NOT EXISTS slidingpuzzle (
    -- singleplayer game, so we will be keeping track of the users score and time
    id SERIAL PRIMARY KEY,
    user_id integer NOT NULL REFERENCES users(id),
    difficulty integer NOT NULL, -- 1 = easy, 2 = medium, 3 = hard
    size integer NOT NULL, -- 3 = 3x3, 4 = 4x4, 5 = 5x5
    score integer NOT NULL, -- The users score
    time integer NOT NULL, -- The users time
    created_at timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);