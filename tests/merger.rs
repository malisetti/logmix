use logmix::merger::merge;
use logmix::record::Record;

fn record_with_ts(ts: i64, raw: &str, source: &str) -> Record {
    let mut r = Record::new(raw, source);
    r.ts = Some(ts);
    r
}

#[test]
fn two_sources_interleave_by_timestamp() {
    let a = vec![record_with_ts(2, "b", "a"), record_with_ts(5, "e", "a")];
    let b = vec![
        record_with_ts(1, "a", "b"),
        record_with_ts(3, "c", "b"),
        record_with_ts(4, "d", "b"),
    ];
    let merged: Vec<_> = merge(vec![a.into_iter(), b.into_iter()]).collect();
    assert_eq!(
        merged.iter().map(|r| r.raw.as_str()).collect::<Vec<_>>(),
        vec!["a", "b", "c", "d", "e"]
    );
}

#[test]
fn missing_ts_fallback_fifo() {
    let a = vec![Record::new("first", "a"), Record::new("second", "a")];
    let b = vec![Record::new("third", "b")];
    let merged: Vec<_> = merge(vec![a.into_iter(), b.into_iter()]).collect();
    assert_eq!(
        merged.iter().map(|r| r.raw.as_str()).collect::<Vec<_>>(),
        vec!["first", "second", "third"]
    );
}

#[test]
fn tie_break_stable_by_source_order() {
    let a = vec![record_with_ts(1, "from-a", "a")];
    let b = vec![record_with_ts(1, "from-b", "b")];
    let merged: Vec<_> = merge(vec![a.into_iter(), b.into_iter()]).collect();
    assert_eq!(merged.len(), 2);
    assert_eq!(merged[0].raw, "from-a");
    assert_eq!(merged[1].raw, "from-b");
}
