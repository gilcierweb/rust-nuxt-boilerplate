-- Add cryptographic hash chaining columns to audit_logs for tamper resistance
-- prev_hash: Hash of the previous audit log entry (for chaining)
-- hash: SHA-256 hash of this entry's content + prev_hash
ALTER TABLE audit_logs
ADD COLUMN prev_hash VARCHAR(64),
ADD COLUMN hash VARCHAR(64) NOT NULL DEFAULT '';

-- Create index for efficient chain verification
CREATE INDEX idx_audit_logs_hash_chain ON audit_logs (prev_hash);

-- Create index for efficient chronological ordering (used by chain verification)
CREATE INDEX idx_audit_logs_created_at ON audit_logs (created_at ASC);