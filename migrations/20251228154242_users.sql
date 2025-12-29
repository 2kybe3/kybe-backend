CREATE TYPE user_role AS ENUM (
    'user',
    'admin'
);

CREATE TABLE users (
    id                      UUID PRIMARY KEY NOT NULL DEFAULT uuidv7(),

    username                TEXT NOT NULL UNIQUE CHECK ( username <> '' ),
    email                   TEXT NOT NULL UNIQUE CHECK ( email ~* '^[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}$' ),

    email_verified          BOOLEAN NOT NULL DEFAULT false,

    password_hash           TEXT NOT NULL CHECK ( password_hash <> '' ),

    discord_id              TEXT UNIQUE,
    discord_linked          TIMESTAMPTZ,

    last_password_change    TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login              TIMESTAMPTZ,

    role                    user_role NOT NULL DEFAULT 'user',

    CONSTRAINT user_discord_check CHECK (
        (discord_id IS NULL AND discord_linked IS NULL) OR
        (discord_id IS NOT NULL AND discord_linked IS NOT NULL)
    )
);

CREATE INDEX idx_users_email            ON users (email);
CREATE INDEX idx_users_discord_id       ON users (discord_id) WHERE discord_id IS NOT NULL;
CREATE INDEX idx_users_role             ON users (role);
CREATE INDEX idx_users_created_at       ON users (created_at DESC);
CREATE INDEX idx_users_last_login       ON users (last_login DESC);