use chrono::{DateTime, FixedOffset, Utc};

#[allow(dead_code)]
pub fn utc_offset() -> FixedOffset {
    FixedOffset::east_opt(0).expect("UTC offset is always valid")
}

#[allow(dead_code)]
pub fn now_fixed() -> DateTime<FixedOffset> {
    Utc::now().with_timezone(&utc_offset())
}

#[allow(dead_code)]
pub fn to_fixed(dt: DateTime<Utc>) -> DateTime<FixedOffset> {
    dt.with_timezone(&utc_offset())
}

#[allow(dead_code)]
pub fn to_utc(dt: DateTime<FixedOffset>) -> DateTime<Utc> {
    dt.with_timezone(&Utc)
}
