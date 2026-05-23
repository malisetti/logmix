use logmix::output::{write_record, Format};
use logmix::record::Record;
use std::io::Cursor;

fn format_record(format: Format, raw: &str, source: &str, ts: Option<i64>) -> String {
    let mut r = Record::new(raw, source);
    r.ts = ts;
    let mut buf = Cursor::new(Vec::new());
    write_record(&mut buf, &r, format).unwrap();
    String::from_utf8(buf.into_inner()).unwrap()
}

#[test]
fn passthrough_prefixes_source_and_raw() {
    let out = format_record(Format::Passthrough, "hello world", "app.log", None);
    assert_eq!(out, "[app.log] hello world\n");
}

#[test]
fn jsonl_emits_source_ts_raw_object() {
    let out = format_record(Format::Jsonl, "line", "src", Some(42));
    assert_eq!(
        out,
        r#"{"source":"src","ts":42,"raw":"line"}"#.to_string() + "\n"
    );
}

#[test]
fn jsonl_ts_null_when_missing() {
    let out = format_record(Format::Jsonl, "line", "src", None);
    assert_eq!(
        out,
        r#"{"source":"src","ts":null,"raw":"line"}"#.to_string() + "\n"
    );
}

#[test]
fn tagged_tab_separates_source_ts_raw() {
    let out = format_record(Format::Tagged, "line", "src", Some(99));
    assert_eq!(out, "src\t99\tline\n");
}

#[test]
fn tagged_empty_ts_when_missing() {
    let out = format_record(Format::Tagged, "line", "src", None);
    assert_eq!(out, "src\t\tline\n");
}
