use regex::Regex;
use std::sync::OnceLock;

fn unix_secs_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\d{10}$").expect("unix seconds regex"))
}

fn unix_millis_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\d{13}$").expect("unix millis regex"))
}

fn rfc3339_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"^(?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2})T(?P<h>\d{2}):(?P<min>\d{2}):(?P<s>\d{2})(?:\.(?P<frac>\d+))?(?P<tz>Z|[+-]\d{2}:?\d{2})$",
        )
        .expect("rfc3339 regex")
    })
}

fn iso8601_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"^(?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2})[T ](?P<h>\d{2}):(?P<min>\d{2}):(?P<s>\d{2})(?:\.(?P<frac>\d+))?(?P<tz>Z|[+-]\d{2}:?\d{2})?$",
        )
        .expect("iso8601 regex")
    })
}

/// Parse a timestamp string into nanoseconds since the Unix epoch.
pub fn parse_timestamp(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    if let Some(nanos) = parse_rfc3339(s) {
        return Some(nanos);
    }
    if let Some(nanos) = parse_iso8601(s) {
        return Some(nanos);
    }
    if unix_secs_re().is_match(s) {
        return s
            .parse::<i64>()
            .ok()
            .map(|sec| sec.saturating_mul(1_000_000_000));
    }
    if unix_millis_re().is_match(s) {
        return s.parse::<i64>().ok().map(|ms| ms.saturating_mul(1_000_000));
    }
    None
}

fn parse_rfc3339(s: &str) -> Option<i64> {
    let caps = rfc3339_re().captures(s)?;
    datetime_to_nanos(&caps, true)
}

fn parse_iso8601(s: &str) -> Option<i64> {
    let caps = iso8601_re().captures(s)?;
    datetime_to_nanos(&caps, false)
}

fn datetime_to_nanos(caps: &regex::Captures<'_>, require_tz: bool) -> Option<i64> {
    let year: i32 = caps.name("y")?.as_str().parse().ok()?;
    let month: u32 = caps.name("m")?.as_str().parse().ok()?;
    let day: u32 = caps.name("d")?.as_str().parse().ok()?;
    let hour: u32 = caps.name("h")?.as_str().parse().ok()?;
    let minute: u32 = caps.name("min")?.as_str().parse().ok()?;
    let second: u32 = caps.name("s")?.as_str().parse().ok()?;

    let frac = caps.name("frac").map(|m| m.as_str()).unwrap_or("");
    let frac_nanos = parse_fraction_nanos(frac)?;

    let tz = caps.name("tz").map(|m| m.as_str());
    if require_tz && tz.is_none() {
        return None;
    }
    let offset_secs = parse_tz_offset(tz.unwrap_or("Z"))?;

    let days = days_from_civil(year, month, day)?;
    let local_secs = i64::from(days) * 86_400
        + i64::from(hour) * 3_600
        + i64::from(minute) * 60
        + i64::from(second);
    let utc_secs = local_secs - i64::from(offset_secs);
    let nanos = utc_secs
        .checked_mul(1_000_000_000)?
        .checked_add(i64::from(frac_nanos))?;
    Some(nanos)
}

fn parse_fraction_nanos(frac: &str) -> Option<u32> {
    if frac.is_empty() {
        return Some(0);
    }
    let digits = frac.len().min(9);
    let padded = format!("{:0<width$}", &frac[..digits.min(frac.len())], width = 9);
    padded.parse().ok()
}

fn parse_tz_offset(tz: &str) -> Option<i32> {
    if tz == "Z" {
        return Some(0);
    }
    let sign = match tz.as_bytes().first()? {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let rest = &tz[1..];
    let (hours, minutes) = if rest.contains(':') {
        let mut parts = rest.split(':');
        let h: i32 = parts.next()?.parse().ok()?;
        let m: i32 = parts.next()?.parse().ok()?;
        (h, m)
    } else {
        let h: i32 = rest.get(..2)?.parse().ok()?;
        let m: i32 = rest.get(2..).unwrap_or("0").parse().ok()?;
        (h, m)
    };
    Some(sign * (hours * 3_600 + minutes * 60))
}

fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i32> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y / 400 } else { (y - 399) / 400 };
    let yoe = (y - era * 400) as u32;
    let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe as i32 - 719_468)
}

#[cfg(test)]
mod unit {
    use super::parse_timestamp;

    #[test]
    fn smoke() {
        assert!(parse_timestamp("2024-01-15T10:30:00Z").is_some());
    }
}
