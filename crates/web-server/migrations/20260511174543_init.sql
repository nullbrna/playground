CREATE TABLE users (
    id         SERIAL       PRIMARY KEY,
    first_name VARCHAR(100) NOT NULL,
    last_name  VARCHAR(100) NOT NULL,
    age        INTEGER      NOT NULL,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE idempotency (
    key        TEXT        PRIMARY KEY,
    status     SMALLINT    NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
)
