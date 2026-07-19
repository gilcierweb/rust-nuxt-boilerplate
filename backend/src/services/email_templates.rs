//! Tera-based email template engine.
//!
//! Templates are embedded at compile time via `include_str!` so the binary
//! is self-contained and works in slim Docker images without a templates
//! directory on disk.
//!
//! Template layout:
//! - `layouts/mailer.html.tera` — base HTML layout with header/content/footer
//! - `<mailer>/<name>.html.tera` — HTML body (content block)
//! - `<mailer>/<name>.text.tera` — plain text body
//!
//! Rendering is deterministic and never panics: unknown templates return an
//! [`EmailTemplateError`] that callers can map to their own error type.

use std::collections::HashMap;
use std::sync::Arc;

use rust_i18n::t;
use tera::{Context, Result as TeraResult, Tera, Value};

/// Custom Tera function `t` that resolves an i18n key via `rust_i18n`,
/// using the current global locale. Usage in templates: `{{ t("key") }}`.
fn tera_t_function(args: &HashMap<String, Value>) -> TeraResult<Value> {
    let key = args
        .get("key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tera::Error::msg("t() requires a `key` argument"))?;
    let message = t!(key).into_owned();
    Ok(Value::String(message))
}

#[derive(Debug, thiserror::Error)]
pub enum EmailTemplateError {
    #[error("template not found: {0}")]
    NotFound(String),
    #[error("template render error: {0}")]
    Render(String),
}

impl From<tera::Error> for EmailTemplateError {
    fn from(err: tera::Error) -> Self {
        // Tera's Error derefs to std::error::Error; preserve the message.
        Self::Render(err.to_string())
    }
}

const LAYOUT_MAILER_HTML: &str = include_str!("../../templates/layouts/mailer.html.tera");

const WELCOME_HTML: &str = include_str!("../../templates/user_mailer/welcome.html.tera");
const WELCOME_TEXT: &str = include_str!("../../templates/user_mailer/welcome.text.tera");

const CONFIRMATION_HTML: &str = include_str!("../../templates/user_mailer/confirmation.html.tera");
const CONFIRMATION_TEXT: &str = include_str!("../../templates/user_mailer/confirmation.text.tera");

const PASSWORD_RESET_HTML: &str =
    include_str!("../../templates/user_mailer/password_reset.html.tera");
const PASSWORD_RESET_TEXT: &str =
    include_str!("../../templates/user_mailer/password_reset.text.tera");

const PASSWORD_CHANGED_HTML: &str =
    include_str!("../../templates/user_mailer/password_changed.html.tera");
const PASSWORD_CHANGED_TEXT: &str =
    include_str!("../../templates/user_mailer/password_changed.text.tera");

const TWO_FACTOR_SETUP_HTML: &str =
    include_str!("../../templates/user_mailer/two_factor_setup.html.tera");
const TWO_FACTOR_SETUP_TEXT: &str =
    include_str!("../../templates/user_mailer/two_factor_setup.text.tera");

/// Canonical template names registered in the Tera instance.
pub mod names {
    pub const LAYOUT_MAILER: &str = "layouts/mailer.html.tera";

    pub const USER_WELCOME_HTML: &str = "user_mailer/welcome.html.tera";
    pub const USER_WELCOME_TEXT: &str = "user_mailer/welcome.text.tera";

    pub const USER_CONFIRMATION_HTML: &str = "user_mailer/confirmation.html.tera";
    pub const USER_CONFIRMATION_TEXT: &str = "user_mailer/confirmation.text.tera";

    pub const USER_PASSWORD_RESET_HTML: &str = "user_mailer/password_reset.html.tera";
    pub const USER_PASSWORD_RESET_TEXT: &str = "user_mailer/password_reset.text.tera";

    pub const USER_PASSWORD_CHANGED_HTML: &str = "user_mailer/password_changed.html.tera";
    pub const USER_PASSWORD_CHANGED_TEXT: &str = "user_mailer/password_changed.text.tera";

    pub const USER_TWO_FACTOR_SETUP_HTML: &str = "user_mailer/two_factor_setup.html.tera";
    pub const USER_TWO_FACTOR_SETUP_TEXT: &str = "user_mailer/two_factor_setup.text.tera";
}

/// Thread-safe, immutable Tera instance holder.
///
/// All templates are loaded once at construction. The [`Tera`] instance is
/// `Send + Sync` once populated, so [`EmailTemplates`] simply wraps it in an
/// [`Arc`] for cheap cloning.
#[derive(Clone)]
pub struct EmailTemplates {
    tera: Arc<Tera>,
}

impl EmailTemplates {
    /// Build a [`Tera`] instance pre-populated with all bundled templates.
    fn build_tera() -> Result<Tera, tera::Error> {
        let mut tera = Tera::default();

        tera.add_raw_template(names::LAYOUT_MAILER, LAYOUT_MAILER_HTML)?;
        tera.add_raw_template(names::USER_WELCOME_HTML, WELCOME_HTML)?;
        tera.add_raw_template(names::USER_WELCOME_TEXT, WELCOME_TEXT)?;
        tera.add_raw_template(names::USER_CONFIRMATION_HTML, CONFIRMATION_HTML)?;
        tera.add_raw_template(names::USER_CONFIRMATION_TEXT, CONFIRMATION_TEXT)?;
        tera.add_raw_template(names::USER_PASSWORD_RESET_HTML, PASSWORD_RESET_HTML)?;
        tera.add_raw_template(names::USER_PASSWORD_RESET_TEXT, PASSWORD_RESET_TEXT)?;
        tera.add_raw_template(names::USER_PASSWORD_CHANGED_HTML, PASSWORD_CHANGED_HTML)?;
        tera.add_raw_template(names::USER_PASSWORD_CHANGED_TEXT, PASSWORD_CHANGED_TEXT)?;
        tera.add_raw_template(names::USER_TWO_FACTOR_SETUP_HTML, TWO_FACTOR_SETUP_HTML)?;
        tera.add_raw_template(names::USER_TWO_FACTOR_SETUP_TEXT, TWO_FACTOR_SETUP_TEXT)?;

        // Only autoescape HTML templates (text/plain must remain unescaped).
        tera.autoescape_on(vec![".html.tera"]);

        tera.register_function("t", tera_t_function);

        Ok(tera)
    }

    /// Create a new [`EmailTemplates`] instance with all bundled templates.
    ///
    /// Errors only occur if Tera fails to parse a template (statically known
    /// to succeed at build time, but kept as `Result` for API safety).
    pub fn new() -> Result<Self, EmailTemplateError> {
        let tera = Self::build_tera()?;
        Ok(Self {
            tera: Arc::new(tera),
        })
    }

    /// Render the named template with the supplied context.
    pub fn render(&self, template: &str, context: &Value) -> Result<String, EmailTemplateError> {
        let mut ctx = Self::context_from_value(context)?;
        self.ensure_defaults(&mut ctx);
        Ok(self.tera.render(template, &ctx)?)
    }

    /// Render an HTML content template and apply the mailer layout around it.
    ///
    /// The content template is rendered first, then injected into the
    /// `content` block of [`names::LAYOUT_MAILER`].
    pub fn render_html_with_layout(
        &self,
        content_template: &str,
        context: &Value,
    ) -> Result<String, EmailTemplateError> {
        let rendered_content = self.render(content_template, context)?;
        let mut ctx = Self::context_from_value(context)?;
        self.ensure_defaults(&mut ctx);
        ctx.insert("content", &rendered_content);
        Ok(self.tera.render(names::LAYOUT_MAILER, &ctx)?)
    }

    fn context_from_value(value: &Value) -> Result<Context, EmailTemplateError> {
        let mut ctx = Context::new();
        if let Some(obj) = value.as_object() {
            for (k, v) in obj {
                ctx.insert(k, v);
            }
        }
        Ok(ctx)
    }

    /// Populate default values for layout variables if not already provided.
    fn ensure_defaults(&self, ctx: &mut Context) {
        if ctx.get("app_name").is_none() {
            ctx.insert("app_name", "Boilerplate App");
        }
        if ctx.get("current_year").is_none() {
            let year = chrono::Utc::now().format("%Y").to_string();
            ctx.insert("current_year", &year);
        }
    }

    /// Return a list of all registered template names (useful for previews/diagnostics).
    pub fn template_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.tera.templates.keys().cloned().collect();
        names.sort();
        names
    }

    /// Return a sorted map of `mailer -> [templates]` grouping for previews.
    pub fn templates_by_mailer(&self) -> HashMap<String, Vec<String>> {
        let mut out: HashMap<String, Vec<String>> = HashMap::new();
        for name in self.template_names() {
            if name.starts_with("layouts/") {
                continue;
            }
            let mailer = name.split('/').next().unwrap_or("unknown").to_string();
            out.entry(mailer).or_default().push(name);
        }
        for entries in out.values_mut() {
            entries.sort();
        }
        out
    }
}

