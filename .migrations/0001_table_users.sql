CREATE TABLE IF NOT EXISTS users (
    id             UUID PRIMARY KEY NOT NULL,
    display_name   VARCHAR(64) NOT NULL,
    email          VARCHAR(320) NOT NULL UNIQUE,
    birth_date     TIMESTAMPTZ,
    password       VARCHAR(128),
    username       VARCHAR(32) NOT NULL UNIQUE,
    banner         VARCHAR(511),
    icon           VARCHAR(511),
    bio            TEXT,
    legal_name     VARCHAR(255),
    gender         VARCHAR(50),
    phone          VARCHAR(50),
    country        VARCHAR(100),
    oauth_provider VARCHAR(32) NOT NULL,
    created        TIMESTAMPTZ NOT NULL
);

-- Create indexes for commonly queried fields
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_created ON users(created);
