//! Email data source for dungeon generation.
//!
//! Parses mbox files and generates dungeon elements:
//! - Unread count affects overall difficulty
//! - Senders become NPCs
//! - Subject lines become item names
//! - Urgency maps to room difficulty

mod parser;
mod types;

pub use parser::{parse_mbox_content, parse_mbox_file, summarize_inbox};
pub use types::{EmailData, EmailError, EmailUrgency, InboxSummary};
