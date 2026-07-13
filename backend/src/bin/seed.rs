use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
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
use faker_rust::{address, commerce, company, job, name};
use hmac::{Mac, SimpleHmac};
use rand::RngCore;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;

#[path = "../utils/email.rs"]
mod email_utils;
#[path = "../utils/money.rs"]
mod money_utils;
#[path = "../utils/sanitize.rs"]
mod sanitize_utils;
#[path = "../db/schema.rs"]
mod schema;
type PgPool = r2d2::Pool<ConnectionManager<PgConnection>>;
type SeedResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

use money_utils::{Currency, Money};

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
    let stripped = sanitize_utils::strip_html(input);
    let sanitized = sanitize_utils::sanitize_input(&stripped);
    let mut bounded = if sanitize_utils::contains_dangerous_patterns(&sanitized) {
        fallback.to_string()
    } else {
        sanitized
    };

    if !sanitize_utils::validate_length(&bounded, 1, max_chars) {
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
        "admin@app-name.local".to_string()
    } else {
        format!("user{user_index:02}@app-name.local")
    }
}

fn test_company_subdomain(company_index: usize) -> String {
    if company_index == 1 {
        "app-name".to_string()
    } else {
        format!("app-name-{company_index:02}")
    }
}

fn test_company_host(company_index: usize) -> String {
    format!("{}.local", test_company_subdomain(company_index))
}

fn seed_money(seed_index: usize) -> Money {
    Money::new(100_000_i64 + (seed_index as i64 * 375_i64), Currency::BRL)
}

