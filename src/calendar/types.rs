//! Calendar data types for dungeon generation.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Data extracted from a calendar event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    /// Unique event identifier
    pub uid: String,
    /// Event start time
    pub start: DateTime<Utc>,
    /// Event end time
    pub end: DateTime<Utc>,
    /// Event summary/title
    pub summary: String,
    /// Event description (optional)
    pub description: Option<String>,
    /// Event location (optional)
    pub location: Option<String>,
    /// Event category for room type mapping
    pub category: EventCategory,
    /// Duration in minutes
    pub duration_minutes: u32,
    /// Number of attendees (affects difficulty)
    pub attendee_count: u32,
}

/// Categories for calendar events.
/// These map to room types in dungeon generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EventCategory {
    /// Regular meeting - normal room
    #[default]
    Meeting,
    /// 1:1 or small sync - sanctuary (easier)
    OneOnOne,
    /// Large meeting or all-hands - boss room
    AllHands,
    /// Focus time block - treasure room (items)
    FocusTime,
    /// Break or lunch - healing zone
    Break,
}

/// Errors that can occur during calendar parsing.
#[derive(Error, Debug)]
pub enum CalendarError {
    #[error("Failed to read calendar file: {0}")]
    ReadFailed(String),

    #[error("Failed to parse ICS content: {0}")]
    ParseFailed(String),

    #[error("No events found in last {0} days")]
    NoEvents(u32),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl EventData {
    /// Get the date portion only.
    pub fn date_naive(&self) -> NaiveDate {
        self.start.date_naive()
    }

    /// Calculate "intensity" based on duration and attendees.
    /// Used for room size and enemy count.
    pub fn intensity(&self) -> u32 {
        let base = self.duration_minutes / 15; // 1 per 15 minutes
        let attendee_bonus = self.attendee_count.saturating_sub(1); // -1 for self
        base + attendee_bonus
    }
}

impl EventCategory {
    /// Detect category from event summary and description.
    pub fn from_event_text(summary: &str, description: Option<&str>) -> Self {
        let text = format!(
            "{} {}",
            summary.to_lowercase(),
            description.unwrap_or("").to_lowercase()
        );

        // Check for specific patterns
        if text.contains("1:1") || text.contains("1-1") || text.contains("sync") {
            return EventCategory::OneOnOne;
        }
        if text.contains("all-hands")
            || text.contains("all hands")
            || text.contains("town hall")
            || text.contains("company meeting")
        {
            return EventCategory::AllHands;
        }
        if text.contains("focus") || text.contains("block") || text.contains("deep work") {
            return EventCategory::FocusTime;
        }
        if text.contains("lunch") || text.contains("break") || text.contains("coffee") {
            return EventCategory::Break;
        }

        EventCategory::Meeting
    }
}
