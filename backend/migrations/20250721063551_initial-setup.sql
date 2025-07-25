CREATE OR REPLACE FUNCTION update_updated_at_column()
    RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Game state tracking
CREATE TYPE game_phase AS ENUM (
    'lobby',         -- waiting for members to join
    'santa_id',        -- step 1: anonymously publishing santa IDs
    'seed_commit', -- step 2a: publish seed commitment
    'seed_reveal', -- step 2b: revealing the seed
    'verification',    -- step 3: checking for self-assignments
    'rejected',        -- game rejected due to self-assignments or other issues. wait for members to acknowledge
    'completed'        -- game finished successfully
);

CREATE TABLE room (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    join_code TEXT UNIQUE NOT NULL,

    name TEXT NOT NULL,
    max_members INTEGER,

    game_phase game_phase NOT NULL DEFAULT 'lobby',
    iteration INTEGER NOT NULL DEFAULT 0,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    started_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    deleted_at TIMESTAMP WITH TIME ZONE DEFAULT NULL

    CHECK ( -- game_phase can only be waiting if iteration is 0
      (iteration = 0) OR
      (iteration > 0 AND game_phase != 'lobby')
    )
);

CREATE TRIGGER trigger_update_room_updated_at
    BEFORE UPDATE ON room
    FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();

CREATE TABLE room_member (
    id          UUID PRIMARY KEY         DEFAULT gen_random_uuid(),
    room_id     UUID    NOT NULL REFERENCES room(id) ON DELETE CASCADE,

    name        TEXT NOT NULL,
    fingerprint TEXT UNIQUE NOT NULL,
    public_key  BYTEA NOT NULL,
    is_owner    BOOLEAN NOT NULL         DEFAULT FALSE,

    seed_commitment TEXT,
    seed INT,

    rejected_proof INT,
    result_acknowledged BOOLEAN NOT NULL DEFAULT FALSE, -- whether the member has acknowledged that the game is now complete/rejected

    joined_at   TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    UNIQUE (room_id, seed_commitment), -- each member in a room must have a unique seed commitment
    CHECK (seed IS NULL OR seed_commitment IS NOT NULL) -- must commit before revealing seed
);

CREATE TYPE token_type AS ENUM (
    'session', -- long-lived session token
    'ephemeral', -- short-lived single use token for ephemeral actions
    'challenge' -- short-lived token for challenge verification
);

CREATE TABLE token (
    id          UUID PRIMARY KEY         DEFAULT gen_random_uuid(),
    member_id  UUID    NOT NULL REFERENCES room_member(id) ON DELETE CASCADE,

    token       TEXT UNIQUE NOT NULL,
    type        token_type NOT NULL,

    user_agent  TEXT,
    ip_address  INET,

    created_at  TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_seen_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE santa_id_round (
    id          UUID PRIMARY KEY         DEFAULT gen_random_uuid(),
    room_id     UUID    NOT NULL REFERENCES room(id) ON DELETE CASCADE,

    round_number INTEGER NOT NULL, -- 0 to N, where N is the number of members in the room

    created_at  TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,

    UNIQUE (room_id, round_number),
    CHECK (round_number >= 0)
);

CREATE TABLE santa_id_message (
    id          UUID PRIMARY KEY         DEFAULT gen_random_uuid(),
    round_id    UUID    NOT NULL REFERENCES santa_id_round(id) ON DELETE CASCADE,
    member_id   UUID    NOT NULL REFERENCES room_member(id) ON DELETE CASCADE,

    content     TEXT NOT NULL,

    created_at  TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,

    UNIQUE (round_id, member_id) -- each member can only send one message per round
);