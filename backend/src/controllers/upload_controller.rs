use actix_multipart::Multipart;
use actix_web::{HttpResponse, post, web};
use futures_util::StreamExt;
use serde::Serialize;

use crate::{
    config::AppConfig,
    errors::{AppError, AppResult},
    middleware::auth::AuthUser,
    utils::file_validation::{
        FileCategory, FileLimits, FileValidationError, ValidatedFile, validate_upload,
    },
};

/// Maximum bytes to buffer for magic byte detection before streaming the rest.
const MAGIC_BYTES_PREFIX: usize = 8192;

/// Response returned after a successful upload validation.
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub filename: String,
    pub content_type: String,
    pub category: &'static str,
    pub size_bytes: u64,
}

impl From<ValidatedFile> for UploadResponse {
    fn from(v: ValidatedFile) -> Self {
        Self {
            filename: v.safe_filename,
            content_type: v.content_type,
            category: match v.category {
                FileCategory::Photo => "photo",
                FileCategory::Video => "video",
                FileCategory::Audio => "audio",
                FileCategory::Document => "document",
            },
            size_bytes: v.size_bytes,
        }
    }
}

fn limits_from_config(config: &AppConfig) -> FileLimits {
    FileLimits {
        max_photo_size_bytes: config.max_photo_size_bytes,
        max_video_size_bytes: config.max_video_size_bytes,
        max_audio_size_bytes: config.max_audio_size_bytes,
        max_document_size_bytes: 20 * 1024 * 1024,
    }
}

fn file_validation_error_to_app(err: FileValidationError) -> AppError {
    match err {
        FileValidationError::FileTooLarge { .. } => AppError::BadRequest(err.to_string()),
        FileValidationError::UnsupportedMimeType(_) => AppError::BadRequest(err.to_string()),
        FileValidationError::MimeTypeMismatch { .. } => AppError::BadRequest(err.to_string()),
        FileValidationError::EmptyFile => AppError::BadRequest(err.to_string()),
        FileValidationError::PathTraversalAttempt => AppError::BadRequest(err.to_string()),
        FileValidationError::NoFileExtension => AppError::BadRequest(err.to_string()),
        FileValidationError::UnsafeFilename => AppError::BadRequest(err.to_string()),
    }
}

