CREATE OR REPLACE FUNCTION update_updated_at_column()
    RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE room (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    join_code TEXT UNIQUE NOT NULL,

    name TEXT NOT NULL,
    max_members INTEGER,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    started_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    deleted_at TIMESTAMP WITH TIME ZONE DEFAULT NULL
);

CREATE TRIGGER trigger_update_room_updated_at
    BEFORE UPDATE ON room
    FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();

CREATE TABLE room_member (
    id          UUID PRIMARY KEY         DEFAULT gen_random_uuid(),
    room_id     UUID    NOT NULL REFERENCES room(id) ON DELETE CASCADE,

    fingerprint TEXT UNIQUE NOT NULL,
    public_key  BYTEA NOT NULL,
    is_owner    BOOLEAN NOT NULL         DEFAULT FALSE,

    joined_at   TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TYPE token_type AS ENUM (
    'session', -- long-lived session token
    'ephemeral', -- short-lived single use token for ephemeral actions
    'challenge' -- short-lived token for challenge verification
);

CREATE TABLE tokens (
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