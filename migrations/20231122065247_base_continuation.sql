-- Add migration script here
-- previously on: migrations
-- CREATE TABLE IF NOT EXISTS users (
--     id SERIAL PRIMARY KEY,
--     name text NOT NULL,
--     discord_id bigint NOT NULL
-- );

-- CREATE TABLE IF NOT EXISTS slidingpuzzle (
--     -- singleplayer game, so we will be keeping track of the users score and time
--     id SERIAL PRIMARY KEY,
--     user_id integer NOT NULL REFERENCES users(id),
--     difficulty integer NOT NULL, -- 1 = easy, 2 = medium, 3 = hard
--     size integer NOT NULL, -- 3 = 3x3, 4 = 4x4, 5 = 5x5
--     score integer NOT NULL, -- The users score
--     time integer NOT NULL, -- The users time
--     created_at timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
-- );

CREATE TABLE IF NOT EXISTS tictactoe (
    -- multiplayer game, so we'll be keeping track of only if the player won or lost
    id SERIAL PRIMARY KEY,
    user_id integer NOT NULL REFERENCES users(id),
    opponent_id integer NOT NULL REFERENCES users(id), -- not really used, we double up on the entries so its easier to query
    won boolean NOT NULL, -- true if the user won, false if the user lost. we dont keep track of draws, draws are lame
    created_at timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS ultimate_tictactoe (
    -- multiplayer game, so we'll be keeping track of only if the player won or lost
    id SERIAL PRIMARY KEY,
    user_id integer NOT NULL REFERENCES users(id),
    opponent_id integer NOT NULL REFERENCES users(id), -- not really used, we double up on the entries so its easier to query
    won boolean NOT NULL, -- true if the user won, false if the user lost. we dont keep track of draws, draws are lame
    created_at timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);