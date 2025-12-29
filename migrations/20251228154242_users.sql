CREATE TABLE users (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuidv7(),
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    email_verified BOOLEAN NOT NULL DEFAULT false,

    password_hash TEXT NOT NULL,

    discord_id TEXT UNIQUE,
    discord_linked TIMESTAMP WITH TIME ZONE,

    last_password_change TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    last_login TIMESTAMP WITH TIME ZONE,

    role TEXT NOT NULL DEFAULT 'user',

    CONSTRAINT user_role_check CHECK ( role in ('user', 'admin') ),
    CONSTRAINT user_discord_check CHECK (
        (discord_id IS NULL AND discord_linked IS NULL) OR
        (discord_id IS NOT NULL AND discord_linked IS NOT NULL)
    )
);