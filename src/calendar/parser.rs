//! ICS calendar file parser.

use super::types::{CalendarError, EventCategory, EventData};
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, TimeZone, Utc};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Parse an ICS file and extract events.
pub fn parse_ics_file(path: &Path, days: u32) -> Result<Vec<EventData>, CalendarError> {
    let content = fs::read_to_string(path).map_err(|e| CalendarError::ReadFailed(e.to_string()))?;
    parse_ics_content(&content, days)
}

/// Parse ICS content string and extract events.
pub fn parse_ics_content(content: &str, days: u32) -> Result<Vec<EventData>, CalendarError> {
    let cutoff = Utc::now() - Duration::days(days as i64);
    let mut events = Vec::new();
    let mut current_event: Option<EventBuilder> = None;

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("BEGIN:VEVENT") {
            current_event = Some(EventBuilder::new());
        } else if line.starts_with("END:VEVENT") {
            if let Some(builder) = current_event.take() {
                if let Some(event) = builder.build() {
                    // Only include events after cutoff
                    if event.start >= cutoff {
                        events.push(event);
                    }
                }
            }
        } else if let Some(ref mut builder) = current_event {
            builder.parse_line(line);
        }
    }

    if events.is_empty() {
        return Err(CalendarError::NoEvents(days));
    }

    // Sort by start time
    events.sort_by(|a, b| a.start.cmp(&b.start));

    Ok(events)
}

/// Group events by date for room generation.
pub fn group_by_date(events: Vec<EventData>) -> BTreeMap<NaiveDate, Vec<EventData>> {
    let mut grouped: BTreeMap<NaiveDate, Vec<EventData>> = BTreeMap::new();

    for event in events {
        let date = event.date_naive();
        grouped.entry(date).or_default().push(event);
    }

    grouped
}

/// Builder for constructing EventData from ICS lines.
#[derive(Default)]
struct EventBuilder {
    uid: Option<String>,
    dtstart: Option<DateTime<Utc>>,
    dtend: Option<DateTime<Utc>>,
    summary: Option<String>,
    description: Option<String>,
    location: Option<String>,
    attendee_count: u32,
}

impl EventBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn parse_line(&mut self, line: &str) {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.split(';').next().unwrap_or(key);

            match key {
                "UID" => self.uid = Some(value.to_string()),
                "DTSTART" => self.dtstart = parse_datetime(value),
                "DTEND" => self.dtend = parse_datetime(value),
                "SUMMARY" => self.summary = Some(unescape_ics(value)),
                "DESCRIPTION" => self.description = Some(unescape_ics(value)),
                "LOCATION" => self.location = Some(unescape_ics(value)),
                "ATTENDEE" => self.attendee_count += 1,
                _ => {}
            }
        }
    }

    fn build(self) -> Option<EventData> {
        let start = self.dtstart?;
        let end = self.dtend.unwrap_or(start + Duration::hours(1));
        let summary = self.summary.unwrap_or_else(|| "Untitled".to_string());
        let duration_minutes = (end - start).num_minutes().max(0) as u32;
        let category = EventCategory::from_event_text(&summary, self.description.as_deref());

        Some(EventData {
            uid: self.uid.unwrap_or_else(|| format!("{}", start.timestamp())),
            start,
            end,
            summary,
            description: self.description,
            location: self.location,
            category,
            duration_minutes,
            attendee_count: self.attendee_count,
        })
    }
}

/// Parse ICS datetime formats.
fn parse_datetime(value: &str) -> Option<DateTime<Utc>> {
    // Handle UTC format: 20240115T100000Z
    if value.ends_with('Z') {
        let value = value.trim_end_matches('Z');
        if let Ok(ndt) = NaiveDateTime::parse_from_str(value, "%Y%m%dT%H%M%S") {
            return Some(Utc.from_utc_datetime(&ndt));
        }
    }

    // Handle local format: 20240115T100000
    if let Ok(ndt) = NaiveDateTime::parse_from_str(value, "%Y%m%dT%H%M%S") {
        return Some(Utc.from_utc_datetime(&ndt));
    }

    // Handle date only: 20240115
    if let Ok(nd) = NaiveDate::parse_from_str(value, "%Y%m%d") {
        let ndt = nd.and_hms_opt(0, 0, 0)?;
        return Some(Utc.from_utc_datetime(&ndt));
    }

    None
}

