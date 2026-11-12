use std::collections::HashMap;
use std::env;

use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, SaltString};
use backend::config::AppConfig;
use backend::security::SecurityService;
use backend::services::email_service::EmailService;
use backend::services::rbac_catalog_service;
use chrono::Utc;
use diesel::dsl::count_star;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use faker_rust::name;
use ipnet::IpNet;
use uuid::Uuid;

#[path = "../db/schema.rs"]
mod schema;
type PgPool = r2d2::Pool<ConnectionManager<PgConnection>>;
type SeedResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone)]
struct SeedUserData {
    first_name: String,
    last_name: String,
    full_name: String,
    nickname: String,
}

fn sanitize_seed_text(input: &str, fallback: &str, max_chars: usize) -> String {
    let sanitized = input
        .chars()
        .filter(|c| !c.is_control() || c.is_whitespace())
        .collect::<String>();
    let mut bounded = if sanitized.trim().is_empty() {
        fallback.to_string()
    } else {
        sanitized
    };

    if bounded.chars().count() > max_chars {
        bounded = bounded.chars().take(max_chars).collect::<String>();
    }

    if bounded.is_empty() {
        fallback.to_string()
    } else {
        bounded
    }
}

fn test_user_email(user_index: usize) -> String {
    if user_index == 1 {
        "admin@example.com".to_string()
    } else {
        format!("user{user_index:02}@example.com")
    }
}

/// The canonical rotation of the non admin seed users: every subject outside
/// the first holds one of these roles, taken from the catalog the migration
/// seeds. The seed never invents a role of its own.
const NON_ADMIN_ROTATION: [&str; 5] = ["manager", "editor", "viewer", "support", "customer"];

fn role_for(user_index: usize) -> &'static str {
    if user_index == 1 {
        "admin"
    } else {
        NON_ADMIN_ROTATION[(user_index - 2) % NON_ADMIN_ROTATION.len()]
    }
}

fn generate_user_data(user_index: usize) -> SeedUserData {
    if user_index == 1 {
        return SeedUserData {
            first_name: "Admin".to_string(),
            last_name: "User".to_string(),
            full_name: "Admin User".to_string(),
            nickname: "Admin".to_string(),
        };
    }

    let first_name = sanitize_seed_text(&name::first_name(), "Test", 255);
    let last_name = sanitize_seed_text(&name::last_name(), "User", 255);
    let full_name = sanitize_seed_text(&format!("{first_name} {last_name}"), "Test User", 255);

    SeedUserData {
        full_name,
        nickname: first_name.clone(),
        first_name,
        last_name,
    }
}

fn pool() -> SeedResult<PgPool> {
    let database_url = std::env::var("DATABASE_URL")?;
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Ok(r2d2::Pool::builder().max_size(4).build(manager)?)
}

/// Thin wrapper that re-exports the SecurityService so seed callers can use
/// the exact same email-protect pipeline as the runtime (normalize -> HMAC
/// blind index + AES-GCM encrypt). This eliminates drift between seed and
/// runtime hash outputs and prevents login failures caused by divergent
/// normalization rules.
fn build_security_service() -> SeedResult<SecurityService> {
    SecurityService::from_env().map_err(|e| format!("failed to build security service: {e}").into())
}

