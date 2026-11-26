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

use rust_i18n::t;
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime, UtcOffset};

use crate::errors::{AppError, AppResult};

/// `OffsetDateTime` as RFC 3339. Use with `#[serde(with = "...")]`.
pub mod datetime {
    pub use time::serde::rfc3339::{deserialize, serialize};
}

/// `Option<OffsetDateTime>` as RFC 3339 or null. Use with `#[serde(with = "...")]`.
pub mod option {
    pub use time::serde::rfc3339::option::{deserialize, serialize};
}

/// Parse an optional query-param date. Invalid values are treated as an
/// absent filter, matching the historical behaviour of list endpoints.
pub fn parse_rfc3339_opt(value: Option<&str>) -> Option<OffsetDateTime> {
    value.and_then(|raw| OffsetDateTime::parse(raw, &Rfc3339).ok())
}

/// Parse a required date, returning a localized validation error instead of
/// silently discarding a malformed value.
pub fn parse_rfc3339(value: &str, field: &str) -> AppResult<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339)
        .map_err(|_| AppError::Validation(t!("validation.field_invalid", field = field).into_owned()))
}

#[allow(dead_code)]
pub fn utc_offset() -> UtcOffset {
    UtcOffset::UTC
}

#[allow(dead_code)]
pub fn now_fixed() -> OffsetDateTime {
    OffsetDateTime::now_utc().to_offset(utc_offset())
}

#[allow(dead_code)]
pub fn to_fixed(dt: OffsetDateTime) -> OffsetDateTime {
    dt.to_offset(utc_offset())
}

#[allow(dead_code)]
pub fn to_utc(dt: OffsetDateTime) -> OffsetDateTime {
    dt.to_offset(UtcOffset::UTC)
}

/// Rails `Date.current` equivalent (now, in UTC).
pub fn today() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

/// Rails `Date#yesterday` equivalent.
pub fn yesterday(dt: OffsetDateTime) -> OffsetDateTime {
    dt.saturating_add(Duration::days(-1))
}

/// Rails `Date#tomorrow` equivalent.
pub fn tomorrow(dt: OffsetDateTime) -> OffsetDateTime {
    dt.saturating_add(Duration::days(1))
}

/// Rails `DateTime#beginning_of_day` equivalent (same offset, time 00:00).
pub fn beginning_of_day(dt: OffsetDateTime) -> OffsetDateTime {
    dt.date().midnight().assume_offset(dt.offset())
}

/// Rails `DateTime#end_of_day` equivalent (23:59:59.999999999, same offset).
pub fn end_of_day(dt: OffsetDateTime) -> OffsetDateTime {
    dt.date().midnight().assume_offset(dt.offset())
        .saturating_add(Duration::seconds(86_399) + Duration::nanoseconds(999_999_999))
}

/// Rails `DateTime#seconds_since_midnight` equivalent.
pub fn seconds_since_midnight(dt: OffsetDateTime) -> u64 {
    u64::from(dt.hour()) * 3600 + u64::from(dt.minute()) * 60 + u64::from(dt.second())
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
        let decoded: Option<OffsetDateTime> = super::option::deserialize(
            &mut serde_json::Deserializer::from_str("\"2026-01-01T00:00:00Z\""),
        )
        .expect("deserialize through the option module");
        assert_eq!(value, decoded);
    }

    #[test]
    fn parses_valid_rfc3339_instant() {
        assert_eq!(
            super::parse_rfc3339("2026-11-20T15:37:52Z", "date_from").expect("valid timestamp"),
            datetime!(2026-11-20 15:37:52 UTC)
        );
    }

    #[test]
    fn rejects_invalid_rfc3339_instant() {
        assert!(super::parse_rfc3339("not-a-date", "date_from").is_err());
    }

    #[test]
    fn optional_parser_ignores_invalid_and_missing_values() {
        assert_eq!(
            super::parse_rfc3339_opt(Some("2026-11-20T15:37:52Z")),
            Some(datetime!(2026-11-20 15:37:52 UTC))
        );
        assert_eq!(super::parse_rfc3339_opt(Some("not-a-date")), None);
        assert_eq!(super::parse_rfc3339_opt(None), None);
    }

    #[test]
    fn yesterday_and_tomorrow_shift_by_one_day() {
        let dt = datetime!(2026-11-20 15:37:52 UTC);
        assert_eq!(super::yesterday(dt), datetime!(2026-11-19 15:37:52 UTC));
        assert_eq!(super::tomorrow(dt), datetime!(2026-11-21 15:37:52 UTC));
    }

    #[test]
    fn beginning_and_end_of_day_keep_the_same_day() {
        let dt = datetime!(2026-11-20 15:37:52.984775 UTC);
        assert_eq!(
            super::beginning_of_day(dt),
            datetime!(2026-11-20 00:00:00 UTC)
        );
        let end = super::end_of_day(dt);
        assert_eq!(end.date(), dt.date());
        assert_eq!(end.hour(), 23);
        assert_eq!(end.minute(), 59);
        assert_eq!(end.second(), 59);
        assert_eq!(end.nanosecond(), 999_999_999);
    }

    #[test]
    fn seconds_since_midnight_returns_time_of_day() {
        assert_eq!(
            super::seconds_since_midnight(datetime!(2026-11-20 12:34:56 UTC)),
            12 * 3600 + 34 * 60 + 56
        );
        assert_eq!(super::seconds_since_midnight(datetime!(2026-11-20 00:00:00 UTC)), 0);
    }

    #[test]
    fn today_is_now() {
        assert_eq!(super::today().date(), OffsetDateTime::now_utc().date());
    }
}
