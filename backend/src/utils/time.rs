use time::{OffsetDateTime, UtcOffset};

#[allow(dead_code)]
pub fn utc_offset() -> UtcOffset {
    UtcOffset::UTC
}

#[allow(dead_code)]
pub fn now_fixed() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

#[allow(dead_code)]
pub fn to_fixed(dt: OffsetDateTime) -> OffsetDateTime {
    dt.to_offset(utc_offset())
}

#[allow(dead_code)]
pub fn to_utc(dt: OffsetDateTime) -> OffsetDateTime {
    dt.to_offset(UtcOffset::UTC)
}
