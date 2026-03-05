//! Mbox email parser for penumbra dungeon generation.

use super::types::{EmailData, EmailError, EmailUrgency, InboxSummary};
use chrono::{DateTime, Utc};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Parse an mbox file and extract emails.
pub fn parse_mbox_file(path: &Path) -> Result<Vec<EmailData>, EmailError> {
    let content = fs::read_to_string(path).map_err(|e| EmailError::ReadFailed(e.to_string()))?;
    parse_mbox_content(&content)
}

/// Parse mbox content string.
pub fn parse_mbox_content(content: &str) -> Result<Vec<EmailData>, EmailError> {
    let mut emails = Vec::new();
    let mut current_email = String::new();

    for line in content.lines() {
        if line.starts_with("From ") && !current_email.is_empty() {
            // Parse the accumulated email
            if let Some(email) = parse_single_email(&current_email) {
                emails.push(email);
            }
            current_email.clear();
        }
        current_email.push_str(line);
        current_email.push('\n');
    }

    // Parse final email
    if !current_email.is_empty() {
        if let Some(email) = parse_single_email(&current_email) {
            emails.push(email);
        }
    }

    if emails.is_empty() {
        return Err(EmailError::NoEmails);
    }

    Ok(emails)
}

/// Parse a single email message.
fn parse_single_email(content: &str) -> Option<EmailData> {
    let mut message_id = String::new();
    let mut from = String::new();
    let mut from_name = None;
    let mut subject = String::new();
    let mut date: Option<DateTime<Utc>> = None;
    let mut is_read = true; // Assume read unless marked unread
    let mut recipient_count = 1u32;
    let mut is_reply = false;
    let mut thread_depth = 0u32;

    let mut in_headers = true;
    let mut headers_text = String::new();

    for line in content.lines() {
        if in_headers {
            if line.is_empty() {
                in_headers = false;
                continue;
            }
            headers_text.push_str(line);
            headers_text.push('\n');

            let lower = line.to_lowercase();

            // Parse headers
            if lower.starts_with("message-id:") {
                message_id = extract_header_value(line);
            } else if lower.starts_with("from:") {
                let value = extract_header_value(line);
                let (name, email) = parse_from_header(&value);
                from = email;
                from_name = name;
            } else if lower.starts_with("subject:") {
                subject = extract_header_value(line);
                // Check for Re: prefix
                if subject.to_lowercase().starts_with("re:") {
                    is_reply = true;
                    // Count Re: depth
                    thread_depth = subject
                        .to_lowercase()
                        .matches("re:")
                        .count()
                        .min(10) as u32;
                }
            } else if lower.starts_with("date:") {
                date = parse_date_header(&extract_header_value(line));
            } else if lower.starts_with("to:") || lower.starts_with("cc:") {
                // Count recipients
                let value = extract_header_value(line);
                recipient_count += value.matches('@').count() as u32;
            } else if lower.starts_with("x-mozilla-status:") || lower.starts_with("status:") {
                // Check for unread status
                let value = lower.clone();
                if value.contains("0000") || value.contains("u") {
                    is_read = false;
                }
            }
        }
    }

    // Generate message ID if missing
    if message_id.is_empty() {
        message_id = format!("gen-{}", subject.len());
    }

    // Use current time if date parsing failed
    let date = date.unwrap_or_else(Utc::now);

    // Determine urgency
    let urgency = EmailUrgency::from_email_text(&subject, &headers_text);

    Some(EmailData {
        message_id,
        from,
        from_name,
        subject,
        date,
        is_read,
        urgency,
        recipient_count: recipient_count.min(20),
        is_reply,
        thread_depth,
    })
}

/// Extract header value after the colon.
fn extract_header_value(line: &str) -> String {
    if let Some(pos) = line.find(':') {
        line[pos + 1..].trim().to_string()
    } else {
        line.to_string()
    }
}

/// Parse From header to extract name and email.
fn parse_from_header(value: &str) -> (Option<String>, String) {
    // Format: "Name <email@example.com>" or just "email@example.com"
    if let Some(start) = value.find('<') {
        if let Some(end) = value.find('>') {
            let email = value[start + 1..end].trim().to_string();
            let name = value[..start].trim().trim_matches('"').to_string();
            let name = if name.is_empty() { None } else { Some(name) };
            return (name, email);
        }
    }
    (None, value.trim().to_string())
}

/// Parse email date header to DateTime.
fn parse_date_header(value: &str) -> Option<DateTime<Utc>> {
    // Try common formats
    let formats = [
        "%a, %d %b %Y %H:%M:%S %z",
        "%d %b %Y %H:%M:%S %z",
        "%a, %d %b %Y %H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
    ];

    for fmt in formats {
        if let Ok(dt) = chrono::DateTime::parse_from_str(value.trim(), fmt) {
            return Some(dt.with_timezone(&Utc));
        }
    }

    // Try without timezone
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(value.trim(), "%a, %d %b %Y %H:%M:%S")
    {
        return Some(DateTime::from_naive_utc_and_offset(naive, Utc));
    }

    None
}

