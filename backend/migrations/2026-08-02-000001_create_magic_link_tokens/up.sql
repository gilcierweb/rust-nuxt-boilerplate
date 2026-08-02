-- MAGIC LINK TOKENS
--
-- Short-lived, single-use tokens used for passwordless authentication.
-- When a user requests a magic link, we store only the HMAC digest of the
-- token (never the plaintext) along with an expiry and consumption marker.
-- A consumed token can never be replayed; an expired token is rejected.
--
-- Unlike `refresh_tokens` (long-lived, multi-use) and the user-level
-- `reset_password_token_digest` (single-use, but reuses the same column as
-- a "latest reset" pointer), magic links deserve their own table because:
--   1. A user may have several valid magic links in flight (e.g. requested
--      twice within the expiry window) — we keep them all so the second
--      request doesn't invalidate the first one.
--   2. Tokens are short-lived (15 minutes) and consumed on first use.
--   3. Keeping the lifecycle isolated makes revocation and observability
--      (rate-limit-by-token) trivial without touching password reset flow.

CREATE TABLE magic_link_tokens (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_digest  VARCHAR(255) NOT NULL,
    request_ip    INET,
    user_agent    TEXT,
    expires_at    TIMESTAMPTZ NOT NULL,
    consumed_at   TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_magic_link_tokens_digest ON magic_link_tokens (token_digest);
CREATE        INDEX idx_magic_link_tokens_user   ON magic_link_tokens (user_id);
CREATE        INDEX idx_magic_link_tokens_expiry ON magic_link_tokens (expires_at)
    WHERE consumed_at IS NULL;
