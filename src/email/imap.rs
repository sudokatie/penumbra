//! IMAP email source for live email fetching.
//!
//! Connects to an IMAP server and fetches emails for dungeon generation.

use super::types::{EmailData, EmailError, EmailUrgency};
use chrono::{DateTime, Utc};
use native_tls::TlsConnector;
use std::net::TcpStream;

/// IMAP connection configuration
#[derive(Debug, Clone)]
pub struct ImapConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub folder: String,
    pub use_tls: bool,
}

impl Default for ImapConfig {
    fn default() -> Self {
        Self {
            host: "imap.gmail.com".to_string(),
            port: 993,
            username: String::new(),
            password: String::new(),
            folder: "INBOX".to_string(),
            use_tls: true,
        }
    }
}

/// Fetch emails from IMAP server
pub fn fetch_emails(config: &ImapConfig, limit: usize) -> Result<Vec<EmailData>, EmailError> {
    if config.username.is_empty() || config.password.is_empty() {
        return Err(EmailError::ParseFailed("Missing IMAP credentials".to_string()));
    }

    let tls = TlsConnector::builder()
        .build()
        .map_err(|e| EmailError::ParseFailed(format!("TLS error: {}", e)))?;

    let client = if config.use_tls {
        imap::connect(
            (config.host.as_str(), config.port),
            &config.host,
            &tls,
        )
        .map_err(|e| EmailError::ParseFailed(format!("Connection error: {}", e)))?
    } else {
        let stream = TcpStream::connect((config.host.as_str(), config.port))
            .map_err(|e| EmailError::ParseFailed(format!("TCP error: {}", e)))?;
        imap::Client::new(stream)
            .secure(&config.host, &tls)
            .map_err(|e| EmailError::ParseFailed(format!("TLS upgrade error: {}", e)))?
    };

    let mut session = client
        .login(&config.username, &config.password)
        .map_err(|(e, _)| EmailError::ParseFailed(format!("Login error: {}", e)))?;

    session
        .select(&config.folder)
        .map_err(|e| EmailError::ParseFailed(format!("Folder error: {}", e)))?;

    // Get recent message sequence numbers
    let search_result = session
        .search("ALL")
        .map_err(|e| EmailError::ParseFailed(format!("Search error: {}", e)))?;

    // Convert to vec and sort descending (most recent first)
    let mut seq_nums: Vec<_> = search_result.into_iter().collect();
    seq_nums.sort_by(|a, b| b.cmp(a));
    seq_nums.truncate(limit);
    
    if seq_nums.is_empty() {
        session.logout().ok();
        return Err(EmailError::NoEmails);
    }

    let seq_set = seq_nums
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let messages = session
        .fetch(&seq_set, "(ENVELOPE FLAGS)")
        .map_err(|e| EmailError::ParseFailed(format!("Fetch error: {}", e)))?;

    let mut emails = Vec::new();

    for message in messages.iter() {
        if let Some(envelope) = message.envelope() {
            let message_id = envelope
                .message_id
                .as_ref()
                .map(|id| String::from_utf8_lossy(id).to_string())
                .unwrap_or_else(|| format!("imap-{}", emails.len()));

            let (from, from_name) = extract_sender(envelope);

            let subject = envelope
                .subject
                .as_ref()
                .map(|s| String::from_utf8_lossy(s).to_string())
                .unwrap_or_else(|| "(No subject)".to_string());

            let date = envelope
                .date
                .as_ref()
                .and_then(|d| {
                    let date_str = String::from_utf8_lossy(d);
                    DateTime::parse_from_rfc2822(&date_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()
                })
                .unwrap_or_else(Utc::now);

            let is_read = message
                .flags()
                .iter()
                .any(|f| matches!(f, imap::types::Flag::Seen));

            let urgency = EmailUrgency::from_email_text(&subject, "");
            let is_reply = subject.to_lowercase().starts_with("re:");
            let thread_depth = count_reply_depth(&subject);
            let recipient_count = count_recipients(envelope);

            emails.push(EmailData {
                message_id,
                from,
                from_name,
                subject,
                date,
                is_read,
                urgency,
                recipient_count,
                is_reply,
                thread_depth,
            });
        }
    }

    session.logout().ok();
    Ok(emails)
}

/// Extract sender info from envelope
fn extract_sender(envelope: &imap_proto::types::Envelope<'_>) -> (String, Option<String>) {
    envelope
        .from
        .as_ref()
        .and_then(|addrs| addrs.first())
        .map(|addr| {
            let name = addr
                .name
                .as_ref()
                .map(|n| String::from_utf8_lossy(n).to_string());
            let mailbox = addr
                .mailbox
                .as_ref()
                .map(|m| String::from_utf8_lossy(m).to_string())
                .unwrap_or_default();
            let host = addr
                .host
                .as_ref()
                .map(|h| String::from_utf8_lossy(h).to_string())
                .unwrap_or_default();
            let email = format!("{}@{}", mailbox, host);
            (email, name)
        })
        .unwrap_or_else(|| ("unknown@unknown".to_string(), None))
}

/// Count recipients (to + cc)
fn count_recipients(envelope: &imap_proto::types::Envelope<'_>) -> u32 {
    let to_count = envelope.to.as_ref().map(|v| v.len()).unwrap_or(0);
    let cc_count = envelope.cc.as_ref().map(|v| v.len()).unwrap_or(0);
    (to_count + cc_count) as u32
}

/// Count reply depth from Re: prefixes
fn count_reply_depth(subject: &str) -> u32 {
    let lower = subject.to_lowercase();
    let mut depth = 0u32;
    let mut remaining = lower.as_str();
    
    while let Some(pos) = remaining.find("re:") {
        depth += 1;
        remaining = &remaining[pos + 3..];
    }
    
    depth
}

/// Get unread count from IMAP folder
pub fn get_unread_count(config: &ImapConfig) -> Result<usize, EmailError> {
    if config.username.is_empty() || config.password.is_empty() {
        return Err(EmailError::ParseFailed("Missing IMAP credentials".to_string()));
    }

    let tls = TlsConnector::builder()
        .build()
        .map_err(|e| EmailError::ParseFailed(format!("TLS error: {}", e)))?;

    let client = imap::connect(
        (config.host.as_str(), config.port),
        &config.host,
        &tls,
    )
    .map_err(|e| EmailError::ParseFailed(format!("Connection error: {}", e)))?;

    let mut session = client
        .login(&config.username, &config.password)
        .map_err(|(e, _)| EmailError::ParseFailed(format!("Login error: {}", e)))?;

    session
        .select(&config.folder)
        .map_err(|e| EmailError::ParseFailed(format!("Folder error: {}", e)))?;

    let unseen = session
        .search("UNSEEN")
        .map_err(|e| EmailError::ParseFailed(format!("Search error: {}", e)))?;

    let count = unseen.len();
    session.logout().ok();
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_reply_depth() {
        assert_eq!(count_reply_depth("Hello"), 0);
        assert_eq!(count_reply_depth("Re: Hello"), 1);
        assert_eq!(count_reply_depth("Re: Re: Hello"), 2);
        assert_eq!(count_reply_depth("RE: RE: RE: Thread"), 3);
    }

    #[test]
    fn test_imap_config_default() {
        let config = ImapConfig::default();
        assert_eq!(config.host, "imap.gmail.com");
        assert_eq!(config.port, 993);
        assert_eq!(config.folder, "INBOX");
        assert!(config.use_tls);
    }

    #[test]
    fn test_fetch_emails_missing_credentials() {
        let config = ImapConfig::default();
        let result = fetch_emails(&config, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_unread_missing_credentials() {
        let config = ImapConfig::default();
        let result = get_unread_count(&config);
        assert!(result.is_err());
    }
}
