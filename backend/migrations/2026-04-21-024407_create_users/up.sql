-- Your SQL goes here
-- USERS (Authentication only) 
CREATE TABLE users (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_blind_index       BYTEA NOT NULL,
    email_encrypted         BYTEA NOT NULL,
    encrypted_password      VARCHAR(255) NOT NULL DEFAULT '',

    -- Recoverable
    reset_password_token_digest VARCHAR(255),
    reset_password_sent_at  TIMESTAMPTZ,

    -- Rememberable
    remember_created_at     TIMESTAMPTZ,

    -- Trackable
    sign_in_count           INTEGER NOT NULL DEFAULT 0,
    current_sign_in_at      TIMESTAMPTZ,
    last_sign_in_at         TIMESTAMPTZ,
    current_sign_in_ip      INET,
    last_sign_in_ip         INET,

    -- Confirmable
    confirmation_token_digest VARCHAR(255),
    confirmed_at            TIMESTAMPTZ,
    confirmation_sent_at    TIMESTAMPTZ,
    unconfirmed_email_blind_index BYTEA,
    unconfirmed_email_encrypted   BYTEA,

    -- Lockable
    failed_attempts         INTEGER NOT NULL DEFAULT 0,
    unlock_token_digest     VARCHAR(255),
    locked_at               TIMESTAMPTZ,

    -- 2FA (TOTP)
    otp_secret              VARCHAR(255),
    otp_enabled_at          TIMESTAMPTZ,
    otp_backup_codes        TEXT[],
    encryption_key_version  INTEGER NOT NULL DEFAULT 1,

    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_users_email_blind_index   ON users (email_blind_index);
CREATE UNIQUE INDEX idx_users_confirmation_token_digest
    ON users (confirmation_token_digest)
    WHERE confirmation_token_digest IS NOT NULL;
CREATE UNIQUE INDEX idx_users_reset_token_digest
    ON users (reset_password_token_digest)
    WHERE reset_password_token_digest IS NOT NULL;
CREATE UNIQUE INDEX idx_users_unlock_token_digest
    ON users (unlock_token_digest)
    WHERE unlock_token_digest IS NOT NULL;
CREATE INDEX idx_users_unconfirmed_email_blind_index
    ON users (unconfirmed_email_blind_index)
    WHERE unconfirmed_email_blind_index IS NOT NULL;
