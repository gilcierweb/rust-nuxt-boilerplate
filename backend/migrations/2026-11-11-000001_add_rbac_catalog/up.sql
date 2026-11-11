-- RBAC catalog schema: human readable names on permissions, the deny aware
-- effect on the role permission link, and a unique role scope. DDL only:
-- the canonical catalog rows are seeded by the seed module, never by a
-- migration. Fully additive: nothing existing is revoked.

ALTER TABLE permissions ADD COLUMN name VARCHAR(255);
-- Backfill required before the NOT NULL constraint: existing rows keep the
-- machine code as their initial human readable name.
UPDATE permissions SET name = code;
ALTER TABLE permissions ALTER COLUMN name SET NOT NULL;
CREATE UNIQUE INDEX uq_permissions_name ON permissions (name);

ALTER TABLE roles_permissions ADD COLUMN effect SMALLINT NOT NULL DEFAULT 0;

-- Unique role scope (name + scope pair). Fails with the native Postgres
-- error when a deployment holds duplicate roles that must be merged manually.
CREATE UNIQUE INDEX uq_roles_scope ON roles
  (name, COALESCE(resource_type, ''), COALESCE(resource_id, '00000000-0000-0000-0000-000000000000'::uuid));
