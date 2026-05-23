use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn bin() -> Command {
    Command::cargo_bin("logmix").unwrap()
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn merge_two_jsonl_files_by_timestamp() {
    let output = bin()
        .args([
            fixture("a.jsonl").to_str().unwrap(),
            fixture("b.jsonl").to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = std::str::from_utf8(&output).unwrap();
    assert!(
        text.contains(r#"[b.jsonl] {"msg":"a","ts":1}"#)
            && text.contains(r#"[a.jsonl] {"msg":"b","ts":2}"#)
            && text.contains(r#"[b.jsonl] {"msg":"c","ts":3}"#)
            && text.contains(r#"[b.jsonl] {"msg":"d","ts":4}"#)
            && text.contains(r#"[a.jsonl] {"msg":"e","ts":5}"#)
    );
    insta::assert_snapshot!("merge_two_jsonl", text);
}

#[test]
fn merge_jsonl_and_logfmt_with_source_tags() {
    insta::assert_snapshot!(
        "merge_jsonl_logfmt",
        std::str::from_utf8(
            &bin()
                .args([
                    fixture("app.jsonl").to_str().unwrap(),
                    fixture("svc.logfmt").to_str().unwrap(),
                ])
                .assert()
                .success()
                .get_output()
                .stdout
        )
        .unwrap()
    );
}

#[test]
fn missing_ts_records_fifo_position() {
    insta::assert_snapshot!(
        "missing_ts_fifo",
        std::str::from_utf8(
            &bin()
                .args([
                    fixture("no_ts_a.log").to_str().unwrap(),
                    fixture("no_ts_b.log").to_str().unwrap(),
                ])
                .assert()
                .success()
                .get_output()
                .stdout
        )
        .unwrap()
    );
}

#[test]
fn format_jsonl_each_line_is_valid_json() {
    let output = bin()
        .args(["--format", "jsonl"])
        .args([
            fixture("a.jsonl").to_str().unwrap(),
            fixture("b.jsonl").to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = std::str::from_utf8(&output).unwrap();
    for line in text.lines() {
        let value: Value = serde_json::from_str(line).expect("valid JSON line");
        assert!(value.get("source").is_some());
        assert!(value.get("raw").is_some());
    }

    insta::assert_snapshot!("format_jsonl", text);
}

#[test]
fn missing_file_exits_with_error() {
    bin()
        .arg("tests/fixtures/does-not-exist.log")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("cannot open"));
}
