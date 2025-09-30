CREATE TABLE users
(
    id            UUID PRIMARY KEY             DEFAULT gen_random_uuid(),

    -- Identity
    username      VARCHAR(40) UNIQUE  NOT NULL,
    first_name    VARCHAR(100)        NOT NULL,
    last_name     VARCHAR(100)        NOT NULL,
    email         VARCHAR(255) UNIQUE NOT NULL,

    -- Authentication
    password_hash TEXT                NOT NULL,
    token_version INTEGER             NOT NULL DEFAULT 0,

    -- Status & Activity
    is_active     BOOLEAN             NOT NULL DEFAULT TRUE,
    locked_until  TIMESTAMPTZ                  DEFAULT NULL,
    last_login_at TIMESTAMPTZ                  DEFAULT NULL,

    -- Soft Delete
    deleted_at    TIMESTAMPTZ                  DEFAULT NULL,

    created_at    TIMESTAMPTZ         NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ         NOT NULL DEFAULT now()
);

-- default owner: password_hash = password123
-- TODO: Remove after given role to another user
INSERT INTO users (first_name, last_name, username, email, password_hash)
VALUES ('Owner', 'Admin', 'owner', 'owner@example.com',
        'eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI4ZmYyMjA2MS1mZjU3LTQwOGYtYTYxNC0zMjc0YjhhZTUwYjEiLCJ1c2VybmFtZSI6IkrDuHJnZW5zZW4iLCJ0b2tlbl92ZXJzaW9uIjowLCJleHAiOjM2MDB9.pt0Al8oRwpPet1Ulh-VBhkTtIZcZimd_fEUj6NMYLKI') ON CONFLICT (username) DO NOTHING;