/// Generate inbox summary from emails.
pub fn summarize_inbox(emails: &[EmailData]) -> InboxSummary {
    let total = emails.len() as u32;
    let unread = emails.iter().filter(|e| !e.is_read).count() as u32;
    let urgent_count = emails
        .iter()
        .filter(|e| e.urgency == EmailUrgency::Urgent)
        .count() as u32;
    let important_count = emails
        .iter()
        .filter(|e| e.urgency == EmailUrgency::Important)
        .count() as u32;

    let unique_senders: HashSet<_> = emails.iter().map(|e| &e.from).collect();

    InboxSummary {
        total,
        unread,
        urgent_count,
        important_count,
        unique_senders: unique_senders.len() as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_MBOX: &str = r#"From test@example.com Mon Jan 15 10:00:00 2024
Message-ID: <123@example.com>
From: John Doe <john@example.com>
To: jane@example.com
Subject: Re: Re: Important meeting notes
Date: Mon, 15 Jan 2024 10:00:00 -0500
Status: RO

This is the email body.

From urgent@example.com Mon Jan 15 11:00:00 2024
Message-ID: <456@example.com>
From: Boss <boss@example.com>
To: team@example.com
Cc: manager@example.com, hr@example.com
Subject: URGENT: Deadline today
Date: Mon, 15 Jan 2024 11:00:00 -0500
X-Mozilla-Status: 0000

Urgent email body.
"#;

    #[test]
    fn test_parse_mbox_content() {
        let emails = parse_mbox_content(SAMPLE_MBOX).unwrap();
        assert_eq!(emails.len(), 2);
    }

    #[test]
    fn test_parse_from_header() {
        let (name, email) = parse_from_header("John Doe <john@example.com>");
        assert_eq!(name, Some("John Doe".to_string()));
        assert_eq!(email, "john@example.com");
    }

    #[test]
    fn test_detect_reply() {
        let emails = parse_mbox_content(SAMPLE_MBOX).unwrap();
        assert!(emails[0].is_reply);
        assert_eq!(emails[0].thread_depth, 2);
    }

    #[test]
    fn test_detect_urgency() {
        let emails = parse_mbox_content(SAMPLE_MBOX).unwrap();
        assert_eq!(emails[1].urgency, EmailUrgency::Urgent);
    }

    #[test]
    fn test_detect_unread() {
        let emails = parse_mbox_content(SAMPLE_MBOX).unwrap();
        // Status: RO contains "O" (old), not matching our patterns
        // X-Mozilla-Status: 0000 = unread
        assert!(!emails[1].is_read); // X-Mozilla-Status: 0000 = unread
    }

    #[test]
    fn test_recipient_count() {
        let emails = parse_mbox_content(SAMPLE_MBOX).unwrap();
        assert_eq!(emails[1].recipient_count, 4); // to + 2 cc + base
    }

    #[test]
    fn test_summarize_inbox() {
        let emails = parse_mbox_content(SAMPLE_MBOX).unwrap();
        let summary = summarize_inbox(&emails);
        assert_eq!(summary.total, 2);
        // At least one should be unread
        assert!(summary.unread >= 1);
        assert_eq!(summary.urgent_count, 1);
        assert_eq!(summary.unique_senders, 2);
    }

    #[test]
    fn test_intensity() {
        let emails = parse_mbox_content(SAMPLE_MBOX).unwrap();
        // Urgent unread email should have high intensity
        assert!(emails[1].intensity() > emails[0].intensity());
    }

    #[test]
    fn test_to_item_name() {
        let emails = parse_mbox_content(SAMPLE_MBOX).unwrap();
        let item = emails[1].to_item_name();
        assert!(!item.is_empty());
        assert!(item.contains("URGENT") || item.contains("Deadline"));
    }

    #[test]
    fn test_to_npc_name() {
        let emails = parse_mbox_content(SAMPLE_MBOX).unwrap();
        assert_eq!(emails[0].to_npc_name(), "John");
        assert_eq!(emails[1].to_npc_name(), "Boss");
    }

    #[test]
    fn test_difficulty_multiplier() {
        let emails = parse_mbox_content(SAMPLE_MBOX).unwrap();
        let summary = summarize_inbox(&emails);
        // With 1 unread out of 2, and 1 urgent, should be > 1.0
        assert!(summary.difficulty_multiplier() > 1.0);
    }

    #[test]
    fn test_empty_mbox() {
        let result = parse_mbox_content("");
        assert!(matches!(result, Err(EmailError::NoEmails)));
    }
}
