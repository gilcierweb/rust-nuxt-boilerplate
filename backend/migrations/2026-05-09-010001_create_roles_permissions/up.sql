-- Many-to-many relation between roles and permissions
CREATE TABLE roles_permissions (
    role_id       UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (role_id, permission_id)
);

CREATE INDEX idx_roles_permissions_role ON roles_permissions (role_id);
CREATE INDEX idx_roles_permissions_permission ON roles_permissions (permission_id);
