-- Your SQL goes here
-- ROLES 
CREATE TABLE roles (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(50)  NOT NULL,
    resource_type   VARCHAR(255),
    resource_id     UUID,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_roles_name_resource ON roles (name, resource_type, resource_id);

-- SEED: Default roles 
INSERT INTO roles (name) VALUES
    ('admin'),
    ('moderator'),
    ('creator'),
    ('subscriber'),
    ('agency'),
    ('support');