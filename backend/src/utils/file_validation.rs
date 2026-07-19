use std::path::Path;

/// Categories of uploaded files with associated size limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileCategory {
    Photo,
    Video,
    Audio,
    Document,
}

/// Result of validating a file upload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedFile {
    pub safe_filename: String,
    pub content_type: String,
    pub category: FileCategory,
    pub size_bytes: u64,
}

/// Errors that can occur during file validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileValidationError {
    FileTooLarge { max: u64, actual: u64 },
    UnsupportedMimeType(String),
    MimeTypeMismatch { declared: String, detected: String },
    EmptyFile,
    PathTraversalAttempt,
    NoFileExtension,
    UnsafeFilename,
}

impl std::fmt::Display for FileValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileTooLarge { max, actual } => {
                write!(
                    f,
                    "{}",
                    t!("validation.file_too_large", size = actual, max = max)
                )
            },
            Self::UnsupportedMimeType(mime) => {
                write!(f, "{}", t!("validation.unsupported_mime_type", mime = mime))
            },
            Self::MimeTypeMismatch { declared, detected } => {
                write!(
                    f,
                    "{}",
                    t!(
                        "validation.mime_type_mismatch",
                        declared = declared,
                        detected = detected
                    )
                )
            },
            Self::EmptyFile => write!(f, "{}", t!("validation.empty_file").into_owned()),
            Self::PathTraversalAttempt => {
                write!(f, "{}", t!("validation.path_traversal").into_owned())
            },
            Self::NoFileExtension => write!(f, "{}", t!("validation.no_extension").into_owned()),
            Self::UnsafeFilename => write!(f, "{}", t!("validation.unsafe_filename").into_owned()),
        }
    }
}

impl std::error::Error for FileValidationError {}

/// Allowed MIME types per category.
fn allowed_photo_types() -> &'static [&'static str] {
    &[
        "image/jpeg",
        "image/png",
        "image/webp",
        "image/gif",
        "image/avif",
    ]
}

fn allowed_video_types() -> &'static [&'static str] {
    &[
        "video/mp4",
        "video/webm",
        "video/quicktime",
        "video/x-msvideo",
    ]
}

fn allowed_audio_types() -> &'static [&'static str] {
    &[
        "audio/mpeg",
        "audio/wav",
        "audio/ogg",
        "audio/mp4",
        "audio/webm",
    ]
}

fn allowed_document_types() -> &'static [&'static str] {
    &["application/pdf", "image/jpeg", "image/png", "image/webp"]
}

/// Returns the maximum file size in bytes for a given category.
pub fn max_size_for_category(category: FileCategory, config: &FileLimits) -> u64 {
    match category {
        FileCategory::Photo => config.max_photo_size_bytes,
        FileCategory::Video => config.max_video_size_bytes,
        FileCategory::Audio => config.max_audio_size_bytes,
        FileCategory::Document => config.max_document_size_bytes,
    }
}

/// Configuration for file upload limits.
#[derive(Debug, Clone)]
pub struct FileLimits {
    pub max_photo_size_bytes: u64,
    pub max_video_size_bytes: u64,
    pub max_audio_size_bytes: u64,
    pub max_document_size_bytes: u64,
}

impl Default for FileLimits {
    fn default() -> Self {
        Self {
            max_photo_size_bytes: 50 * 1024 * 1024,
            max_video_size_bytes: 10 * 1024 * 1024 * 1024,
            max_audio_size_bytes: 500 * 1024 * 1024,
            max_document_size_bytes: 20 * 1024 * 1024,
        }
    }
}

/// Detect the file category from a MIME type string.
pub fn detect_category(mime_type: &str) -> Option<FileCategory> {
    if allowed_photo_types().contains(&mime_type) {
        Some(FileCategory::Photo)
    } else if allowed_video_types().contains(&mime_type) {
        Some(FileCategory::Video)
    } else if allowed_audio_types().contains(&mime_type) {
        Some(FileCategory::Audio)
    } else if allowed_document_types().contains(&mime_type) {
        Some(FileCategory::Document)
    } else {
        None
    }
}

/// Check if a MIME type is allowed for any supported category.
pub fn is_allowed_mime_type(mime_type: &str) -> bool {
    detect_category(mime_type).is_some()
}

/// Validate magic bytes against declared MIME type.
/// Returns the detected MIME type from magic bytes.
pub fn detect_mime_from_bytes(data: &[u8]) -> Option<&'static str> {
    infer::get(data).map(|kind| kind.mime_type())
}

