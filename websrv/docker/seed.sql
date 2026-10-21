BEGIN;

CREATE TABLE IF NOT EXISTS users (
    id           BIGINT      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name         TEXT        NOT NULL,
    email        TEXT        NOT NULL UNIQUE,
    provider_sub TEXT        UNIQUE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO users (name,                email)
SELECT            FORMAT('User %s', i), FORMAT('%s@test.com', i)
FROM              GENERATE_SERIES(1, 250_000) AS series(i)
ON CONFLICT                             (email) DO NOTHING;

CREATE TABLE IF NOT EXISTS tokens (
    hash       BYTEA       PRIMARY KEY,
    user_id    BIGINT      NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL
);

COMMIT;
