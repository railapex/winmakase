//! Timestamps without a date crate.
//!
//! Health state and log lines carry ISO-8601 UTC (`2026-08-28T16:40:00Z`).
//! UTC rather than local time so a log read months later, or on another
//! machine, means one unambiguous thing. `winsome status` converts to
//! human-scale durations at the point of display.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Format a [`SystemTime`] as `YYYY-MM-DDTHH:MM:SSZ`.
///
/// Times before the Unix epoch clamp to the epoch; a desktop supervisor has no
/// business printing 1969, and the alternative is an error branch nobody reads.
pub fn iso8601(t: SystemTime) -> String {
    let secs = t
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    format_epoch_secs(secs)
}

pub fn now_iso8601() -> String {
    iso8601(SystemTime::now())
}

fn format_epoch_secs(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    let (hh, mm, ss) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

/// Parse the format [`iso8601`] writes. Deliberately strict — this reads our
/// own state file, not arbitrary input, and a sloppy parser would turn a
/// corrupt health file into a plausible-looking uptime.
pub fn parse_iso8601(s: &str) -> Option<SystemTime> {
    let b = s.as_bytes();
    if b.len() != 20
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
        || b[19] != b'Z'
    {
        return None;
    }
    let num = |from: usize, to: usize| s.get(from..to)?.parse::<i64>().ok();
    let (y, mo, d) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    let (h, mi, se) = (num(11, 13)?, num(14, 16)?, num(17, 19)?);
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) || h > 23 || mi > 59 || se > 59 {
        return None;
    }
    let secs = days_from_civil(y, mo as u32, d as u32) * 86_400 + h * 3600 + mi * 60 + se;
    if secs < 0 {
        return None;
    }
    let t = UNIX_EPOCH + Duration::from_secs(secs as u64);
    // The civil-calendar math normalises rather than rejects: Feb 30 comes back
    // as Mar 2, April 31 as May 1. Round-tripping is the cheapest way to hold
    // the strictness this parser claims — if it does not format back to what
    // was handed in, it was not a real date.
    (iso8601(t) == s).then_some(t)
}

/// How long ago `stamp` was, or `None` if it is unparseable or in the future.
pub fn elapsed_since(stamp: &str) -> Option<Duration> {
    SystemTime::now().duration_since(parse_iso8601(stamp)?).ok()
}

/// `"4m 12s"`, `"2h 05m"`, `"3d 4h"` — for humans reading `winsome status`.
pub fn human_duration(d: Duration) -> String {
    let s = d.as_secs();
    match s {
        0..60 => format!("{s}s"),
        60..3600 => format!("{}m {:02}s", s / 60, s % 60),
        3600..86_400 => format!("{}h {:02}m", s / 3600, (s % 3600) / 60),
        _ => format!("{}d {}h", s / 86_400, (s % 86_400) / 3600),
    }
}

// Howard Hinnant's civil-calendar algorithms (public domain), the standard
// no-dependency answer for days <-> y/m/d.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (y + i64::from(m <= 2), m as u32, d as u32)
}

fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = y - i64::from(m <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = i64::from(m) + if m > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_known_instants() {
        assert_eq!(format_epoch_secs(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_epoch_secs(1_000_000_000), "2001-09-09T01:46:40Z");
        // 2026-08-28T16:40:00Z
        assert_eq!(format_epoch_secs(1_787_935_200), "2026-08-28T16:40:00Z");
    }

    #[test]
    fn handles_leap_day() {
        let stamp = "2028-02-29T12:00:00Z";
        assert_eq!(iso8601(parse_iso8601(stamp).unwrap()), stamp);
    }

    #[test]
    fn rejects_days_that_month_does_not_have() {
        // These all parse arithmetically and normalise into the next month.
        // Silently accepting them would make the parser a date generator.
        for bad in [
            "2026-02-30T00:00:00Z",
            "2026-02-29T00:00:00Z", // 2026 is not a leap year
            "2100-02-29T00:00:00Z", // century, not a leap year
            "2026-04-31T00:00:00Z",
            "2026-06-31T00:00:00Z",
            "2026-09-31T00:00:00Z",
            "2026-11-31T00:00:00Z",
        ] {
            assert!(parse_iso8601(bad).is_none(), "should have rejected {bad}");
        }
        // The valid neighbours still parse.
        for good in [
            "2024-02-29T00:00:00Z", // leap year
            "2000-02-29T00:00:00Z", // 400-year leap century
            "2026-02-28T00:00:00Z",
            "2026-04-30T00:00:00Z",
            "2026-12-31T23:59:59Z",
        ] {
            assert_eq!(iso8601(parse_iso8601(good).unwrap()), good);
        }
    }

    #[test]
    fn rejects_leap_seconds() {
        // We never write :60, so accepting it could only smuggle in a
        // timestamp that rolls over to the next minute.
        assert!(parse_iso8601("2026-06-30T23:59:60Z").is_none());
    }

    #[test]
    fn round_trips_across_a_decade() {
        // Every ~19 hours for ten years: catches month-boundary and leap-year drift.
        let mut secs = 1_700_000_000i64;
        while secs < 2_015_000_000 {
            let s = format_epoch_secs(secs);
            let back = parse_iso8601(&s).expect("our own format must parse");
            assert_eq!(iso8601(back), s);
            secs += 68_400;
        }
    }

    #[test]
    fn rejects_malformed_stamps() {
        for bad in [
            "",
            "2026-08-28",
            "2026-08-28T16:40:00",
            "2026-08-28 16:40:00Z",
            "2026-13-01T00:00:00Z",
            "2026-00-01T00:00:00Z",
            "2026-08-32T00:00:00Z",
            "2026-08-28T24:00:00Z",
            "not-a-timestamp-at-a",
        ] {
            assert!(parse_iso8601(bad).is_none(), "should have rejected {bad:?}");
        }
    }

    #[test]
    fn durations_read_like_english() {
        assert_eq!(human_duration(Duration::from_secs(0)), "0s");
        assert_eq!(human_duration(Duration::from_secs(59)), "59s");
        assert_eq!(human_duration(Duration::from_secs(252)), "4m 12s");
        assert_eq!(human_duration(Duration::from_secs(7500)), "2h 05m");
        assert_eq!(human_duration(Duration::from_secs(273_600)), "3d 4h");
    }
}
