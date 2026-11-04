BEGIN;

CREATE TABLE IF NOT EXISTS users (
    id           BIGINT      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name         TEXT        NOT NULL,
    email        TEXT        NOT NULL,
    provider_iss TEXT,
    provider_sub TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT users_provider_iss_sub_key UNIQUE (provider_iss, provider_sub)
);

INSERT INTO users (name,                email)
SELECT            FORMAT('User %s', index), FORMAT('%s@test.com', index)
FROM              GENERATE_SERIES(1, 250_000) AS series(index);

CREATE TABLE IF NOT EXISTS auth_tokens (
    hash       BYTEA       PRIMARY KEY,
    user_id    BIGINT      NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL
);

COMMIT;
