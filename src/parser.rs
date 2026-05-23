use crate::record::Record;
use serde_json::Value;
use std::collections::BTreeMap;

const TS_KEYS: &[&str] = &["ts", "time", "@timestamp"];

pub fn parse_jsonl(line: &str, source: &str) -> Option<Record> {
    let value: Value = serde_json::from_str(line).ok()?;
    let obj = value.as_object()?;
    let mut fields = BTreeMap::new();
    flatten_one_level(obj, &mut fields);
    let ts = ts_from_json_object(obj).or_else(|| ts_from_fields(&fields));
    Some(Record {
        ts,
        raw: line.to_string(),
        fields,
        source: source.to_string(),
    })
}

pub fn parse_logfmt(line: &str, source: &str) -> Option<Record> {
    let fields = parse_logfmt_pairs(line)?;
    if fields.is_empty() {
        return None;
    }
    let ts = ts_from_fields(&fields);
    Some(Record {
        ts,
        raw: line.to_string(),
        fields,
        source: source.to_string(),
    })
}

pub fn parse_plain(line: &str, source: &str) -> Record {
    Record::new(line, source)
}

pub fn sniff_and_parse(line: &str, source: &str) -> Record {
    if let Some(record) = parse_jsonl(line, source) {
        return record;
    }
    if let Some(record) = parse_logfmt(line, source) {
        return record;
    }
    parse_plain(line, source)
}

fn flatten_one_level(obj: &serde_json::Map<String, Value>, fields: &mut BTreeMap<String, String>) {
    for (key, value) in obj {
        match value {
            Value::Object(nested) => {
                for (nested_key, nested_value) in nested {
                    let flat_key = format!("{key}.{nested_key}");
                    if let Some(s) = json_value_to_string(nested_value) {
                        fields.insert(flat_key, s);
                    }
                }
            }
            _ => {
                if let Some(s) = json_value_to_string(value) {
                    fields.insert(key.clone(), s);
                }
            }
        }
    }
}

fn json_value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Null => None,
        Value::Array(_) | Value::Object(_) => None,
    }
}

fn ts_from_json_object(obj: &serde_json::Map<String, Value>) -> Option<i64> {
    for key in TS_KEYS {
        if let Some(value) = obj.get(*key) {
            if let Some(ts) = json_value_to_i64(value) {
                return Some(ts);
            }
        }
    }
    None
}

fn json_value_to_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Number(n) => n.as_i64().or_else(|| n.as_u64().map(|u| u as i64)),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

fn ts_from_fields(fields: &BTreeMap<String, String>) -> Option<i64> {
    for key in TS_KEYS {
        if let Some(value) = fields.get(*key) {
            if let Ok(ts) = value.parse::<i64>() {
                return Some(ts);
            }
        }
    }
    None
}

fn parse_logfmt_pairs(line: &str) -> Option<BTreeMap<String, String>> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut fields = BTreeMap::new();
    let mut i = 0;
    let bytes = trimmed.as_bytes();
    while i < bytes.len() {
        while i < bytes.len() && bytes[i] == b' ' {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let key_start = i;
        while i < bytes.len() && bytes[i] != b'=' {
            i += 1;
        }
        if i >= bytes.len() {
            return None;
        }
        let key = std::str::from_utf8(&bytes[key_start..i]).ok()?.to_string();
        i += 1; // skip '='
        let value = if i < bytes.len() && bytes[i] == b'"' {
            i += 1;
            let value_start = i;
            while i < bytes.len() && bytes[i] != b'"' {
                i += 1;
            }
            if i >= bytes.len() {
                return None;
            }
            let value = std::str::from_utf8(&bytes[value_start..i])
                .ok()?
                .to_string();
            i += 1;
            value
        } else {
            let value_start = i;
            while i < bytes.len() && bytes[i] != b' ' {
                i += 1;
            }
            std::str::from_utf8(&bytes[value_start..i])
                .ok()?
                .to_string()
        };
        fields.insert(key, value);
    }
    Some(fields)
}
