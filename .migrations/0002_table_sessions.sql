CREATE TABLE IF NOT EXISTS sessions (
    unsigned_ssid  UUID PRIMARY KEY,

    user_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    user_agent     TEXT,
    ip_address     INET NOT NULL,

    created_at     TIMESTAMPTZ NOT NULL,
    last_used      TIMESTAMPTZ NOT NULL,
    expires_at     TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at);
