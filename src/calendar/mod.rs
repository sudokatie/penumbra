//! Calendar data source for dungeon generation.
//!
//! Parse ICS calendar files and convert events to dungeon elements.

pub mod parser;
pub mod types;

pub use parser::{group_by_date, parse_ics_content, parse_ics_file};
pub use types::{CalendarError, EventCategory, EventData};
