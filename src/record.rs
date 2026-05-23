use std::collections::BTreeMap;

pub struct Record {
    pub ts: Option<i64>,
    pub raw: String,
    pub fields: BTreeMap<String, String>,
    pub source: String,
}

impl Record {
    pub fn new(raw: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            ts: None,
            raw: raw.into(),
            fields: BTreeMap::new(),
            source: source.into(),
        }
    }
}
