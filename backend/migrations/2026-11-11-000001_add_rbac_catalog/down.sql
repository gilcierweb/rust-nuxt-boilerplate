-- Reverts the rbac catalog schema migration. Reverse order of up.sql.
-- No data was inserted by this migration, so nothing needs deleting.

DROP INDEX uq_roles_scope;

ALTER TABLE roles_permissions DROP COLUMN effect;

DROP INDEX uq_permissions_name;
ALTER TABLE permissions DROP COLUMN name;
