use logmix::ts::parse_timestamp;

#[test]
fn ts_rfc3339_utc_z() {
    let nanos = parse_timestamp("2024-01-15T10:30:00Z").expect("rfc3339");
    assert_eq!(nanos, 1_705_314_600_000_000_000);
}

#[test]
fn ts_rfc3339_fractional_and_offset() {
    let nanos = parse_timestamp("2024-01-15T10:30:00.5+00:00").expect("rfc3339 frac");
    assert_eq!(nanos, 1_705_314_600_500_000_000);
}

#[test]
fn ts_iso8601_space_separator_no_tz_treated_as_utc() {
    let nanos = parse_timestamp("2024-01-15 10:30:00").expect("iso8601");
    assert_eq!(nanos, 1_705_314_600_000_000_000);
}

#[test]
fn ts_unix_seconds() {
    let nanos = parse_timestamp("1705314600").expect("unix seconds");
    assert_eq!(nanos, 1_705_314_600_000_000_000);
}

#[test]
fn ts_unix_millis() {
    let nanos = parse_timestamp("1705314600123").expect("unix millis");
    assert_eq!(nanos, 1_705_314_600_123_000_000);
}

#[test]
fn ts_unparseable_returns_none() {
    assert!(parse_timestamp("not-a-timestamp").is_none());
    assert!(parse_timestamp("").is_none());
    assert!(parse_timestamp("2024-13-40T99:99:99Z").is_none());
}
