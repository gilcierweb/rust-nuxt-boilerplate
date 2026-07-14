-- Your SQL goes here
CREATE TABLE audit_logs
(
    id                  UUID PRIMARY KEY      DEFAULT gen_random_uuid(),
    actor_user_id       UUID         REFERENCES users (id) ON DELETE SET NULL,
    actor_role_snapshot VARCHAR(255),
    action              VARCHAR(255) NOT NULL,
    resource_type       VARCHAR(255) NOT NULL,
    resource_id         UUID,
    ip_address          INET,
    user_agent          VARCHAR(500),
    request_id          UUID,
    changes             JSONB        NOT NULL DEFAULT '{}'::jsonb,
    metadata            JSONB        NOT NULL DEFAULT '{}'::jsonb,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_actor_created_at
    ON audit_logs (actor_user_id, created_at DESC) WHERE actor_user_id IS NOT NULL;
CREATE INDEX idx_audit_logs_resource_created_at
    ON audit_logs (resource_type, resource_id, created_at DESC);