/// Send all test emails via the configured email transport.
/// Requires EMAIL_TRANSPORT=smtp and SMTP_URL set (e.g., for MailCatcher).
async fn send_test_emails() -> SeedResult<()> {
    // Load config from environment (respects EMAIL_TRANSPORT, SMTP_URL, FRONTEND_URL, etc.)
    let config = AppConfig::from_env().map_err(|e| format!("failed to load AppConfig: {e}"))?;

    // Check if email service is configured
    use backend::config::app_config::EmailTransportKind;
    if config.email_transport != EmailTransportKind::Smtp && config.resend_api_key.is_empty() {
        println!(
            "⚠️  Email service not configured. Set EMAIL_TRANSPORT=smtp and SMTP_URL for MailCatcher,"
        );
        println!("   or RESEND_API_KEY for Resend API.");
        return Ok(());
    }

    // Build email service
    let email_service = EmailService::new(&config);

    if !email_service.is_configured() {
        println!("⚠️  Email service not configured. Set EMAIL_TRANSPORT=smtp and SMTP_URL,");
        println!("   or RESEND_API_KEY for Resend API.");
        return Ok(());
    }

    let test_email = env::var("TEST_EMAIL").unwrap_or_else(|_| "test@example.com".to_string());
    let frontend_url = config.frontend_url.trim_end_matches('/');
    let magic_token = "test_magic_token_1234567890123456789012345678901234";
    let reset_token = "test_reset_token_1234567890123456789012345678901234";
    let confirm_token = "test_confirm_token_1234567890123456789012345678901234";

    println!("📧 Sending test emails to: {}", test_email);
    println!("   Transport: {:?}", email_service.transport_kind());
    println!("   Frontend URL: {}", frontend_url);

    // 1. Magic Link
    println!("\n1/5 Sending magic link email...");
    let magic_url = format!(
        "{}/auth/magic-link-verify?token={}",
        frontend_url, magic_token
    );
    email_service
        .send_magic_link_email(&test_email, &magic_url, "pt-BR")
        .await
        .map_err(|e| format!("magic link failed: {e}"))?;
    println!("   ✓ Magic link sent");

    // 2. Password Reset
    println!("\n2/5 Sending password reset email...");
    let reset_url = format!("{}/auth/reset-password?token={}", frontend_url, reset_token);
    email_service
        .send_password_reset_email(&test_email, &reset_url, "pt-BR")
        .await
        .map_err(|e| format!("password reset failed: {e}"))?;
    println!("   ✓ Password reset sent");

    // 3. Email Confirmation (Welcome)
    println!("\n3/5 Sending email confirmation (welcome) email...");
    let confirm_url = format!("{}/auth/confirm?token={}", frontend_url, confirm_token);
    email_service
        .send_confirmation_email(&test_email, &confirm_url, "pt-BR")
        .await
        .map_err(|e| format!("confirmation failed: {e}"))?;
    println!("   ✓ Confirmation sent");

    // 4. Password Changed Notification
    println!("\n4/5 Sending password changed notification...");
    email_service
        .send_password_changed_notification(&test_email, "pt-BR")
        .await
        .map_err(|e| format!("password changed failed: {e}"))?;
    println!("   ✓ Password changed notification sent");

    // 5. 2FA Setup
    println!("\n5/5 Sending 2FA setup email...");
    let secret = "JBSWY3DPEHPK3PXP"; // Example base32 secret
    let qr_code_url = format!("{}/auth/2fa/qr?secret={}", frontend_url, secret);
    let backup_codes = vec![
        "12345678".to_string(),
        "87654321".to_string(),
        "11223344".to_string(),
        "44332211".to_string(),
        "55667788".to_string(),
    ];
    email_service
        .send_2fa_setup_email(&test_email, secret, &qr_code_url, &backup_codes, "pt-BR")
        .await
        .map_err(|e| format!("2fa setup failed: {e}"))?;
    println!("   ✓ 2FA setup sent");

    println!("\n✅ All test emails sent!");
    println!(
        "\n📬 Check MailCatcher at http://localhost:1080 (UI) or http://localhost:1025 (SMTP)"
    );
    println!("   Or check your email inbox if using Resend.");

    Ok(())
}

fn password_hash(password: &str) -> SeedResult<String> {
    let mut rng = rand::thread_rng();
    let salt = SaltString::generate(&mut rng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("failed to hash password: {e}"))?
        .to_string())
}