/// Sanitize a filename: remove path traversal, strip unsafe characters,
/// and generate a secure UUID-based name while preserving the extension.
pub fn sanitize_filename(original_filename: &str) -> Result<String, FileValidationError> {
    let filename = original_filename.trim();

    if filename.is_empty() {
        return Err(FileValidationError::UnsafeFilename);
    }

    // Block path traversal attempts
    if filename.contains("..")
        || filename.contains('/')
        || filename.contains('\\')
        || filename.contains('\0')
    {
        return Err(FileValidationError::PathTraversalAttempt);
    }

    // Extract extension
    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .ok_or(FileValidationError::NoFileExtension)?;

    // Validate extension contains only safe characters
    if !ext.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(FileValidationError::UnsafeFilename);
    }

    // Block double extensions that could be used to disguise file types
    let dangerous_double_exts = [
        "php", "php3", "php4", "php5", "phtml", "phps", "cgi", "pl", "py", "rb", "sh", "bash",
        "jsp", "jspx", "asp", "aspx", "cer", "cfm", "htaccess", "htpasswd", "shtml",
    ];
    if dangerous_double_exts
        .iter()
        .any(|&d| ext.eq_ignore_ascii_case(d))
    {
        return Err(FileValidationError::UnsupportedMimeType(format!(
            ".{} files are not allowed",
            ext
        )));
    }

    // Generate secure filename: UUID + lowercase extension
    let secure_name = format!("{}.{}", uuid::Uuid::new_v4(), ext.to_lowercase());
    Ok(secure_name)
}

