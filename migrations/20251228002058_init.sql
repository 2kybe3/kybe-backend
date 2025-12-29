CREATE TABLE command_traces (
    trace_id UUID PRIMARY KEY NOT NULL DEFAULT uuidv7(),
    command TEXT NOT NULL,
    user_id BIGINT NOT NULL,
    username TEXT NOT NULL,
    guild_id BIGINT NULL,
    channel_id BIGINT NOT NULL,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL,
    duration_ms BIGINT NOT NULL,
    status TEXT NOT NULL,
    input jsonb NOT NULL,
    data jsonb,
    output TEXT,
    error TEXT
);