fn ensure_refresh_token(
    conn: &mut PgConnection,
    user_uuid: Uuid,
    seed_index: usize,
) -> SeedResult<Uuid> {
    use schema::refresh_tokens::dsl::*;

    let token_hash_value = format!("seed-refresh-token-{seed_index:02}");

    if let Some(existing_id) = refresh_tokens
        .filter(token_hash.eq(&token_hash_value))
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    Ok(diesel::insert_into(refresh_tokens)
        .values((
            id.eq(Uuid::new_v4()),
            user_id.eq(user_uuid),
            token_hash.eq(token_hash_value),
            device_info.eq(Some(format!("Seed Device {seed_index:02}"))),
            ip_address.eq(Some("127.0.0.1".to_string())),
            expires_at.eq(Utc::now() + chrono::Duration::days(30)),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

fn ensure_user(
    conn: &mut PgConnection,
    security: &SecurityService,
    user_index: usize,
) -> SeedResult<Uuid> {
    use schema::users::dsl::*;

    let user_email = test_user_email(user_index);
    let user_password = "password123";
    let protected = security
        .protect_email(&user_email)
        .map_err(|e| format!("failed to protect seed email: {e}"))?;
    let email_bidx = protected.blind_index;
    let email_enc = protected.encrypted;
    let key_version = protected.key_version;
    let pwd_hash = password_hash(user_password)?;

    if let Some(existing_id) = users
        .filter(email_blind_index.eq(email_bidx.clone()))
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    Ok(diesel::insert_into(users)
        .values((
            id.eq(Uuid::new_v4()),
            email_blind_index.eq(email_bidx),
            email_encrypted.eq(email_enc),
            encrypted_password.eq(pwd_hash),
            confirmed_at.eq(Some(Utc::now())),
            encryption_key_version.eq(key_version),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

fn ensure_profile(
    conn: &mut PgConnection,
    user_uuid: Uuid,
    _user_index: usize,
    user_data: &SeedUserData,
) -> SeedResult<Uuid> {
    use schema::profiles::dsl::*;

    if let Some(existing_id) = profiles
        .filter(user_id.eq(user_uuid))
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    Ok(diesel::insert_into(profiles)
        .values((
            id.eq(Uuid::new_v4()),
            user_id.eq(user_uuid),
            first_name.eq(Some(user_data.first_name.clone())),
            last_name.eq(Some(user_data.last_name.clone())),
            full_name.eq(Some(user_data.full_name.clone())),
            nickname.eq(Some(user_data.nickname.clone())),
            bio.eq(Some("Seeded user profile".to_string())),
            avatar.eq(Some(format!(
                "https://api.dicebear.com/9.x/initials/svg?seed={}",
                user_data.full_name.replace(' ', "%20")
            ))),
            status.eq(true),
            social_network.eq(serde_json::json!({})),
            encryption_key_version.eq(1_i32),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

/// Converges the links of a seed user to the desired set.
///
/// The old seed only inserted, so a second run accumulated links: the broken
/// catalog of a previous run stayed in place and nobody could tell which roles
/// a subject really held. The link table has no second owner, so the links the
/// desired state does not name are removed and the missing ones inserted.
fn ensure_user_roles(conn: &mut PgConnection, user_uuid: Uuid, desired: &[Uuid]) -> SeedResult<()> {
    use schema::users_roles::dsl::*;

    let stored: Vec<Uuid> = users_roles
        .filter(user_id.eq(user_uuid))
        .select(role_id)
        .load::<Uuid>(conn)?;

    let stale: Vec<Uuid> = stored
        .iter()
        .copied()
        .filter(|stored_role| !desired.contains(stored_role))
        .collect();
    if !stale.is_empty() {
        diesel::delete(users_roles.filter(user_id.eq(user_uuid).and(role_id.eq_any(stale))))
            .execute(conn)?;
    }

    let missing: Vec<Uuid> = desired
        .iter()
        .copied()
        .filter(|desired_role| !stored.contains(desired_role))
        .collect();
    for role_uuid in missing {
        diesel::insert_into(users_roles)
            .values((user_id.eq(user_uuid), role_id.eq(role_uuid)))
            .execute(conn)?;
    }

    Ok(())
}

fn ensure_audit_log(
    conn: &mut PgConnection,
    user_uuid: Uuid,
    seed_index: usize,
) -> SeedResult<Uuid> {
    use schema::audit_logs::dsl::*;

    let request_uuid = Uuid::from_u128(10_000 + seed_index as u128);

    if let Some(existing_id) = audit_logs
        .filter(request_id.eq(Some(request_uuid)))
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    Ok(diesel::insert_into(audit_logs)
        .values((
            id.eq(Uuid::new_v4()),
            actor_user_id.eq(Some(user_uuid)),
            actor_role_snapshot.eq(Some(if seed_index == 1 {
                "admin".to_string()
            } else {
                "user".to_string()
            })),
            action.eq("seed.create"),
            resource_type.eq("user"),
            resource_id.eq(Some(user_uuid)),
            ip_address.eq::<Option<IpNet>>(None),
            user_agent.eq(Some(format!("seed-runner/{seed_index:02}"))),
            request_id.eq(Some(request_uuid)),
            changes.eq(serde_json::json!({})),
            metadata.eq(serde_json::json!({ "seed": true, "index": seed_index })),
            created_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

fn print_counts(conn: &mut PgConnection) -> SeedResult<()> {
    use schema::audit_logs::table as audit_logs_table;
    use schema::profiles::table as profiles_table;
    use schema::refresh_tokens::table as refresh_tokens_table;
    use schema::roles::table as roles_table;
    use schema::users::table as users_table;
    use schema::users_roles::table as users_roles_table;

    let users_count: i64 = users_table.select(count_star()).first(conn)?;
    let roles_count: i64 = roles_table.select(count_star()).first(conn)?;
    let user_roles_count: i64 = users_roles_table.select(count_star()).first(conn)?;
    let profiles_count: i64 = profiles_table.select(count_star()).first(conn)?;
    let refresh_tokens_count: i64 = refresh_tokens_table.select(count_star()).first(conn)?;
    let audit_logs_count: i64 = audit_logs_table.select(count_star()).first(conn)?;

    println!("Record counts by table:");
    println!("users: {users_count}");
    println!("roles: {roles_count}");
    println!("users_roles: {user_roles_count}");
    println!("profiles: {profiles_count}");
    println!("refresh_tokens: {refresh_tokens_count}");
    println!("audit_logs: {audit_logs_count}");

    Ok(())
}

/// Prints the role names each seed user really holds, read back from the link
/// table. The old output printed a fixed `role=user`, which was false for every
/// subject the rotation had given another role.
fn print_seed_subjects(conn: &mut PgConnection, security: &SecurityService) -> SeedResult<()> {
    let mut blind_indexes: Vec<Vec<u8>> = Vec::with_capacity(100);
    for index in 1..=100 {
        blind_indexes.push(
            security
                .email_blind_index(&test_user_email(index))
                .map_err(|e| format!("failed to index the seed email: {e}"))?,
        );
    }

    let rows: Vec<(Vec<u8>, String)> = schema::users::table
        .inner_join(
            schema::users_roles::table.on(schema::users_roles::user_id.eq(schema::users::id)),
        )
        .inner_join(schema::roles::table.on(schema::roles::id.eq(schema::users_roles::role_id)))
        .select((schema::users::email_blind_index, schema::roles::name))
        .filter(schema::users::email_blind_index.eq_any(&blind_indexes))
        .load(conn)?;

    let mut roles_by_index: HashMap<usize, Vec<String>> = HashMap::new();
    for (blind_index, role_name) in rows {
        if let Some(index) = (1..=100).find(|index| {
            security
                .email_blind_index(&test_user_email(*index))
                .is_ok_and(|value| value == blind_index)
        }) {
            roles_by_index.entry(index).or_default().push(role_name);
        }
    }

    println!("\nTest users (password123):");
    for index in 1..=100 {
        let roles = roles_by_index
            .get(&index)
            .cloned()
            .unwrap_or_else(|| vec!["none".to_string()]);
        println!("- {} | role={}", test_user_email(index), roles.join(", "));
    }

    Ok(())
}

fn main() -> SeedResult<()> {
    dotenvy::dotenv().ok();

    // Check if user wants to send test emails (exit early)
    if env::args().any(|arg| arg == "--send-test-emails") {
        // A runtime that fails to build is an error of the seed, not a panic.
        return tokio::runtime::Runtime::new()
            .map_err(|error| format!("failed to build the tokio runtime: {error}"))?
            .block_on(send_test_emails());
    }

    let pool = pool()?;
    let mut conn = pool.get()?;
    let security = build_security_service()?;

    conn.transaction(|conn| {
        // Canonical RBAC catalog: roles, permissions, and allow links. The
        // seed reuses it and never invents a role of its own, so the catalog
        // the admin screens edit is the one the subjects carry.
        let canonical_roles = rbac_catalog_service::ensure_canonical_catalog(conn)?;

        for index in 1..=100 {
            let user_data = generate_user_data(index);
            let user_id = ensure_user(conn, &security, index)?;
            let _profile_id = ensure_profile(conn, user_id, index, &user_data)?;
            let role_id = canonical_roles[role_for(index)];
            ensure_user_roles(conn, user_id, &[role_id])?;
            let _refresh_token_id = ensure_refresh_token(conn, user_id, index)?;
            let _audit_log_id = ensure_audit_log(conn, user_id, index)?;
        }

        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    })?;

    println!("Seed completed successfully");
    println!("Admin email: admin@example.com");
    println!("Default password: password123");
    print_seed_subjects(&mut conn, &security)?;
    print_counts(&mut conn)?;

    Ok(())
}
