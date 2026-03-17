//! Email data source for dungeon generation.
//!
//! Parses mbox files or fetches from IMAP servers to generate dungeon elements:
//! - Unread count affects overall difficulty
//! - Senders become NPCs
//! - Subject lines become item names
//! - Urgency maps to room difficulty

mod imap;
mod parser;
mod types;

pub use imap::{fetch_emails, get_unread_count, ImapConfig};
pub use parser::{parse_mbox_content, parse_mbox_file, summarize_inbox};
pub use types::{EmailData, EmailError, EmailUrgency, InboxSummary};
