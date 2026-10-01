use chrono::{DateTime, Datelike, Utc};
use js_sys::Date;
use wasm_bindgen::JsValue;

/// Renders a UTC instant — every timestamp the api sends — in the browser's
/// own time zone as `YYYY-MM-DD HH:MM (how long ago)`, e.g.
/// `2026-01-05 08:45 (8 months ago)`.
///
/// The conversion goes through the browser's `Date` rather than a fixed
/// offset: `Date` applies the zone's full rules, DST included, to each
/// instant, so a winter timestamp keeps its winter offset instead of
/// today's.
pub fn format_local(instant: &DateTime<Utc>) -> String {
    let local = Date::new(&JsValue::from_f64(instant.timestamp_millis() as f64));

    let year = local.get_full_year();
    let month = local.get_month() + 1; // JS months are 0-based.
    let day = local.get_date();
    let hour = local.get_hours();
    let minute = local.get_minutes();

    let ago = how_long_ago(instant, &Utc::now());

    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02} ({ago})")
}

/// Human text for the distance from `instant` to `now`, e.g. `3 hours ago`.
/// The reading rounds down to the largest unit below the next one, so a
/// 90-minute-old instant is `1 hour ago`. An instant in the future — clock
/// skew between the browser and the api — reads as `just now` rather than a
/// negative distance.
fn how_long_ago(instant: &DateTime<Utc>, now: &DateTime<Utc>) -> String {
    let delta = now.signed_duration_since(*instant);

    if delta.num_seconds() < 60 {
        return "just now".to_string();
    }
    if delta.num_minutes() < 60 {
        return plural(delta.num_minutes(), "minute");
    }
    if delta.num_hours() < 24 {
        return plural(delta.num_hours(), "hour");
    }

    // Days, months, and years follow the calendar rather than fixed spans:
    // "a month old" means the same day-of-month has come around, not 30
    // days have passed. Until the first month completes, the distance stays
    // in days, however long that run of days is.
    let months = whole_months(instant, now);
    if months >= 12 {
        return plural(months / 12, "year");
    }
    if months >= 1 {
        return plural(months, "month");
    }
    plural(delta.num_days(), "day")
}

/// Whole calendar months from `earlier` to `later`: month numbers apart,
/// backed off by one while the later day-of-month has not reached the
/// earlier one yet.
fn whole_months(earlier: &DateTime<Utc>, later: &DateTime<Utc>) -> i64 {
    let months =
        (later.year() - earlier.year()) as i64 * 12 + later.month() as i64 - earlier.month() as i64;

    (months - i64::from(later.day() < earlier.day())).max(0)
}

fn plural(count: i64, unit: &str) -> String {
    if count == 1 {
        format!("{count} {unit} ago")
    } else {
        format!("{count} {unit}s ago")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn instant(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn ago(instant_text: &str, now_text: &str) -> String {
        how_long_ago(&instant(instant_text), &instant(now_text))
    }

    #[test]
    fn under_a_minute_is_just_now() {
        assert_eq!(
            ago("2026-01-05T13:45:00Z", "2026-01-05T13:45:59Z"),
            "just now"
        );
        assert_eq!(
            ago("2026-01-05T13:45:00Z", "2026-01-05T13:44:10Z"),
            "just now"
        );
    }

    #[test]
    fn minutes_hours_and_days_round_down() {
        assert_eq!(
            ago("2026-01-05T13:45:00Z", "2026-01-05T13:46:00Z"),
            "1 minute ago"
        );
        assert_eq!(
            ago("2026-01-05T13:45:00Z", "2026-01-05T15:15:00Z"),
            "1 hour ago"
        );
        assert_eq!(
            ago("2026-01-05T13:45:00Z", "2026-01-05T19:45:00Z"),
            "6 hours ago"
        );
        assert_eq!(
            ago("2026-01-01T13:45:00Z", "2026-01-04T20:00:00Z"),
            "3 days ago"
        );
    }

    #[test]
    fn months_need_the_day_of_month_to_come_around() {
        // 30 days in, but the 25th has not returned, so it is still days.
        assert_eq!(
            ago("2026-01-25T13:45:00Z", "2026-02-24T13:45:00Z"),
            "30 days ago"
        );
        assert_eq!(
            ago("2026-01-25T13:45:00Z", "2026-02-25T13:45:00Z"),
            "1 month ago"
        );
        assert_eq!(
            ago("2026-01-05T13:45:00Z", "2026-09-20T13:45:00Z"),
            "8 months ago"
        );
    }

    #[test]
    fn years_count_whole_calendar_years() {
        assert_eq!(
            ago("2025-01-05T13:45:00Z", "2026-01-04T13:45:00Z"),
            "11 months ago"
        );
        assert_eq!(
            ago("2025-01-05T13:45:00Z", "2026-01-05T13:45:00Z"),
            "1 year ago"
        );
        assert_eq!(
            ago("2023-06-01T00:00:00Z", "2026-09-30T00:00:00Z"),
            "3 years ago"
        );
    }
}
