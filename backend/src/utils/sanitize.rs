use ammonia;
use std::collections::HashSet;

pub fn sanitize_input(input: &str) -> String {
    input
        .trim()
        .chars()
        .filter(|c| !c.is_control() || matches!(c, '\t' | '\n' | '\r'))
        .collect()
}

#[allow(dead_code)]
pub fn validate_length(input: &str, min: usize, max: usize) -> bool {
    let len = input.chars().count();
    len >= min && len <= max
}

#[allow(dead_code)]
pub fn contains_dangerous_patterns(input: &str) -> bool {
    let lower = input.to_lowercase();
    [
        "<script",
        "javascript:",
        "onerror=",
        "onclick=",
        "union select",
        "drop table",
    ]
    .iter()
    .any(|p| lower.contains(p))
}

#[allow(dead_code)]
pub fn sanitize_html(input: &str) -> String {
    let tags: HashSet<&str> = ["b", "i", "strong", "em", "p", "br", "ul", "ol", "li"]
        .into_iter()
        .collect();

    ammonia::Builder::new().tags(tags).clean(input).to_string()
}

#[allow(dead_code)]
pub fn strip_html(input: &str) -> String {
    ammonia::Builder::new()
        .tags(HashSet::new())
        .clean(input)
        .to_string()
}

pub fn sanitize_for_email(input: &str) -> String {
    ammonia::Builder::new()
        .tags(HashSet::new())
        .clean(input)
        .to_string()
}

pub fn sanitize_for_html_email(input: &str) -> String {
    let allowed: HashSet<&str> = ["b", "i", "strong", "em", "br"].into_iter().collect();
    ammonia::Builder::new()
        .tags(allowed)
        .clean(input)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_input_trims_whitespace() {
        assert_eq!(sanitize_input("  hello  "), "hello");
    }

    #[test]
    fn sanitize_input_strips_control_characters() {
        assert_eq!(sanitize_input("hello\x00world"), "helloworld");
        assert_eq!(sanitize_input("a\x01b\x02c"), "abc");
    }

    #[test]
    fn sanitize_input_preserves_tab_newline_carriage_return() {
        assert_eq!(sanitize_input("a\tb\nc\rd"), "a\tb\nc\rd");
    }

    #[test]
    fn validate_length_within_bounds() {
        assert!(validate_length("hello", 3, 10));
    }

    #[test]
    fn validate_length_below_min() {
        assert!(!validate_length("hi", 3, 10));
    }

    #[test]
    fn validate_length_above_max() {
        assert!(!validate_length("hello world", 3, 5));
    }

    #[test]
    fn validate_length_exact_bounds() {
        assert!(validate_length("abc", 3, 3));
    }

    #[test]
    fn contains_dangerous_patterns_detects_xss() {
        assert!(contains_dangerous_patterns("<script>alert(1)</script>"));
        assert!(contains_dangerous_patterns("javascript:void(0)"));
        assert!(contains_dangerous_patterns("onerror=alert(1)"));
        assert!(contains_dangerous_patterns("onclick=alert(1)"));
    }

    #[test]
    fn contains_dangerous_patterns_detects_sqli() {
        assert!(contains_dangerous_patterns("UNION SELECT * FROM users"));
        assert!(contains_dangerous_patterns("DROP TABLE users"));
    }

    #[test]
    fn contains_dangerous_patterns_case_insensitive() {
        assert!(contains_dangerous_patterns("<SCRIPT>alert(1)</SCRIPT>"));
        assert!(contains_dangerous_patterns("JavaScript:void(0)"));
    }

    #[test]
    fn contains_dangerous_patterns_safe_input() {
        assert!(!contains_dangerous_patterns("hello world"));
        assert!(!contains_dangerous_patterns("user@example.com"));
    }

    #[test]
    fn sanitize_html_allows_safe_tags() {
        let input = "<b>bold</b> <script>alert(1)</script>";
        let result = sanitize_html(input);
        assert!(result.contains("<b>bold</b>"));
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn sanitize_html_strips_unsafe_tags() {
        let input = "<img src=x onerror=alert(1)>";
        let result = sanitize_html(input);
        assert!(!result.contains("<img"));
    }

    #[test]
    fn strip_html_removes_all_tags() {
        let input = "<p>Hello <b>world</b></p>";
        let result = strip_html(input);
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn strip_html_empty_input() {
        assert_eq!(strip_html(""), "");
    }

    #[test]
    fn sanitize_for_email_strips_all_tags() {
        let input = "<script>alert('xss')</script>user@example.com";
        let result = sanitize_for_email(input);
        assert_eq!(result, "user@example.com");
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn sanitize_for_email_strips_img_onerror() {
        let input = "<img src=x onerror=alert(1)>user@example.com";
        let result = sanitize_for_email(input);
        assert!(!result.contains("<img"));
        assert!(result.contains("user@example.com"));
    }

    #[test]
    fn sanitize_for_email_preserves_plain_text() {
        let result = sanitize_for_email("hello world");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn sanitize_for_html_email_allows_safe_tags() {
        let input = "<b>bold</b> and <i>italic</i> user";
        let result = sanitize_for_html_email(input);
        assert!(result.contains("<b>bold</b>"));
        assert!(result.contains("<i>italic</i>"));
    }

    #[test]
    fn sanitize_for_html_email_strips_script() {
        let input = "<script>alert(1)</script><b>safe</b>";
        let result = sanitize_for_html_email(input);
        assert!(!result.contains("<script>"));
        assert!(result.contains("<b>safe</b>"));
    }

    #[test]
    fn sanitize_for_html_email_strips_event_handlers() {
        let input = "<b onclick=alert(1)>text</b>";
        let result = sanitize_for_html_email(input);
        assert!(!result.contains("onclick"));
        assert!(result.contains("text"));
    }
}