/// Strip directory components from a filename, returning only the base name.
pub fn strip_path_components(filename: &str) -> &str {
    Path::new(filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(filename)
}

/// Validate a complete file upload.
///
/// Checks:
/// 1. File is not empty
/// 2. Filename is safe (no path traversal)
/// 3. MIME type is allowed
/// 4. Magic bytes match declared MIME type
/// 5. File size is within category limits
pub fn validate_upload(
    original_filename: &str,
    declared_mime: &str,
    file_size: u64,
    data_prefix: &[u8],
    limits: &FileLimits,
) -> Result<ValidatedFile, FileValidationError> {
    if file_size == 0 {
        return Err(FileValidationError::EmptyFile);
    }

    // Check for path traversal on the ORIGINAL filename before stripping
    if original_filename.contains("..")
        || original_filename.contains('/')
        || original_filename.contains('\\')
        || original_filename.contains('\0')
    {
        return Err(FileValidationError::PathTraversalAttempt);
    }

    let base_name = strip_path_components(original_filename);
    let safe_filename = sanitize_filename(base_name)?;

    let category = detect_category(declared_mime)
        .ok_or_else(|| FileValidationError::UnsupportedMimeType(declared_mime.to_string()))?;

    // Validate magic bytes match declared MIME type
    if !data_prefix.is_empty()
        && let Some(detected_mime) = detect_mime_from_bytes(data_prefix)
        && detected_mime != declared_mime
    {
        return Err(FileValidationError::MimeTypeMismatch {
            declared: declared_mime.to_string(),
            detected: detected_mime.to_string(),
        });
    }

    // Check size limit
    let max = max_size_for_category(category, limits);
    if file_size > max {
        return Err(FileValidationError::FileTooLarge {
            max,
            actual: file_size,
        });
    }

    Ok(ValidatedFile {
        safe_filename,
        content_type: declared_mime.to_string(),
        category,
        size_bytes: file_size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_limits() -> FileLimits {
        FileLimits {
            max_photo_size_bytes: 5 * 1024 * 1024,
            max_video_size_bytes: 50 * 1024 * 1024,
            max_audio_size_bytes: 10 * 1024 * 1024,
            max_document_size_bytes: 10 * 1024 * 1024,
        }
    }

    #[test]
    fn sanitize_filename_removes_path_traversal() {
        assert_eq!(
            sanitize_filename("../../../etc/passwd"),
            Err(FileValidationError::PathTraversalAttempt)
        );
    }

    #[test]
    fn sanitize_filename_blocks_double_dots() {
        assert_eq!(
            sanitize_filename("..hidden.jpg"),
            Err(FileValidationError::PathTraversalAttempt)
        );
    }

    #[test]
    fn sanitize_filename_blocks_backslashes() {
        assert_eq!(
            sanitize_filename("C:\\Windows\\file.exe"),
            Err(FileValidationError::PathTraversalAttempt)
        );
    }

    #[test]
    fn sanitize_filename_blocks_null_bytes() {
        assert_eq!(
            sanitize_filename("file\0.jpg"),
            Err(FileValidationError::PathTraversalAttempt)
        );
    }

    #[test]
    fn sanitize_filename_blocks_dangerous_extensions() {
        assert!(sanitize_filename("shell.php").is_err());
        assert!(sanitize_filename("script.cgi").is_err());
        assert!(sanitize_filename("code.jsp").is_err());
        assert!(sanitize_filename("page.aspx").is_err());
        assert!(sanitize_filename("config.htaccess").is_err());
    }

    #[test]
    fn sanitize_filename_generates_uuid_name() {
        let result = sanitize_filename("photo.jpg").unwrap();
        assert!(result.ends_with(".jpg"));
        assert_ne!(result, "photo.jpg");
        let name_part = result.strip_suffix(".jpg").unwrap();
        assert!(uuid::Uuid::parse_str(name_part).is_ok());
    }

    #[test]
    fn sanitize_filename_lowercases_extension() {
        let result = sanitize_filename("image.JPEG").unwrap();
        assert!(result.ends_with(".jpeg"));
    }

    #[test]
    fn sanitize_filename_rejects_empty() {
        assert_eq!(
            sanitize_filename(""),
            Err(FileValidationError::UnsafeFilename)
        );
    }

    #[test]
    fn sanitize_filename_rejects_no_extension() {
        assert_eq!(
            sanitize_filename("noext"),
            Err(FileValidationError::NoFileExtension)
        );
    }

    #[test]
    fn sanitize_filename_rejects_special_chars_in_ext() {
        assert_eq!(
            sanitize_filename("file.j!pg"),
            Err(FileValidationError::UnsafeFilename)
        );
    }

    #[test]
    fn strip_path_components_works() {
        assert_eq!(strip_path_components("/foo/bar/baz.jpg"), "baz.jpg");
        assert_eq!(strip_path_components("baz.jpg"), "baz.jpg");
    }

    #[test]
    fn detect_category_works() {
        assert_eq!(detect_category("image/jpeg"), Some(FileCategory::Photo));
        assert_eq!(detect_category("video/mp4"), Some(FileCategory::Video));
        assert_eq!(detect_category("audio/mpeg"), Some(FileCategory::Audio));
        assert_eq!(
            detect_category("application/pdf"),
            Some(FileCategory::Document)
        );
        assert_eq!(detect_category("application/x-msdownload"), None);
    }

    #[test]
    fn is_allowed_mime_type_works() {
        assert!(is_allowed_mime_type("image/png"));
        assert!(is_allowed_mime_type("video/webm"));
        assert!(!is_allowed_mime_type("application/x-executable"));
    }

    #[test]
    fn validate_upload_rejects_empty_file() {
        let limits = test_limits();
        assert_eq!(
            validate_upload("photo.jpg", "image/jpeg", 0, &[], &limits),
            Err(FileValidationError::EmptyFile)
        );
    }

    #[test]
    fn validate_upload_rejects_oversized_photo() {
        let limits = test_limits();
        let big = vec![0u8; 6 * 1024 * 1024]; // 6MB > 5MB limit
        assert!(matches!(
            validate_upload("photo.jpg", "image/jpeg", big.len() as u64, &big, &limits),
            Err(FileValidationError::FileTooLarge { .. })
        ));
    }

    #[test]
    fn validate_upload_rejects_unsupported_type() {
        let limits = test_limits();
        assert!(matches!(
            validate_upload("malware.exe", "application/x-msdownload", 100, &[], &limits),
            Err(FileValidationError::UnsupportedMimeType(_))
        ));
    }

    #[test]
    fn validate_upload_rejects_path_traversal() {
        let limits = test_limits();
        assert!(matches!(
            validate_upload("../../../etc/passwd", "image/jpeg", 100, &[], &limits),
            Err(FileValidationError::PathTraversalAttempt)
        ));
    }

    #[test]
    fn validate_upload_accepts_valid_jpeg() {
        let limits = test_limits();
        // JPEG magic bytes: FF D8 FF
        let data = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        let result = validate_upload("photo.jpg", "image/jpeg", 1024, &data, &limits);
        assert!(result.is_ok());
        let v = result.unwrap();
        assert_eq!(v.category, FileCategory::Photo);
        assert!(v.safe_filename.ends_with(".jpg"));
        assert_eq!(v.size_bytes, 1024);
    }

    #[test]
    fn validate_upload_rejects_mime_mismatch() {
        let limits = test_limits();
        // Declared as JPEG but PNG magic bytes
        let data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A];
        assert!(matches!(
            validate_upload("photo.jpg", "image/jpeg", 1024, &data, &limits),
            Err(FileValidationError::MimeTypeMismatch { .. })
        ));
    }

    #[test]
    fn validate_upload_skips_magic_check_when_prefix_empty() {
        let limits = test_limits();
        let result = validate_upload("photo.jpg", "image/jpeg", 1024, &[], &limits);
        assert!(result.is_ok());
    }

    #[test]
    fn max_size_for_category_works() {
        let limits = test_limits();
        assert_eq!(
            max_size_for_category(FileCategory::Photo, &limits),
            5 * 1024 * 1024
        );
        assert_eq!(
            max_size_for_category(FileCategory::Video, &limits),
            50 * 1024 * 1024
        );
    }
}