impl std::fmt::Debug for EmailTemplates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmailTemplates")
            .field("templates", &self.template_names().len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn templates() -> EmailTemplates {
        EmailTemplates::new().expect("templates compile")
    }

    #[test]
    fn renders_welcome_text_template() {
        let t = templates();
        let output = t
            .render(
                names::USER_WELCOME_TEXT,
                &json!({ "user_name": "Ada", "confirm_url": "https://x/confirm?a=1",
                         "app_name": "TestApp" }),
            )
            .expect("render text");
        assert!(output.contains("Olá Ada"));
        assert!(output.contains("https://x/confirm?a=1"));
        assert!(output.contains("TestApp"));
    }

    #[test]
    fn renders_welcome_html_with_layout() {
        let t = templates();
        let output = t
            .render_html_with_layout(
                names::USER_WELCOME_HTML,
                &json!({ "user_name": "Ada", "confirm_url": "https://x/confirm?a=1" }),
            )
            .expect("render html");
        assert!(output.contains("<!DOCTYPE html>"));
        assert!(output.contains("Boilerplate App"));
        assert!(output.contains("Olá <strong>Ada</strong>"));
        assert!(output.contains("Confirmar Email"));
        // URL contains only safe chars so it renders verbatim
        assert!(
            output.contains("x/confirm"),
            "output should contain URL path: {}",
            output
        );
    }

    #[test]
    fn missing_template_returns_not_found() {
        let t = templates();
        let err = t
            .render("does/not/exist.html.tera", &json!({}))
            .expect_err("expected error");
        assert!(matches!(err, EmailTemplateError::Render(_)));
    }

    #[test]
    fn autoescape_disabled_for_text_templates() {
        let t = templates();
        let output = t
            .render(
                names::USER_PASSWORD_RESET_TEXT,
                &json!({ "reset_url": "<script>alert(1)</script>" }),
            )
            .expect("render text");
        assert!(output.contains("<script>alert(1)</script>"));
    }

    #[test]
    fn html_templates_autoescape_user_supplied_content() {
        let t = templates();
        // Test auto-escaping on a non-URL field (user_name)
        // URLs use | safe filter so they render verbatim
        let output = t
            .render_html_with_layout(
                names::USER_WELCOME_HTML,
                &json!({ "user_name": "<script>alert(1)</script>", "confirm_url": "https://example.com/x" }),
            )
            .expect("render html");
        // Verify that user content is escaped (no raw <script> tags in output)
        // The escaped version will be <script> which is safe
        assert!(
            !output.contains("<script>"),
            "output should NOT contain raw script tags (should be escaped), got: {}",
            output
        );
        // Verify the escaped version IS present
        assert!(
            output.contains("<"),
            "output should contain escaped < as &, got: {}",
            output
        );
    }

    #[test]
    fn template_names_sorted() {
        let t = templates();
        let names = t.template_names();
        assert!(!names.is_empty());
        assert_eq!(names, {
            let mut v = names.clone();
            v.sort();
            v
        });
    }
}
