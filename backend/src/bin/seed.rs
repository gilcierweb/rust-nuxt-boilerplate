use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString},
};
use base64::Engine;
use chrono::Utc;
use diesel::dsl::count_star;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::sql_types::{Text, Uuid as SqlUuid};
use faker_rust::{name};
use ipnet::IpNet;
use rand::RngCore;
use std::collections::HashMap;
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

#[derive(QueryableByName)]
struct SqlUuidRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
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

fn protect_email(email: &str) -> SeedResult<(Vec<u8>, Vec<u8>, i32)> {
    use aes_gcm::aead::{Aead, KeyInit};
    use hmac::Mac;
    use sha2::Digest;
    let normalized = email
        .trim()
        .to_lowercase()
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>();

    let blind_index_key_b64 = std::env::var("BLIND_INDEX_KEY")?;
    let blind_index_key = base64::engine::general_purpose::STANDARD.decode(blind_index_key_b64)?;
    let blind = {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(&blind_index_key)
            .map_err(|e| format!("invalid blind index key: {e}"))?;
        mac.update(normalized.as_bytes());
        mac.finalize().into_bytes().to_vec()
    };

    let version = std::env::var("CURRENT_ENCRYPTION_KEY_VERSION")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(1);

    let master_key_b64 = std::env::var("MASTER_KEY")?;
    let master_key = base64::engine::general_purpose::STANDARD.decode(master_key_b64)?;
    let key = sha2::Sha256::digest(&master_key);
    let mut mac = <hmac::Hmac<sha2::Sha256> as hmac::Mac>::new_from_slice(&key)
        .map_err(|e| format!("invalid encryption key material: {e}"))?;
    mac.update(format!("encryption:v{}", version).as_bytes());
    mac.update(&version.to_le_bytes());
    let result = mac.finalize();
    let mut encryption_key = [0u8; 32];
    encryption_key.copy_from_slice(&result.into_bytes()[..32]);

    let cipher = aes_gcm::Aes256Gcm::new_from_slice(&encryption_key)
        .map_err(|_| "invalid encryption key length".to_string())?;
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = aes_gcm::Nonce::from(nonce_bytes);
    let ciphertext = cipher
        .encrypt(&nonce, normalized.as_bytes())
        .map_err(|_| "failed to encrypt seed data".to_string())?;
    let encrypted = [nonce_bytes.to_vec(), ciphertext].concat();

    Ok((blind, encrypted, version as i32))
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

fn ensure_role(conn: &mut PgConnection, role_name: &str) -> SeedResult<Uuid> {
    use schema::roles::dsl::*;

    if let Some(existing_id) = roles
        .filter(name.eq(role_name))
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    Ok(diesel::insert_into(roles)
        .values((
            id.eq(Uuid::new_v4()),
            name.eq(role_name),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

/// Find a role by name, returning a descriptive error if not found.
/// This should only be called after ensure_role has been run for the role.
fn find_role_id(role_ids: &[(String, Uuid)], role_name: &str) -> SeedResult<Uuid> {
    role_ids
        .iter()
        .find(|(name, _)| name == role_name)
        .map(|(_, id)| *id)
        .ok_or_else(|| {
            format!("Role '{}' not found after creation - this should not happen", role_name).into()
        })
}

fn ensure_permission(
    conn: &mut PgConnection,
    permission_code: &str,
    permission_description: Option<&str>,
) -> SeedResult<Uuid> {
    if let Some(existing) = diesel::sql_query("SELECT id FROM permissions WHERE code = $1 LIMIT 1")
        .bind::<Text, _>(permission_code)
        .get_result::<SqlUuidRow>(conn)
        .optional()?
    {
        return Ok(existing.id);
    }

    let created = diesel::sql_query(
        "INSERT INTO permissions (id, code, description, created_at, updated_at)
         VALUES ($1, $2, $3, NOW(), NOW())
         RETURNING id",
    )
    .bind::<SqlUuid, _>(Uuid::new_v4())
    .bind::<Text, _>(permission_code)
    .bind::<diesel::sql_types::Nullable<Text>, _>(permission_description)
    .get_result::<SqlUuidRow>(conn)?;

    Ok(created.id)
}

fn ensure_role_permission(
    conn: &mut PgConnection,
    role_uuid: Uuid,
    permission_uuid: Uuid,
) -> SeedResult<()> {
    diesel::sql_query(
        "INSERT INTO roles_permissions (role_id, permission_id, created_at)
         VALUES ($1, $2, NOW())
         ON CONFLICT (role_id, permission_id) DO NOTHING",
    )
    .bind::<SqlUuid, _>(role_uuid)
    .bind::<SqlUuid, _>(permission_uuid)
    .execute(conn)?;

    Ok(())
}

fn ensure_user(conn: &mut PgConnection, user_index: usize) -> SeedResult<Uuid> {
    use schema::users::dsl::*;

    let user_email = test_user_email(user_index);
    let user_password = "password123";
    let (email_bidx, email_enc, key_version) = protect_email(&user_email)?;
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

fn ensure_user_role(conn: &mut PgConnection, user_uuid: Uuid, role_uuid: Uuid) -> SeedResult<()> {
    use schema::users_roles::dsl::*;

    let exists = users_roles
        .filter(user_id.eq(user_uuid).and(role_id.eq(role_uuid)))
        .select(user_id)
        .first::<Uuid>(conn)
        .optional()?
        .is_some();

    if !exists {
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
    use schema::{audit_logs::table as audit_logs_table, profiles::table as profiles_table, refresh_tokens::table as refresh_tokens_table, roles::table as roles_table, users::table as users_table, users_roles::table as users_roles_table};

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

fn print_test_users() {
    println!("\nTest users:");
    for index in 1..=10 {
        let role = if index == 1 { "admin" } else { "user" };
        println!(
            "- {} | password123 | role={}",
            test_user_email(index),
            role
        );
    }
}

fn main() -> SeedResult<()> {
    dotenvy::dotenv().ok();
    let pool = pool()?;
    let mut conn = pool.get()?;

    conn.transaction(|conn| {
        let role_names = [
            "admin",
            "manager",
            "editor",
            "viewer",
            "support",
            "finance",
        ];
        let mut role_ids = Vec::with_capacity(role_names.len());
        for role_name in role_names {
            role_ids.push((role_name.to_string(), ensure_role(conn, role_name)?));
        }

        let permission_codes = [
            "users:read",
            "users:create",
            "users:update",
            "users:delete",
            "roles:read",
            "roles:create",
            "roles:update",
            "roles:delete",
            "audit_logs:read",
            "audit_logs:create",
            "profiles:read",
            "profiles:update",
        ];

        let mut permission_ids = HashMap::new();
        for code in permission_codes {
            let id = ensure_permission(conn, code, None)?;
            permission_ids.insert(code.to_string(), id);
        }

        let admin_role_id = find_role_id(&role_ids, "admin")?;

        for permission_id in permission_ids.values() {
            ensure_role_permission(conn, admin_role_id, *permission_id)?;
        }

        let manager_role_id = find_role_id(&role_ids, "manager")?;

        let manager_permissions = [
            "users:read",
            "users:create",
            "users:update",
            "roles:read",
            "audit_logs:read",
            "profiles:read",
            "profiles:update",
        ];

        for permission_code in manager_permissions {
            if let Some(permission_id) = permission_ids.get(permission_code) {
                ensure_role_permission(conn, manager_role_id, *permission_id)?;
            }
        }

        let viewer_role_id = find_role_id(&role_ids, "viewer")?;

        let viewer_permissions = ["users:read", "roles:read", "audit_logs:read", "profiles:read"];

        for permission_code in viewer_permissions {
            if let Some(permission_id) = permission_ids.get(permission_code) {
                ensure_role_permission(conn, viewer_role_id, *permission_id)?;
            }
        }

        for index in 1..=10 {
            let user_data = generate_user_data(index);
            let user_id = ensure_user(conn, index)?;
            let _profile_id = ensure_profile(conn, user_id, index, &user_data)?;
            let role_id = if index == 1 {
                admin_role_id
            } else {
                role_ids[(index - 1) % role_ids.len()].1
            };
            ensure_user_role(conn, user_id, role_id)?;
            let _refresh_token_id = ensure_refresh_token(conn, user_id, index)?;
            let _audit_log_id = ensure_audit_log(conn, user_id, index)?;
        }

        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    })?;

    println!("Seed completed successfully");
    println!("Admin email: admin@example.com");
    println!("Default password: password123");
    print_test_users();
    print_counts(&mut conn)?;

    Ok(())
}