fn generate_user_data(user_index: usize) -> SeedUserData {
    if user_index == 1 {
        return SeedUserData {
            first_name: "Admin".to_string(),
            last_name: "name".to_string(),
            full_name: "Admin name".to_string(),
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

fn generate_company_name(company_index: usize) -> String {
    if company_index == 1 {
        "name Tecnologia Ltda".to_string()
    } else {
        sanitize_seed_text(
            &format!("{} {}", company::name(), company_index),
            "Seed Company",
            255,
        )
    }
}

fn generate_trade_name(company_index: usize) -> String {
    if company_index == 1 {
        "name".to_string()
    } else {
        sanitize_seed_text(
            &format!("{} {}", company::department(), company_index),
            "Operations",
            255,
        )
    }
}

fn generate_document_title(seed_index: usize) -> String {
    sanitize_seed_text(
        &format!("{} Document {seed_index:02}", commerce::department()),
        "Seed Document",
        255,
    )
}

fn generate_document_description(seed_index: usize) -> String {
    sanitize_seed_text(
        &(format!("{} record for {}", job::title(), address::city())
            + &format!(" #{seed_index:02}")),
        "Seed document record",
        500,
    )
}

fn generate_debt_title(seed_index: usize) -> String {
    sanitize_seed_text(
        &format!("{} Charge {seed_index:02}", company::industry()),
        "Service Charge",
        255,
    )
}

fn generate_debt_description(seed_index: usize) -> String {
    sanitize_seed_text(
        &format!(
            "{} service generated for {} #{seed_index:02}",
            job::title(),
            company::name()
        ),
        "Seed debt description",
        500,
    )
}

fn generate_invoice_description(seed_index: usize) -> String {
    sanitize_seed_text(
        &format!(
            "{} for {} #{seed_index:02}",
            commerce::product_name(),
            company::name()
        ),
        "Seed invoice request",
        500,
    )
}

fn pool() -> SeedResult<PgPool> {
    let database_url = std::env::var("DATABASE_URL")?;
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Ok(r2d2::Pool::builder().max_size(4).build(manager)?)
}

fn blind_index(value: &str, key: &[u8]) -> SeedResult<Vec<u8>> {
    let mut mac = <hmac::Hmac<Sha256> as Mac>::new_from_slice(key)
        .map_err(|e| format!("invalid blind index key: {e}"))?;
    mac.update(value.as_bytes());
    Ok(mac.finalize().into_bytes().to_vec())
}

fn derive_encryption_key(master_key_b64: &str, version: u32) -> SeedResult<[u8; 32]> {
    let master_key = base64::engine::general_purpose::STANDARD.decode(master_key_b64)?;
    let key = Sha256::digest(&master_key);
    let mut mac = <SimpleHmac<Sha256> as Mac>::new_from_slice(&key)
        .map_err(|e| format!("invalid encryption key material: {e}"))?;
    mac.update(format!("encryption:v{}", version).as_bytes());
    mac.update(&version.to_le_bytes());
    let result = mac.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result.into_bytes()[..32]);
    Ok(out)
}

fn encrypt(data: &[u8], key: &[u8]) -> SeedResult<Vec<u8>> {
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|_| "invalid encryption key length".to_string())?;
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from(nonce_bytes);
    let ciphertext = cipher
        .encrypt(&nonce, data)
        .map_err(|_| "failed to encrypt seed data".to_string())?;
    Ok([nonce_bytes.to_vec(), ciphertext].concat())
}

fn protect_email(email: &str) -> SeedResult<(Vec<u8>, Vec<u8>, i32)> {
    let normalized = email_utils::normalize_email(email)
        .ok_or_else(|| format!("invalid seed email: {email}"))?;
    let blind_index_key_b64 = std::env::var("BLIND_INDEX_KEY")?;
    let blind_index_key = base64::engine::general_purpose::STANDARD.decode(blind_index_key_b64)?;
    let blind = blind_index(&normalized, &blind_index_key)?;
    let version = std::env::var("CURRENT_ENCRYPTION_KEY_VERSION")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(1);
    let encryption_key = derive_encryption_key(&std::env::var("MASTER_KEY")?, version)?;
    let encrypted = encrypt(normalized.as_bytes(), &encryption_key)?;
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

fn ensure_company(conn: &mut PgConnection, company_index: usize) -> SeedResult<Uuid> {
    use schema::companies::dsl::*;

    let company_slug = if company_index == 1 {
        "name".to_string()
    } else {
        format!("name-{company_index:02}")
    };
    let company_subdomain = if company_index == 1 {
        "app-name".to_string()
    } else {
        format!("app-name-{company_index:02}")
    };
    let company_name = generate_company_name(company_index);
    let company_trade_name = Some(generate_trade_name(company_index));

    if let Some(existing_id) = companies
        .filter(slug.eq(&company_slug))
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    Ok(diesel::insert_into(companies)
        .values((
            id.eq(Uuid::new_v4()),
            slug.eq(company_slug),
            subdomain.eq(company_subdomain),
            legal_name.eq(company_name),
            trade_name.eq(company_trade_name),
            status.eq(1_i16),
            settings.eq(json!({})),
            encryption_key_version.eq(1_i32),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

fn ensure_company_domain(
    conn: &mut PgConnection,
    company_uuid: Uuid,
    company_index: usize,
) -> SeedResult<Uuid> {
    use schema::company_domains::dsl::*;

    let host_value = test_company_host(company_index);

    if let Some(existing_id) = company_domains
        .filter(host.eq(&host_value))
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    Ok(diesel::insert_into(company_domains)
        .values((
            id.eq(Uuid::new_v4()),
            company_id.eq(company_uuid),
            host.eq(host_value),
            domain_type.eq(1_i16),
            is_primary.eq(true),
            verified_at.eq(Some(Utc::now())),
            disabled_at.eq::<Option<chrono::DateTime<Utc>>>(None),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

fn ensure_company_settings(conn: &mut PgConnection, company_uuid: Uuid) -> SeedResult<Uuid> {
    use schema::company_settings::dsl::*;

    if let Some(existing_company_id) = company_settings
        .filter(company_id.eq(company_uuid))
        .select(company_id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_company_id);
    }

    Ok(diesel::insert_into(company_settings)
        .values((
            company_id.eq(company_uuid),
            timezone.eq("America/Fortaleza"),
            locale.eq("pt-BR"),
            currency_code.eq("BRL"),
            invoice_provider.eq::<Option<i16>>(None),
            payment_provider.eq::<Option<i16>>(None),
            whatsapp_provider.eq::<Option<i16>>(None),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(company_id)
        .get_result(conn)?)
}

fn ensure_user(conn: &mut PgConnection, user_index: usize) -> SeedResult<Uuid> {
    use schema::users::dsl::*;

    let user_email = test_user_email(user_index);
    let user_password = "password123";
    let (email_bidx, email_enc, key_version) = protect_email(&user_email)?;

    if let Some(existing_id) = users
        .filter(email_blind_index.eq(email_bidx.clone()))
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    let pwd_hash = password_hash(user_password)?;

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
            bio.eq(Some(sanitize_seed_text(
                &format!("{} based in {}", job::title(), address::city()),
                "Seed user bio",
                500,
            ))),
            avatar.eq(Some(format!(
                "https://api.dicebear.com/9.x/initials/svg?seed={}",
                user_data.full_name.replace(' ', "%20")
            ))),
            status.eq(true),
            social_network.eq(json!({})),
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

fn ensure_customer(
    conn: &mut PgConnection,
    company_uuid: Uuid,
    user_index: usize,
) -> SeedResult<Uuid> {
    use schema::customers::dsl::*;
    let customer_code_value = format!("CUS-{user_index:04}");

    if let Some(existing_id) = customers
        .filter(
            company_id
                .eq(company_uuid)
                .and(customer_code.eq(Some(customer_code_value.clone()))),
        )
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    Ok(diesel::insert_into(customers)
        .values((
            id.eq(Uuid::new_v4()),
            company_id.eq(company_uuid),
            customer_code.eq(Some(customer_code_value)),
            status.eq(1_i16),
            origin.eq(1_i16),
            activated_at.eq(Some(Utc::now())),
            encryption_key_version.eq(1_i32),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

fn ensure_debt_category(
    conn: &mut PgConnection,
    company_uuid: Uuid,
    category_code: &str,
    category_name: &str,
    order: i32,
) -> SeedResult<Uuid> {
    use schema::debt_categories::dsl::*;

    if let Some(existing_id) = debt_categories
        .filter(company_id.eq(company_uuid).and(code.eq(category_code)))
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    Ok(diesel::insert_into(debt_categories)
        .values((
            id.eq(Uuid::new_v4()),
            company_id.eq(company_uuid),
            code.eq(category_code),
            name.eq(category_name),
            status.eq(1_i16),
            sort_order.eq(order),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

fn ensure_customer_user(
    conn: &mut PgConnection,
    customer_uuid: Uuid,
    user_uuid: Uuid,
) -> SeedResult<Uuid> {
    use schema::customer_users::dsl::*;

    if let Some(existing_id) = customer_users
        .filter(customer_id.eq(customer_uuid).and(user_id.eq(user_uuid)))
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    Ok(diesel::insert_into(customer_users)
        .values((
            id.eq(Uuid::new_v4()),
            customer_id.eq(customer_uuid),
            user_id.eq(user_uuid),
            portal_role.eq(1_i16),
            access_status.eq(1_i16),
            is_primary_contact.eq(true),
            invited_at.eq(Some(Utc::now())),
            accepted_at.eq(Some(Utc::now())),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

fn ensure_customer_contact(
    conn: &mut PgConnection,
    customer_uuid: Uuid,
    user_data: &SeedUserData,
    seed_index: usize,
) -> SeedResult<Uuid> {
    use schema::customer_contacts::dsl::*;

    let contact_name_value = user_data.full_name.clone();

    if let Some(existing_id) = customer_contacts
        .filter(
            customer_id
                .eq(customer_uuid)
                .and(contact_name.eq(&contact_name_value)),
        )
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    let contact_email = test_user_email(seed_index);
    let (email_bidx, email_enc, _) = protect_email(&contact_email)?;

    Ok(diesel::insert_into(customer_contacts)
        .values((
            id.eq(Uuid::new_v4()),
            customer_id.eq(customer_uuid),
            contact_name.eq(contact_name_value),
            department.eq(Some("Finance".to_string())),
            email_encrypted.eq(Some(email_enc)),
            email_blind_index.eq(Some(email_bidx)),
            phone_encrypted.eq::<Option<Vec<u8>>>(None),
            phone_blind_index.eq::<Option<Vec<u8>>>(None),
            is_primary.eq(true),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

fn ensure_customer_address(
    conn: &mut PgConnection,
    customer_uuid: Uuid,
    seed_index: usize,
) -> SeedResult<Uuid> {
    use schema::customer_addresses::dsl::*;

    let street_value = format!(
        "{} Avenue",
        sanitize_seed_text(&address::street_name(), "Central", 100)
    );

    if let Some(existing_id) = customer_addresses
        .filter(customer_id.eq(customer_uuid).and(street.eq(&street_value)))
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    Ok(diesel::insert_into(customer_addresses)
        .values((
            id.eq(Uuid::new_v4()),
            customer_id.eq(customer_uuid),
            address_type.eq(1_i16),
            street.eq(street_value),
            number.eq(Some(format!("{}", 100 + seed_index))),
            complement.eq::<Option<String>>(None),
            district.eq(Some("Centro".to_string())),
            city.eq(sanitize_seed_text(&address::city(), "Fortaleza", 100)),
            state.eq("CE"),
            postal_code.eq(Some(format!("60000-{:03}", seed_index))),
            country_code.eq("BR"),
            is_primary.eq(true),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

fn ensure_storage_object(
    conn: &mut PgConnection,
    company_uuid: Uuid,
    user_uuid: Uuid,
    seed_index: usize,
) -> SeedResult<Uuid> {
    use schema::storage_objects::dsl::*;

    let object_key_value =
        format!("normalized/company-{seed_index:02}/document-{seed_index:02}.pdf");

    if let Some(existing_id) = storage_objects
        .filter(
            bucket_name
                .eq("name-private")
                .and(object_key.eq(&object_key_value)),
        )
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    Ok(diesel::insert_into(storage_objects)
        .values((
            id.eq(Uuid::new_v4()),
            company_id.eq(company_uuid),
            storage_provider.eq(1_i16),
            bucket_name.eq("name-private"),
            object_key.eq(object_key_value),
            original_file_name.eq(format!("seed-document-{seed_index:02}.pdf")),
            mime_type.eq("application/pdf"),
            size_bytes.eq(2048_i64 + seed_index as i64),
            checksum_sha256.eq(Some(format!("{:064x}", seed_index))),
            visibility.eq(1_i16),
            status.eq(1_i16),
            uploaded_by_user_id.eq(Some(user_uuid)),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

fn ensure_document(
    conn: &mut PgConnection,
    company_uuid: Uuid,
    customer_uuid: Uuid,
    user_uuid: Uuid,
    seed_index: usize,
) -> SeedResult<Uuid> {
    use schema::documents::dsl::*;

    let external_ref = format!("seed-doc-{seed_index:02}");

    if let Some(existing_id) = documents
        .filter(external_reference.eq(Some(external_ref.clone())))
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    Ok(diesel::insert_into(documents)
        .values((
            id.eq(Uuid::new_v4()),
            company_id.eq(company_uuid),
            customer_id.eq(customer_uuid),
            uploaded_by_user_id.eq(Some(user_uuid)),
            document_type.eq(1_i16),
            status.eq(1_i16),
            title.eq(generate_document_title(seed_index)),
            description.eq(Some(generate_document_description(seed_index))),
            reference_date.eq(Some(
                (Utc::now() - chrono::Duration::days(seed_index as i64)).date_naive(),
            )),
            is_visible_to_customer.eq(true),
            external_reference.eq(Some(external_ref)),
            metadata.eq(json!({ "seed": true, "index": seed_index })),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

fn ensure_document_file(
    conn: &mut PgConnection,
    document_uuid: Uuid,
    storage_object_uuid: Uuid,
) -> SeedResult<Uuid> {
    use schema::document_files::dsl::*;

    if let Some(existing_id) = document_files
        .filter(
            document_id
                .eq(document_uuid)
                .and(storage_object_id.eq(storage_object_uuid)),
        )
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    Ok(diesel::insert_into(document_files)
        .values((
            id.eq(Uuid::new_v4()),
            document_id.eq(document_uuid),
            storage_object_id.eq(storage_object_uuid),
            file_role.eq(1_i16),
            sort_order.eq(0_i32),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

fn ensure_debt(
    conn: &mut PgConnection,
    company_uuid: Uuid,
    customer_uuid: Uuid,
    user_uuid: Uuid,
    debt_category_uuid: Uuid,
    seed_index: usize,
) -> SeedResult<Uuid> {
    use schema::debts::dsl::*;

    let external_ref = format!("seed-debt-{seed_index:02}");

    if let Some(existing_id) = debts
        .filter(
            company_id
                .eq(company_uuid)
                .and(external_reference.eq(Some(external_ref.clone()))),
        )
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    Ok(diesel::insert_into(debts)
        .values((
            id.eq(Uuid::new_v4()),
            company_id.eq(company_uuid),
            customer_id.eq(customer_uuid),
            created_by_user_id.eq(Some(user_uuid)),
            debt_category_id.eq(debt_category_uuid),
            status.eq(1_i16),
            title.eq(generate_debt_title(seed_index)),
            description.eq(Some(generate_debt_description(seed_index))),
            competence_date.eq(Some((Utc::now() - chrono::Duration::days(30)).date_naive())),
            due_date.eq((Utc::now() + chrono::Duration::days(seed_index as i64)).date_naive()),
            amount.eq(seed_money(seed_index).to_decimal()),
            currency_code.eq("BRL"),
            external_reference.eq(Some(external_ref)),
            metadata.eq(json!({ "seed": true, "index": seed_index })),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

fn ensure_payment_transaction(
    conn: &mut PgConnection,
    company_uuid: Uuid,
    customer_uuid: Uuid,
    user_uuid: Uuid,
    seed_index: usize,
) -> SeedResult<Uuid> {
    use schema::payment_transactions::dsl::*;

    let provider_ref = format!("seed-payment-transaction-{seed_index:02}");
    let amount = seed_money(seed_index).to_decimal();

    if let Some(existing_id) = payment_transactions
        .filter(
            company_id
                .eq(company_uuid)
                .and(provider_reference.eq(Some(provider_ref.clone()))),
        )
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    Ok(diesel::insert_into(payment_transactions)
        .values((
            id.eq(Uuid::new_v4()),
            company_id.eq(company_uuid),
            customer_id.eq(customer_uuid),
            received_by_user_id.eq(Some(user_uuid)),
            provider.eq(Some(1_i16)),
            payment_method.eq(1_i16),
            status.eq(1_i16),
            gross_amount.eq(amount.clone()),
            net_amount.eq(Some(amount)),
            provider_reference.eq(Some(provider_ref)),
            paid_at.eq(Some(Utc::now())),
            metadata.eq(json!({ "seed": true, "normalized": true, "index": seed_index })),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

fn ensure_payment_allocation(
    conn: &mut PgConnection,
    payment_transaction_uuid: Uuid,
    debt_uuid: Uuid,
    seed_index: usize,
) -> SeedResult<()> {
    use schema::payment_allocations::dsl::*;

    let exists = payment_allocations
        .filter(
            payment_transaction_id
                .eq(payment_transaction_uuid)
                .and(debt_id.eq(debt_uuid)),
        )
        .select(payment_transaction_id)
        .first::<Uuid>(conn)
        .optional()?
        .is_some();

    if !exists {
        diesel::insert_into(payment_allocations)
            .values((
                payment_transaction_id.eq(payment_transaction_uuid),
                debt_id.eq(debt_uuid),
                allocated_amount.eq(seed_money(seed_index).to_decimal()),
                created_at.eq(Utc::now()),
            ))
            .execute(conn)?;
    }

    Ok(())
}

fn ensure_invoice_request(
    conn: &mut PgConnection,
    company_uuid: Uuid,
    customer_uuid: Uuid,
    user_uuid: Uuid,
    seed_index: usize,
) -> SeedResult<Uuid> {
    use schema::invoice_requests::dsl::*;

    let provider_ref = format!("seed-invoice-request-{seed_index:02}");

    if let Some(existing_id) = invoice_requests
        .filter(
            company_id
                .eq(company_uuid)
                .and(fiscal_provider.eq(Some(1_i16)))
                .and(fiscal_provider_reference.eq(Some(provider_ref.clone()))),
        )
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    Ok(diesel::insert_into(invoice_requests)
        .values((
            id.eq(Uuid::new_v4()),
            company_id.eq(company_uuid),
            customer_id.eq(customer_uuid),
            requested_by_user_id.eq(Some(user_uuid)),
            status.eq(1_i16),
            fiscal_provider.eq(Some(1_i16)),
            service_description.eq(generate_invoice_description(seed_index)),
            service_amount.eq(seed_money(seed_index).to_decimal()),
            service_date.eq(Some(Utc::now().date_naive())),
            fiscal_provider_reference.eq(Some(provider_ref)),
            notes_encrypted.eq::<Option<Vec<u8>>>(None),
            metadata.eq(json!({ "seed": true, "index": seed_index })),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

fn ensure_issued_invoice(
    conn: &mut PgConnection,
    company_uuid: Uuid,
    customer_uuid: Uuid,
    invoice_request_uuid: Uuid,
    seed_index: usize,
) -> SeedResult<Uuid> {
    use schema::issued_invoices::dsl::*;

    let provider_ref = format!("seed-issued-invoice-{seed_index:02}");
    let amount = seed_money(seed_index).to_decimal();

    if let Some(existing_id) = issued_invoices
        .filter(
            company_id
                .eq(company_uuid)
                .and(provider_reference.eq(Some(provider_ref.clone()))),
        )
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    Ok(diesel::insert_into(issued_invoices)
        .values((
            id.eq(Uuid::new_v4()),
            company_id.eq(company_uuid),
            customer_id.eq(customer_uuid),
            invoice_request_id.eq(Some(invoice_request_uuid)),
            provider.eq(1_i16),
            provider_reference.eq(Some(provider_ref)),
            invoice_number.eq(Some(format!("NF-{:04}", seed_index))),
            series.eq(Some("A".to_string())),
            issued_at.eq(Some(Utc::now())),
            status.eq(1_i16),
            total_amount.eq(amount),
            created_at.eq(Utc::now()),
            updated_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

fn ensure_audit_log(
    conn: &mut PgConnection,
    company_uuid: Uuid,
    customer_uuid: Uuid,
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
            company_id.eq(Some(company_uuid)),
            actor_user_id.eq(Some(user_uuid)),
            actor_role_snapshot.eq(Some(if seed_index == 1 {
                "admin".to_string()
            } else {
                "subscriber".to_string()
            })),
            action.eq("seed.create"),
            resource_type.eq("customer"),
            resource_id.eq(Some(customer_uuid)),
            target_customer_id.eq(Some(customer_uuid)),
            ip_address.eq::<Option<ipnet::IpNet>>(None),
            user_agent.eq(Some(format!("seed-runner/{seed_index:02}"))),
            request_id.eq(Some(request_uuid)),
            changes.eq(json!({})),
            metadata.eq(json!({ "seed": true, "index": seed_index })),
            created_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

fn ensure_outbox_event(
    conn: &mut PgConnection,
    company_uuid: Uuid,
    aggregate_id_value: Uuid,
    seed_index: usize,
) -> SeedResult<Uuid> {
    use schema::outbox_events::dsl::*;

    let event_type_value = format!("invoice.issued.{}", seed_index);

    if let Some(existing_id) = outbox_events
        .filter(event_type.eq(&event_type_value))
        .select(id)
        .first::<Uuid>(conn)
        .optional()?
    {
        return Ok(existing_id);
    }

    Ok(diesel::insert_into(outbox_events)
        .values((
            id.eq(Uuid::new_v4()),
            company_id.eq(Some(company_uuid)),
            aggregate_type.eq("issued_invoice"),
            aggregate_id.eq(aggregate_id_value),
            event_type.eq(event_type_value),
            payload.eq(json!({ "seed": true, "index": seed_index })),
            status.eq(1_i16),
            available_at.eq(Utc::now()),
            processed_at.eq::<Option<chrono::DateTime<Utc>>>(None),
            created_at.eq(Utc::now()),
        ))
        .returning(id)
        .get_result(conn)?)
}

fn print_counts(conn: &mut PgConnection) -> SeedResult<()> {
    let companies_count: i64 = schema::companies::table.select(count_star()).first(conn)?;
    let roles_count: i64 = schema::roles::table.select(count_star()).first(conn)?;
    let users_count: i64 = schema::users::table.select(count_star()).first(conn)?;
    let user_roles_count: i64 = schema::users_roles::table
        .select(count_star())
        .first(conn)?;
    let profiles_count: i64 = schema::profiles::table.select(count_star()).first(conn)?;
    let customers_count: i64 = schema::customers::table.select(count_star()).first(conn)?;
    let customer_users_count: i64 = schema::customer_users::table
        .select(count_star())
        .first(conn)?;
    let customer_contacts_count: i64 = schema::customer_contacts::table
        .select(count_star())
        .first(conn)?;
    let customer_addresses_count: i64 = schema::customer_addresses::table
        .select(count_star())
        .first(conn)?;
    let company_domains_count: i64 = schema::company_domains::table
        .select(count_star())
        .first(conn)?;
    let company_settings_count: i64 = schema::company_settings::table
        .select(count_star())
        .first(conn)?;
    let debt_categories_count: i64 = schema::debt_categories::table
        .select(count_star())
        .first(conn)?;
    let debts_count: i64 = schema::debts::table.select(count_star()).first(conn)?;
    let storage_objects_count: i64 = schema::storage_objects::table
        .select(count_star())
        .first(conn)?;
    let documents_count: i64 = schema::documents::table.select(count_star()).first(conn)?;
    let document_files_count: i64 = schema::document_files::table
        .select(count_star())
        .first(conn)?;
    let payment_transactions_count: i64 = schema::payment_transactions::table
        .select(count_star())
        .first(conn)?;
    let payment_allocations_count: i64 = schema::payment_allocations::table
        .select(count_star())
        .first(conn)?;
    let invoice_requests_count: i64 = schema::invoice_requests::table
        .select(count_star())
        .first(conn)?;
    let issued_invoices_count: i64 = schema::issued_invoices::table
        .select(count_star())
        .first(conn)?;
    let refresh_tokens_count: i64 = schema::refresh_tokens::table
        .select(count_star())
        .first(conn)?;
    let audit_logs_count: i64 = schema::audit_logs::table.select(count_star()).first(conn)?;
    let outbox_events_count: i64 = schema::outbox_events::table
        .select(count_star())
        .first(conn)?;

    println!("Record counts by table:");
    println!("companies: {companies_count}");
    println!("roles: {roles_count}");
    println!("users: {users_count}");
    println!("users_roles: {user_roles_count}");
    println!("profiles: {profiles_count}");
    println!("customers: {customers_count}");
    println!("customer_users: {customer_users_count}");
    println!("customer_contacts: {customer_contacts_count}");
    println!("customer_addresses: {customer_addresses_count}");
    println!("company_domains: {company_domains_count}");
    println!("company_settings: {company_settings_count}");
    println!("debt_categories: {debt_categories_count}");
    println!("debts: {debts_count}");
    println!("storage_objects: {storage_objects_count}");
    println!("documents: {documents_count}");
    println!("document_files: {document_files_count}");
    println!("payment_transactions: {payment_transactions_count}");
    println!("payment_allocations: {payment_allocations_count}");
    println!("invoice_requests: {invoice_requests_count}");
    println!("issued_invoices: {issued_invoices_count}");
    println!("refresh_tokens: {refresh_tokens_count}");
    println!("audit_logs: {audit_logs_count}");
    println!("outbox_events: {outbox_events_count}");

    Ok(())
}

fn print_test_users() {
    println!("Test users:");
    for index in 1..=10 {
        let role = if index == 1 { "admin" } else { "test-user" };
        println!(
            "- {} | password123 | role={} | company={}",
            test_user_email(index),
            role,
            test_company_subdomain(index)
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
            "customer",
            "support",
            "finance",
            "operator",
            "viewer",
            "accountant",
        ];
        let mut role_ids = Vec::with_capacity(role_names.len());
        for role_name in role_names {
            role_ids.push((role_name.to_string(), ensure_role(conn, role_name)?));
        }

        let permission_codes = [
            "audit_logs:read",
            "audit_logs:create",
            "audit_logs:update",
            "audit_logs:delete",
            "companies:read",
            "companies:create",
            "companies:update",
            "companies:delete",
            "company_domains:read",
            "company_domains:create",
            "company_domains:update",
            "company_domains:delete",
            "company_settings:read",
            "company_settings:create",
            "company_settings:update",
            "company_settings:delete",
            "customer_users:read",
            "customer_users:create",
            "customer_users:update",
            "customer_users:delete",
            "customers:read",
            "customers:create",
            "customers:update",
            "customers:delete",
            "debt_categories:read",
            "debt_categories:create",
            "debt_categories:update",
            "debt_categories:delete",
            "debts:read",
            "debts:create",
            "debts:update",
            "debts:delete",
            "documents:read",
            "documents:create",
            "documents:update",
            "documents:delete",
            "invoice_requests:read",
            "invoice_requests:create",
            "invoice_requests:update",
            "invoice_requests:delete",
            "issued_invoices:read",
            "issued_invoices:create",
            "issued_invoices:update",
            "issued_invoices:delete",
            "payment_transactions:read",
            "payment_transactions:create",
            "payment_transactions:update",
            "payment_transactions:delete",
            "roles:read",
            "roles:create",
            "roles:update",
            "roles:delete",
            "storage_objects:read",
            "storage_objects:create",
            "storage_objects:update",
            "storage_objects:delete",
            "users:read",
            "users:create",
            "users:update",
            "users:delete",
        ];

        let mut permission_ids = HashMap::new();
        for code in permission_codes {
            let id = ensure_permission(conn, code, None)?;
            permission_ids.insert(code.to_string(), id);
        }

        let admin_role_id = role_ids
            .iter()
            .find(|(name, _)| name == "admin")
            .map(|(_, id)| *id)
            .expect("admin role must exist");

        for permission_id in permission_ids.values() {
            ensure_role_permission(conn, admin_role_id, *permission_id)?;
        }

        let customer_role_id = role_ids
            .iter()
            .find(|(name, _)| name == "customer")
            .map(|(_, id)| *id)
            .expect("customer role must exist");

        let customer_permissions = [
            "customers:read",
            "customer_users:read",
            "debt_categories:read",
            "debts:read",
            "documents:read",
            "invoice_requests:read",
            "invoice_requests:create",
            "issued_invoices:read",
            "payment_transactions:read",
        ];

        for permission_code in customer_permissions {
            if let Some(permission_id) = permission_ids.get(permission_code) {
                ensure_role_permission(conn, customer_role_id, *permission_id)?;
            }
        }

        for index in 1..=10 {
            let company_id = ensure_company(conn, index)?;
            let _company_domain_id = ensure_company_domain(conn, company_id, index)?;
            let _company_settings_id = ensure_company_settings(conn, company_id)?;
            let user_data = generate_user_data(index);
            let user_id = ensure_user(conn, index)?;
            let _profile_id = ensure_profile(conn, user_id, index, &user_data)?;
            let customer_id = ensure_customer(conn, company_id, index)?;
            let _customer_user_id = ensure_customer_user(conn, customer_id, user_id)?;
            let _customer_contact_id =
                ensure_customer_contact(conn, customer_id, &user_data, index)?;
            let _customer_address_id = ensure_customer_address(conn, customer_id, index)?;
            let role_id = if index == 1 {
                role_ids
                    .iter()
                    .find(|(name, _)| name == "admin")
                    .map(|(_, id)| *id)
                    .expect("admin role must exist")
            } else {
                role_ids[(index - 1) % role_ids.len()].1
            };
            ensure_user_role(conn, user_id, role_id)?;
            let _refresh_token_id = ensure_refresh_token(conn, user_id, index)?;

            let debt_category_id = ensure_debt_category(
                conn,
                company_id,
                &format!("category-{index:02}"),
                &format!("{} {index:02}", commerce::department()),
                index as i32,
            )?;
            let storage_object_id = ensure_storage_object(conn, company_id, user_id, index)?;
            let document_id = ensure_document(conn, company_id, customer_id, user_id, index)?;
            let _document_file_id = ensure_document_file(conn, document_id, storage_object_id)?;
            let debt_id = ensure_debt(
                conn,
                company_id,
                customer_id,
                user_id,
                debt_category_id,
                index,
            )?;
            let payment_transaction_id =
                ensure_payment_transaction(conn, company_id, customer_id, user_id, index)?;
            ensure_payment_allocation(conn, payment_transaction_id, debt_id, index)?;
            let _invoice_request_id =
                ensure_invoice_request(conn, company_id, customer_id, user_id, index)?;
            let issued_invoice_id =
                ensure_issued_invoice(conn, company_id, customer_id, _invoice_request_id, index)?;
            let _audit_log_id = ensure_audit_log(conn, company_id, customer_id, user_id, index)?;
            let _outbox_event_id = ensure_outbox_event(conn, company_id, issued_invoice_id, index)?;
        }

        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    })?;

    println!("Seed completed successfully");
    println!("Admin email: admin@app-name.local");
    println!("Default password: password123");
    print_test_users();
    print_counts(&mut conn)?;

    Ok(())
}
