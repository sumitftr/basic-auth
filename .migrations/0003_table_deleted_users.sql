CREATE TABLE IF NOT EXISTS deleted_users (
    id             UUID PRIMARY KEY NOT NULL,
    display_name   VARCHAR(255) NOT NULL,
    email          VARCHAR(320) NOT NULL,
    birth_date     TIMESTAMPTZ,
    password       VARCHAR(128),
    username       VARCHAR(32) NOT NULL,
    banner         VARCHAR(511),
    icon           VARCHAR(511),
    bio            TEXT,
    legal_name     VARCHAR(255),
    gender         VARCHAR(50),
    phone          VARCHAR(50),
    country        VARCHAR(100),
    oauth_provider VARCHAR(50),
    created        TIMESTAMPTZ NOT NULL,
    deleted        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for commonly queried fields
CREATE INDEX IF NOT EXISTS idx_deleted_users_created ON deleted_users(created);
CREATE INDEX IF NOT EXISTS idx_deleted_users_deleted ON deleted_users(deleted);
