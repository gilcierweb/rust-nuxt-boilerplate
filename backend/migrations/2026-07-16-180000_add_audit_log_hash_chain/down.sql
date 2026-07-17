-- Remove cryptographic hash chaining columns from audit_logs
DROP INDEX IF EXISTS idx_audit_logs_hash_chain;
ALTER TABLE audit_logs
DROP COLUMN IF EXISTS prev_hash,
DROP COLUMN IF EXISTS hash;