/// Unescape ICS text values.
fn unescape_ics(value: &str) -> String {
    value
        .replace("\\n", "\n")
        .replace("\\,", ",")
        .replace("\\;", ";")
        .replace("\\\\", "\\")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_datetime_utc() {
        let dt = parse_datetime("20240115T100000Z").unwrap();
        assert_eq!(dt.format("%Y-%m-%d %H:%M").to_string(), "2024-01-15 10:00");
    }

    #[test]
    fn test_parse_datetime_local() {
        let dt = parse_datetime("20240115T143000").unwrap();
        assert_eq!(dt.format("%Y-%m-%d %H:%M").to_string(), "2024-01-15 14:30");
    }

    #[test]
    fn test_parse_datetime_date_only() {
        let dt = parse_datetime("20240115").unwrap();
        assert_eq!(dt.format("%Y-%m-%d").to_string(), "2024-01-15");
    }

    #[test]
    fn test_unescape_ics() {
        assert_eq!(unescape_ics("Hello\\nWorld"), "Hello\nWorld");
        assert_eq!(unescape_ics("A\\,B\\;C"), "A,B;C");
    }

    #[test]
    fn test_parse_simple_event() {
        let ics = r#"BEGIN:VCALENDAR
VERSION:2.0
BEGIN:VEVENT
UID:test-123
DTSTART:20260215T100000Z
DTEND:20260215T110000Z
SUMMARY:Team Meeting
END:VEVENT
END:VCALENDAR"#;

        let events = parse_ics_content(ics, 30).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].summary, "Team Meeting");
        assert_eq!(events[0].duration_minutes, 60);
    }

    #[test]
    fn test_event_category_detection() {
        assert_eq!(
            EventCategory::from_event_text("1:1 with Bob", None),
            EventCategory::OneOnOne
        );
        assert_eq!(
            EventCategory::from_event_text("All-Hands Meeting", None),
            EventCategory::AllHands
        );
        assert_eq!(
            EventCategory::from_event_text("Focus Time", None),
            EventCategory::FocusTime
        );
        assert_eq!(
            EventCategory::from_event_text("Lunch", None),
            EventCategory::Break
        );
        assert_eq!(
            EventCategory::from_event_text("Sprint Planning", None),
            EventCategory::Meeting
        );
    }

    #[test]
    fn test_event_intensity() {
        let event = EventData {
            uid: "test".to_string(),
            start: Utc::now(),
            end: Utc::now() + Duration::hours(1),
            summary: "Test".to_string(),
            description: None,
            location: None,
            category: EventCategory::Meeting,
            duration_minutes: 60,
            attendee_count: 5,
        };

        // 60 min / 15 = 4, + 4 attendees (minus self) = 8
        assert_eq!(event.intensity(), 8);
    }

    #[test]
    fn test_group_by_date() {
        let events = vec![
            EventData {
                uid: "1".to_string(),
                start: Utc.with_ymd_and_hms(2026, 2, 15, 10, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 2, 15, 11, 0, 0).unwrap(),
                summary: "Meeting 1".to_string(),
                description: None,
                location: None,
                category: EventCategory::Meeting,
                duration_minutes: 60,
                attendee_count: 2,
            },
            EventData {
                uid: "2".to_string(),
                start: Utc.with_ymd_and_hms(2026, 2, 15, 14, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 2, 15, 15, 0, 0).unwrap(),
                summary: "Meeting 2".to_string(),
                description: None,
                location: None,
                category: EventCategory::Meeting,
                duration_minutes: 60,
                attendee_count: 3,
            },
        ];

        let grouped = group_by_date(events);
        assert_eq!(grouped.len(), 1);
        let date = NaiveDate::from_ymd_opt(2026, 2, 15).unwrap();
        assert_eq!(grouped.get(&date).unwrap().len(), 2);
    }
}
