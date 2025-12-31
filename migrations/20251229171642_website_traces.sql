CREATE TYPE request_status AS ENUM (
    'success', -- 2xx
    'redirect', -- 3xx
    'client_error', -- 4xx
    'server_error', -- 5xx
    'unauthorized' -- 401/403
    );

CREATE TABLE website_traces
(
    trace_id        UUID PRIMARY KEY                  DEFAULT uuidv7(),

    method          TEXT                     NOT NULL CHECK (method IN ('GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS')),
    path            TEXT                     NOT NULL CHECK ( path <> '' ),
    query           TEXT,
    ip_address      TEXT,
    user_agent      TEXT,
    user_id         UUID                     REFERENCES users (id) ON DELETE SET NULL,

    started_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    duration_ms     BIGINT                   NOT NULL,
    status_code     INTEGER                  NOT NULL,
    request_status  request_status           NOT NULL DEFAULT 'success',

    data            JSONB                    NOT NULL,
    request_headers JSONB                    NOT NULL,
    request_body    JSONB,
    response_body   JSONB,
    error           TEXT,

    created_at      TIMESTAMPTZ              NOT NULL DEFAULT now()
);

CREATE INDEX idx_website_traces_started_at ON website_traces (started_at DESC);
CREATE INDEX idx_website_traces_user_id ON website_traces (user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_website_traces_status_code ON website_traces (status_code);
CREATE INDEX idx_website_traces_request_status ON website_traces (request_status);

CREATE INDEX idx_website_traces_data_gin ON website_traces USING GIN (data);
CREATE INDEX idx_website_traces_request_headers_gin ON website_traces USING GIN (request_headers);