/// Upload a single file via multipart form-data.
///
/// The `file` field must contain the file content. The `content_type` field
/// is optional and overrides the declared MIME type from the multipart header.
///
/// # Security
///
/// - Magic bytes are verified against the declared MIME type
/// - Filenames are sanitized: path traversal, null bytes, and dangerous
///   extensions (`.php`, `.exe`, etc.) are rejected
/// - File size is enforced per category
/// - Original filenames are never used for storage
#[post("/upload")]
pub async fn upload_file(
    _user: AuthUser,
    mut payload: Multipart,
    config: web::Data<AppConfig>,
) -> AppResult<HttpResponse> {
    let limits = limits_from_config(&config);

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| AppError::BadRequest(e.to_string()))?;
        let content_disposition = field.content_disposition();
        let field_name = content_disposition
            .as_ref()
            .and_then(|cd| cd.get_name())
            .unwrap_or("")
            .to_string();

        if field_name != "file" {
            continue;
        }

        let original_filename = content_disposition
            .as_ref()
            .and_then(|cd| cd.get_filename())
            .unwrap_or("unknown")
            .to_string();

        let declared_mime = field
            .content_type()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let mut total_size: u64 = 0;
        let mut prefix_buf = Vec::with_capacity(MAGIC_BYTES_PREFIX);
        let mut all_data = Vec::new();

        while let Some(chunk) = field.next().await {
            let chunk = chunk.map_err(|e| AppError::BadRequest(e.to_string()))?;
            total_size += chunk.len() as u64;

            if prefix_buf.len() < MAGIC_BYTES_PREFIX {
                let remaining = MAGIC_BYTES_PREFIX - prefix_buf.len();
                let take = chunk.len().min(remaining);
                prefix_buf.extend_from_slice(&chunk[..take]);
            }

            all_data.extend_from_slice(&chunk);
        }

        let validated = validate_upload(
            &original_filename,
            &declared_mime,
            total_size,
            &prefix_buf,
            &limits,
        )
        .map_err(file_validation_error_to_app)?;

        return Ok(HttpResponse::Ok().json(UploadResponse::from(validated)));
    }

    Err(AppError::BadRequest(
        "No file field found in multipart form".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> AppConfig {
        use crate::config::app_config::Environment;

        AppConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            https_port: 8443,
            tls_cert_path: String::new(),
            tls_key_path: String::new(),
            frontend_url: "http://localhost:3000".to_string(),
            environment: Environment::Test,
            database_url: "postgres://localhost/test".to_string(),
            db_pool_size: 1,
            db_pool_min_idle: Some(1),
            db_pool_max_lifetime_secs: Some(1800),
            db_pool_idle_timeout_secs: Some(600),
            db_pool_connection_timeout_secs: 10,
            db_statement_timeout_secs: Some(30),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            redis_pool_size: 10,
            jwt_secret: "test-secret-key-for-unit-tests-32b!".to_string(),
            jwt_public_key: None,
            jwt_access_expiry_secs: 3600,
            jwt_refresh_expiry_secs: 3600,
            master_key: "dGVzdC1tYXN0ZXIta2V5LWZvci11bml0LXRlc3Rz".to_string(),
            blind_index_key: "dGVzdC1ibluZC1pbmRleC1rZXktZm9yLXRlc3Rz".to_string(),
            current_encryption_key_version: 1,
            internal_api_keys: vec![],
            resend_api_key: String::new(),
            email_from: String::new(),
            email_from_name: String::new(),
            bunny_storage_zone: String::new(),
            bunny_storage_key: String::new(),
            bunny_cdn_url: String::new(),
            bunny_token_key: String::new(),
            bunny_stream_library_id: String::new(),
            bunny_stream_key: String::new(),
            bunny_stream_webhook_secret: String::new(),
            b2_key_id: String::new(),
            b2_application_key: String::new(),
            b2_bucket_id: String::new(),
            b2_bucket_name: String::new(),
            b2_endpoint: String::new(),
            stripe_secret_key: String::new(),
            stripe_webhook_secret: String::new(),
            stripe_publishable_key: String::new(),
            platform_commission_percent: 20.0,
            min_subscription_price_cents: 500,
            max_subscription_price_cents: 50_000,
            min_withdrawal_amount_cents: 2_000,
            totp_issuer: "Test".to_string(),
            max_video_size_bytes: 50 * 1024 * 1024,
            max_photo_size_bytes: 5 * 1024 * 1024,
            max_audio_size_bytes: 10 * 1024 * 1024,
            json_payload_limit: 16 * 1024 * 1024,
            form_payload_limit: 20 * 1024 * 1024,
            csrf_secret_key: "test-csrf-key-32-chars-long!!!!!".to_string(),
            refresh_token_hash_salt: "test-salt-for-refresh-tokens-16b!".to_string(),
            rate_limit_enabled: false,
            jwt_secrets: vec![crate::config::app_config::JwtSecretKey {
                kid: "test-key-1".to_string(),
                secret: "test-secret-key-for-unit-tests-32b!".to_string(),
                created_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
                expires_at: None,
            }],
        }
    }

    #[test]
    fn limits_from_config_uses_config_values() {
        let mut config = test_config();
        config.max_photo_size_bytes = 1024;
        config.max_video_size_bytes = 2048;
        config.max_audio_size_bytes = 4096;
        let limits = limits_from_config(&config);
        assert_eq!(limits.max_photo_size_bytes, 1024);
        assert_eq!(limits.max_video_size_bytes, 2048);
        assert_eq!(limits.max_audio_size_bytes, 4096);
        assert_eq!(limits.max_document_size_bytes, 20 * 1024 * 1024);
    }

    #[test]
    fn upload_response_category_photo() {
        let v = ValidatedFile {
            safe_filename: "test.jpg".into(),
            content_type: "image/jpeg".into(),
            category: FileCategory::Photo,
            size_bytes: 1024,
        };
        let resp: UploadResponse = v.into();
        assert_eq!(resp.category, "photo");
        assert_eq!(resp.filename, "test.jpg");
    }

    #[test]
    fn upload_response_category_video() {
        let v = ValidatedFile {
            safe_filename: "test.mp4".into(),
            content_type: "video/mp4".into(),
            category: FileCategory::Video,
            size_bytes: 2048,
        };
        let resp: UploadResponse = v.into();
        assert_eq!(resp.category, "video");
    }

    #[test]
    fn upload_response_category_audio() {
        let v = ValidatedFile {
            safe_filename: "test.mp3".into(),
            content_type: "audio/mpeg".into(),
            category: FileCategory::Audio,
            size_bytes: 4096,
        };
        let resp: UploadResponse = v.into();
        assert_eq!(resp.category, "audio");
    }

    #[test]
    fn upload_response_category_document() {
        let v = ValidatedFile {
            safe_filename: "doc.pdf".into(),
            content_type: "application/pdf".into(),
            category: FileCategory::Document,
            size_bytes: 8192,
        };
        let resp: UploadResponse = v.into();
        assert_eq!(resp.category, "document");
    }
}
