CREATE TYPE command_status AS ENUM (
    'success',
    'error',
    'disabled',
    'unauthorized'
);

CREATE TABLE command_traces (
    trace_id        UUID PRIMARY KEY NOT NULL DEFAULT uuidv7(),

    command         TEXT NOT NULL CHECK ( command <> '' ),
    user_id         BIGINT NOT NULL,
    username        TEXT NOT NULL CHECK ( username <> '' ),
    guild_id        BIGINT,
    channel_id      BIGINT NOT NULL,

    started_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    duration_ms     BIGINT NOT NULL,
    status          command_status NOT NULL DEFAULT 'success',

    input           jsonb NOT NULL DEFAULT '{}',
    data            jsonb NOT NULL DEFAULT '{}',
    output          TEXT,
    error           TEXT,

    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_command_traces_started_at    ON command_traces (started_at DESC);
CREATE INDEX idx_command_traces_user_id       ON command_traces (user_id);
CREATE INDEX idx_command_traces_guild_id      ON command_traces (guild_id) WHERE guild_id IS NOT NULL;
CREATE INDEX idx_command_traces_status        ON command_traces (status);

CREATE INDEX idx_command_traces_input_gin     ON command_traces USING GIN (input);
CREATE INDEX idx_command_traces_data_gin      ON command_traces USING GIN (data);