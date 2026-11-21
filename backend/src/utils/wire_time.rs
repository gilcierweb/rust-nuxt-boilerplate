//! Platform-wide JSON wire format for date and time values.
//!
//! Every timestamp that crosses the HTTP boundary serializes as an
//! ISO 8601 / RFC 3339 string, for example "2026-11-20T15:37:52.984775Z"
//! or "2026-11-15T03:37:58.4446-03:00".
//!
//! # Why not the crate default
//!
//! With `serde-human-readable`, `time::OffsetDateTime` serializes as
//! "2026-11-20 22:09:30.18324218 +00:00:00" (space separator, offset with
//! seconds). That string is outside the ECMAScript date-time grammar, so
//! `new Date(value)` yields `Invalid Date` in every browser and in Node.
//! Without that feature the crate emits a 9-element tuple, which no
//! standard client parses either. RFC 3339 is the widely interoperable
//! choice and it is the exact format the API emitted while models ran on
//! chrono, whose serde default writes RFC 3339.
//!
//! # Locale and country never change the wire
//!
//! A timestamp carries its UTC offset and stays language-neutral; rendering
//! it as dd/mm/yyyy, long text or any other locale-specific shape is the
//! frontend's job (Intl.DateTimeFormat with the viewer locale).
//!
//! # Changing the format
//!
//! The format decision lives in this module only. Model and response fields
//! reference it with `#[serde(with = "crate::utils::wire_time::datetime")]`
//! (or the `option` variant), so switching the platform-wide wire format
//! means editing this file alone.

/// `OffsetDateTime` as RFC 3339. Use with `#[serde(with = "...")]`.
pub mod datetime {
    pub use time::serde::rfc3339::{deserialize, serialize};
}

/// `Option<OffsetDateTime>` as RFC 3339 or null. Use with `#[serde(with = "...")]`.
pub mod option {
    pub use time::serde::rfc3339::option::{deserialize, serialize};
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use time::OffsetDateTime;
    use time::macros::datetime;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Sample {
        #[serde(with = "crate::utils::wire_time::datetime")]
        at: OffsetDateTime,
        #[serde(with = "crate::utils::wire_time::option")]
        maybe: Option<OffsetDateTime>,
        #[serde(with = "crate::utils::wire_time::option")]
        none: Option<OffsetDateTime>,
    }

    #[test]
    fn serializes_datetime_as_rfc3339_string() {
        let sample = Sample {
            at: datetime!(2026-11-20 15:37:52.984775 UTC),
            maybe: Some(datetime!(2026-11-15 03:37:58.4446 -03:00)),
            none: None,
        };

        let json = serde_json::to_value(&sample).expect("serialize");
        assert_eq!(json["at"], "2026-11-20T15:37:52.984775Z");
        assert_eq!(json["maybe"], "2026-11-15T03:37:58.4446-03:00");
        assert!(json["none"].is_null());
        assert!(json["at"].is_string(), "must be a string, never a tuple");
    }

    #[test]
    fn round_trips_rfc3339() {
        let original = Sample {
            at: datetime!(2026-11-20 15:37:52 UTC),
            maybe: Some(datetime!(2026-11-20 15:37:52 UTC)),
            none: None,
        };

        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: Sample = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, original);
    }

    #[test]
    fn module_names_do_not_collide_with_core_option() {
        // `Option` (prelude type) and our `wire_time::option` submodule coexist:
        // the type resolves to core::option::Option, the path resolves to our module.
        let value: Option<OffsetDateTime> = Some(datetime!(2026-01-01 00:00:00 UTC));
        let decoded: Option<OffsetDateTime> = super::option::deserialize(&mut serde_json::Deserializer::from_str(
            "\"2026-01-01T00:00:00Z\"",
        ))
        .expect("deserialize through the option module");
        assert_eq!(value, decoded);
    }
}