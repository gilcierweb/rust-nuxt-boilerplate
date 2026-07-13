-- Your SQL goes here
-- PROFILES 
CREATE TABLE profiles (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,

    first_name          VARCHAR(255),
    last_name           VARCHAR(255),
    full_name           VARCHAR(255),
    nickname            VARCHAR(255),
    slug                VARCHAR(255) UNIQUE,
    bio                 TEXT,
    avatar              VARCHAR(500),

    birthday            DATE,
    cpf_encrypted       BYTEA,
    cpf_blind_index     BYTEA,
    phone_encrypted     BYTEA,
    phone_blind_index   BYTEA,
    whatsapp_encrypted  BYTEA,
    whatsapp_blind_index BYTEA,
    status              BOOLEAN NOT NULL DEFAULT TRUE,
    social_network      JSONB NOT NULL DEFAULT '{}',
    encryption_key_version INTEGER NOT NULL DEFAULT 1,

    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE        INDEX idx_profiles_user_id ON profiles (user_id);
CREATE UNIQUE INDEX idx_profiles_cpf_blind_index
    ON profiles (cpf_blind_index)
    WHERE cpf_blind_index IS NOT NULL;
CREATE UNIQUE INDEX idx_profiles_phone_blind_index
    ON profiles (phone_blind_index)
    WHERE phone_blind_index IS NOT NULL;
CREATE UNIQUE INDEX idx_profiles_whatsapp_blind_index
    ON profiles (whatsapp_blind_index)
    WHERE whatsapp_blind_index IS NOT NULL;
