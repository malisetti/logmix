use std::io::{self, Write};

use crate::record::Record;

#[derive(Clone, Copy)]
pub enum Format {
    Passthrough,
    Jsonl,
    Tagged,
}

pub fn write_record<W: Write>(w: &mut W, r: &Record, format: Format) -> io::Result<()> {
    match format {
        Format::Passthrough => writeln!(w, "[{}] {}", r.source, r.raw),
        Format::Jsonl => {
            let ts = match r.ts {
                Some(t) => t.to_string(),
                None => "null".to_string(),
            };
            writeln!(
                w,
                r#"{{"source":"{}","ts":{},"raw":"{}"}}"#,
                json_escape(&r.source),
                ts,
                json_escape(&r.raw)
            )
        }
        Format::Tagged => {
            let ts = r.ts.map(|t| t.to_string()).unwrap_or_default();
            writeln!(w, "{}\t{}\t{}", r.source, ts, r.raw)
        }
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}
