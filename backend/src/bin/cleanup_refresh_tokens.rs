//! Refresh Token Cleanup Binary
//!
//! This binary cleans up expired and revoked refresh tokens from the database.
//! Run periodically (e.g., daily via cron) to prevent the refresh_tokens table
//! from growing unbounded with expired/revoked tokens.
//!
//! Usage: cargo run --bin cleanup_refresh_tokens

use chrono::Utc;
use diesel::delete;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use diesel::sql_types::*;
use diesel::sql_query;

use std::env;

#[path = "../db/schema.rs"]
mod schema;

use self::schema::refresh_tokens as refresh_tokens_table;

type PgPool = r2d2::Pool<ConnectionManager<PgConnection>>;
type CleanupResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn main() -> CleanupResult<()> {
    // Initialize logging
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/boilerplate_dev".to_string());

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .max_size(1)
        .build(manager)
        .expect("Failed to create pool");

    let mut conn = pool.get()?;

    println!("🧹 Starting refresh token cleanup at {}", Utc::now());

    // 1. Delete expired tokens (expires_at < NOW)
    let deleted_expired = delete(
        refresh_tokens_table::table.filter(refresh_tokens_table::expires_at.lt(Utc::now().naive_utc()))
    )
    .execute(&mut conn)?;
    println!("  ✅ Deleted {} expired tokens", deleted_expired);

    // 2. Delete revoked tokens older than 30 days
    let thirty_days_ago = Utc::now() - chrono::Duration::days(30);
    let deleted_revoked = delete(
        refresh_tokens_table::table
            .filter(refresh_tokens_table::revoked_at.is_not_null())
            .filter(refresh_tokens_table::revoked_at.lt(thirty_days_ago.naive_utc())),
    )
    .execute(&mut conn)?;
    println!("  ✅ Deleted {} revoked tokens (older than 30 days)", deleted_revoked);

    // 3. Delete excess valid tokens per user (keep only 5 most recent valid tokens per user)
    let deleted_per_user = sql_query(
        r#"
        DELETE FROM refresh_tokens rt1
        USING (
            SELECT rt2.id
            FROM refresh_tokens rt2
            WHERE rt2.revoked_at IS NULL
              AND rt2.expires_at > NOW()
            WINDOW w AS (PARTITION BY rt2.user_id ORDER BY rt2.expires_at DESC)
            QUALIFY ROW_NUMBER() OVER w > 5
        ) sub
        WHERE rt1.id = sub.id
        "#,
    )
    .execute(&mut conn)?;
    println!("  ✅ Deleted {} excess valid tokens (keeping 5 most recent per user)", deleted_per_user);

    let total = deleted_expired + deleted_revoked + deleted_per_user;
    println!("\n✨ Cleanup complete at {}: {} total tokens removed", Utc::now(), total);

    Ok(())
}