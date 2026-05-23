use logmix::record::Record;

#[test]
fn record_new_sets_raw_and_source() {
    let r = Record::new("line content", "app.log");
    assert_eq!(r.raw, "line content");
    assert_eq!(r.source, "app.log");
}

#[test]
fn record_new_defaults_ts_to_none() {
    let r = Record::new("x", "src");
    assert!(r.ts.is_none());
}

#[test]
fn record_new_defaults_fields_to_empty() {
    let r = Record::new("x", "src");
    assert!(r.fields.is_empty());
}
