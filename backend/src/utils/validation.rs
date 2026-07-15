use validator::{ValidationError, ValidationErrors};

fn resolve_field_label(field: &str) -> String {
    let key = format!("validation.fields.{}", field);
    let locale = rust_i18n::locale();

    if let Some(translated) = crate::_rust_i18n_try_translate(&locale, key.as_str()) {
        return translated.into_owned();
    }

    field.to_owned()
}

fn resolve_field_error_message(field: &str, error: &ValidationError) -> String {
    let field_label = resolve_field_label(field);

    if let Some(message) = &error.message {
        let key_or_message = message.as_ref();
        let translated = t!(key_or_message, field = field_label.as_str()).into_owned();
        if translated != key_or_message {
            return translated;
        }
        return key_or_message
            .replace("%{field}", field_label.as_str())
            .replace("{field}", field_label.as_str());
    }

    if error.code.as_ref() == "required" {
        return t!("validation.field_required", field = field_label.as_str()).into_owned();
    }

    t!("validation.field_invalid", field = field_label.as_str()).into_owned()
}

pub fn first_validation_error_message(errors: &ValidationErrors) -> String {
    for (field, list) in errors.field_errors() {
        if let Some(error) = list.first() {
            return resolve_field_error_message(field, error);
        }
    }

    t!("errors.validation").into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::{ValidationError, ValidationErrors};

    #[test]
    fn resolves_translated_label_for_legal_name_in_pt_br() {
        rust_i18n::set_locale("pt-BR");
        assert_eq!(resolve_field_label("legal_name"), "Razão social");
    }

    #[test]
    fn resolves_translated_label_for_email_in_pt_br() {
        rust_i18n::set_locale("pt-BR");
        assert_eq!(resolve_field_label("email"), "E-mail");
    }

    #[test]
    fn falls_back_to_field_name_for_unknown_field() {
        rust_i18n::set_locale("en");
        assert_eq!(resolve_field_label("unknown_field_xyz"), "unknown_field_xyz");
    }

    #[test]
    fn first_validation_error_message_returns_required_error() {
        rust_i18n::set_locale("pt-BR");
        let mut errors = ValidationErrors::new();
        let mut err = ValidationError::new("required");
        err.message = Some("required".into());
        errors.add("email", err);

        let msg = first_validation_error_message(&errors);
        assert!(!msg.is_empty());
    }

    #[test]
    fn first_validation_error_message_returns_fallback_when_no_errors() {
        rust_i18n::set_locale("pt-BR");
        let errors = ValidationErrors::new();
        let msg = first_validation_error_message(&errors);
        assert!(!msg.is_empty());
    }
}
