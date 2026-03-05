//! Email data types for dungeon generation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Data extracted from an email.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailData {
    /// Message ID
    pub message_id: String,
    /// Sender address
    pub from: String,
    /// Sender display name
    pub from_name: Option<String>,
    /// Subject line
    pub subject: String,
    /// Received timestamp
    pub date: DateTime<Utc>,
    /// Whether the email has been read
    pub is_read: bool,
    /// Email urgency category
    pub urgency: EmailUrgency,
    /// Number of recipients (cc/to)
    pub recipient_count: u32,
    /// Whether this is a reply
    pub is_reply: bool,
    /// Thread depth (for reply chains)
    pub thread_depth: u32,
}

/// Urgency levels for emails.
/// Maps to room difficulty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EmailUrgency {
    /// Low priority - sanctuary room
    Low,
    /// Normal email - standard room
    #[default]
    Normal,
    /// Important - challenge room
    Important,
    /// Urgent/Deadline - boss room
    Urgent,
}

/// Summary of inbox state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxSummary {
    /// Total email count
    pub total: u32,
    /// Unread count (affects dungeon difficulty)
    pub unread: u32,
    /// Count by urgency
    pub urgent_count: u32,
    pub important_count: u32,
    /// Unique senders (become NPCs)
    pub unique_senders: u32,
}

/// Errors during email parsing.
#[derive(Error, Debug)]
pub enum EmailError {
    #[error("Failed to read mbox file: {0}")]
    ReadFailed(String),

    #[error("Failed to parse email: {0}")]
    ParseFailed(String),

    #[error("No emails found")]
    NoEmails,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl EmailData {
    /// Calculate email "intensity" for dungeon generation.
    /// Higher = harder room.
    pub fn intensity(&self) -> u32 {
        let mut score = 0;

        // Unread adds difficulty
        if !self.is_read {
            score += 2;
        }

        // Urgency multiplier
        score += match self.urgency {
            EmailUrgency::Low => 0,
            EmailUrgency::Normal => 1,
            EmailUrgency::Important => 3,
            EmailUrgency::Urgent => 5,
        };

        // More recipients = more complex
        score += self.recipient_count.min(3);

        // Deep threads are exhausting
        score += self.thread_depth.min(3);

        score
    }

    /// Generate an item name from subject line.
    pub fn to_item_name(&self) -> String {
        // Take first few meaningful words from subject
        let words: Vec<&str> = self
            .subject
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .take(3)
            .collect();

        if words.is_empty() {
            "Mysterious Message".to_string()
        } else {
            words.join(" ")
        }
    }

    /// Generate NPC name from sender.
    pub fn to_npc_name(&self) -> String {
        if let Some(name) = &self.from_name {
            // Take first name only
            name.split_whitespace()
                .next()
                .unwrap_or("Stranger")
                .to_string()
        } else {
            // Use part before @ in email
            self.from
                .split('@')
                .next()
                .unwrap_or("Unknown")
                .to_string()
        }
    }
}

impl EmailUrgency {
    /// Detect urgency from email headers and content.
    pub fn from_email_text(subject: &str, headers: &str) -> Self {
        let text = format!("{} {}", subject.to_lowercase(), headers.to_lowercase());

        // Check for urgent patterns
        if text.contains("urgent")
            || text.contains("asap")
            || text.contains("deadline")
            || text.contains("eod")
            || text.contains("end of day")
        {
            return EmailUrgency::Urgent;
        }

        // Important patterns
        if text.contains("important")
            || text.contains("action required")
            || text.contains("please review")
            || text.contains("x-priority: 1")
            || text.contains("x-priority: 2")
        {
            return EmailUrgency::Important;
        }

        // Low priority patterns
        if text.contains("fyi")
            || text.contains("no action")
            || text.contains("newsletter")
            || text.contains("unsubscribe")
        {
            return EmailUrgency::Low;
        }

        EmailUrgency::Normal
    }
}

impl InboxSummary {
    /// Calculate overall difficulty multiplier.
    /// 1.0 = normal, higher = harder.
    pub fn difficulty_multiplier(&self) -> f32 {
        if self.total == 0 {
            return 1.0;
        }

        let unread_ratio = self.unread as f32 / self.total as f32;
        let urgent_ratio = self.urgent_count as f32 / self.total as f32;

        // Base multiplier from unread percentage
        let base = 1.0 + (unread_ratio * 0.5);

        // Urgent emails add extra difficulty
        base + (urgent_ratio * 0.3)
    }
}
