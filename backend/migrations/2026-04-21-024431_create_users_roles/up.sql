-- Your SQL goes here
-- USERS_ROLES (pivot, no surrogate PK — matches Devise/Rolify) 
CREATE TABLE users_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

CREATE INDEX idx_users_roles_user ON users_roles (user_id);
CREATE INDEX idx_users_roles_role ON users_roles (role_id);