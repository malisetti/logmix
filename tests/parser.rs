use logmix::parser::{parse_jsonl, parse_logfmt, parse_plain, sniff_and_parse};

#[test]
fn parse_jsonl_extracts_fields_and_ts() {
    let r = parse_jsonl(r#"{"msg":"hi","ts":123}"#, "app.log").unwrap();
    assert_eq!(r.raw, r#"{"msg":"hi","ts":123}"#);
    assert_eq!(r.source, "app.log");
    assert_eq!(r.ts, Some(123));
    assert_eq!(r.fields.get("msg").map(String::as_str), Some("hi"));
}

#[test]
fn parse_jsonl_ts_from_time_and_timestamp_aliases() {
    let time = parse_jsonl(r#"{"time":"456"}"#, "s").unwrap();
    assert_eq!(time.ts, Some(456));
    let at = parse_jsonl(r#"{"@timestamp":789}"#, "s").unwrap();
    assert_eq!(at.ts, Some(789));
}

#[test]
fn parse_jsonl_flattens_one_level() {
    let r = parse_jsonl(r#"{"user":{"id":"42","name":"ada"}}"#, "s").unwrap();
    assert_eq!(r.fields.get("user.id").map(String::as_str), Some("42"));
    assert_eq!(r.fields.get("user.name").map(String::as_str), Some("ada"));
}

#[test]
fn parse_jsonl_invalid_returns_none() {
    assert!(parse_jsonl("not-json", "s").is_none());
    assert!(parse_jsonl("[1,2,3]", "s").is_none());
}

#[test]
fn parse_logfmt_extracts_pairs_and_ts() {
    let r = parse_logfmt("level=info msg=hello ts=999", "svc").unwrap();
    assert_eq!(r.raw, "level=info msg=hello ts=999");
    assert_eq!(r.source, "svc");
    assert_eq!(r.ts, Some(999));
    assert_eq!(r.fields.get("level").map(String::as_str), Some("info"));
    assert_eq!(r.fields.get("msg").map(String::as_str), Some("hello"));
}

#[test]
fn parse_logfmt_handles_quoted_values() {
    let r = parse_logfmt(r#"msg="hello world" level=warn"#, "s").unwrap();
    assert_eq!(r.fields.get("msg").map(String::as_str), Some("hello world"));
    assert_eq!(r.fields.get("level").map(String::as_str), Some("warn"));
}

#[test]
fn parse_logfmt_empty_or_invalid_returns_none() {
    assert!(parse_logfmt("", "s").is_none());
    assert!(parse_logfmt("   ", "s").is_none());
    assert!(parse_logfmt("noequals", "s").is_none());
}

#[test]
fn parse_plain_sets_raw_and_empty_fields() {
    let r = parse_plain("plain text line", "stdin");
    assert_eq!(r.raw, "plain text line");
    assert_eq!(r.source, "stdin");
    assert!(r.ts.is_none());
    assert!(r.fields.is_empty());
}

#[test]
fn sniff_and_parse_uses_json_when_valid() {
    let r = sniff_and_parse(r#"{"msg":"json","ts":1}"#, "s");
    assert_eq!(r.ts, Some(1));
    assert_eq!(r.fields.get("msg").map(String::as_str), Some("json"));
}

#[test]
fn sniff_and_parse_falls_back_to_logfmt() {
    let r = sniff_and_parse("level=error ts=2", "s");
    assert_eq!(r.ts, Some(2));
    assert_eq!(r.fields.get("level").map(String::as_str), Some("error"));
}

#[test]
fn sniff_and_parse_falls_back_to_plain() {
    let r = sniff_and_parse("just some text", "s");
    assert_eq!(r.raw, "just some text");
    assert!(r.fields.is_empty());
    assert!(r.ts.is_none());
}
