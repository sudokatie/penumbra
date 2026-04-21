//! Dungeon generation from git, calendar, email, and weather data.

use chrono::NaiveDate;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::calendar::{EventCategory, EventData};
use crate::git::CommitData;
use crate::weather::{DungeonAtmosphere, WeatherCondition, WeatherData};

use super::{Room, RoomType, Tile, World};

/// Generate a complete dungeon from git commit data.
pub fn generate_dungeon(git_data: &[CommitData], seed: u64) -> World {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let grouped = crate::git::group_by_date(git_data.to_vec());

    let mut rooms = Vec::new();

    for (index, (date, commits)) in grouped.iter().enumerate() {
        let room = generate_room(*date, commits, index, &mut rng);
        rooms.push(room);
    }

    // Place connections between rooms
    place_connections(&mut rooms);

    World::new(rooms)
}

/// Generate a single room from a day's commits.
pub fn generate_room(
    date: NaiveDate,
    commits: &[CommitData],
    index: usize,
    rng: &mut impl Rng,
) -> Room {
    let total_lines: u32 = commits.iter().map(|c| c.lines_changed()).sum();
    let (width, height) = calculate_room_size(total_lines);
    let room_type = determine_room_type(commits);

    let mut room = Room::new(index, width, height, room_type, date);
    room.source_commits = commits.to_vec();

    generate_layout(&mut room, rng);

    room
}

/// Calculate room dimensions from total lines changed.
/// Spec: Room size 3x3 to 9x9 based on lines changed.
pub fn calculate_room_size(total_lines: u32) -> (u8, u8) {
    match total_lines {
        0..=19 => (3, 3),
        20..=49 => (5, 5),
        50..=199 => (7, 7),
        _ => (9, 9), // Max 9x9 per spec
    }
}

/// Determine room type from commit data.
pub fn determine_room_type(commits: &[CommitData]) -> RoomType {
    // Merge commits always create boss rooms
    if commits.iter().any(|c| c.is_merge) {
        return RoomType::Boss;
    }

    // Aggregate file categories across all commits
    let mut total_test_files = 0u32;
    let mut total_config_files = 0u32;
    let mut total_doc_files = 0u32;
    let mut total_other_files = 0u32;

    for commit in commits {
        total_test_files += commit.file_categories.test_files;
        total_config_files += commit.file_categories.config_files;
        total_doc_files += commit.file_categories.doc_files;
        total_other_files += commit.file_categories.other_files;
    }

    let total_files = total_test_files + total_config_files + total_doc_files + total_other_files;

    if total_files > 0 {
        // Test-heavy rooms become sanctuaries (safe resting areas)
        if total_test_files * 2 > total_files {
            return RoomType::Sanctuary;
        }
        // Config-heavy rooms become treasure rooms
        if total_config_files * 2 > total_files {
            return RoomType::Treasure;
        }
        // Doc-heavy rooms become libraries (knowledge/hints)
        if total_doc_files * 2 > total_files {
            return RoomType::Library;
        }
    }

    // Fall back to message-based heuristics for commits with no file data
    let test_commits = commits
        .iter()
        .filter(|c| {
            let msg = c.message.to_lowercase();
            msg.contains("test") || msg.contains("spec")
        })
        .count();

    let config_commits = commits
        .iter()
        .filter(|c| {
            let msg = c.message.to_lowercase();
            msg.contains("config") || msg.contains("setting")
        })
        .count();

    let total = commits.len();
    if total > 0 && total_files == 0 {
        if test_commits * 2 > total {
            return RoomType::Sanctuary;
        }
        if config_commits * 2 > total {
            return RoomType::Treasure;
        }
    }

    RoomType::Normal
}

/// Generate the room layout (walls, doors).
fn generate_layout(room: &mut Room, _rng: &mut impl Rng) {
    let w = room.width as i32;
    let h = room.height as i32;

    // Add perimeter walls
    for x in 0..w {
        room.set_tile(x, 0, Tile::Wall);
        room.set_tile(x, h - 1, Tile::Wall);
    }
    for y in 0..h {
        room.set_tile(0, y, Tile::Wall);
        room.set_tile(w - 1, y, Tile::Wall);
    }

    // Sanctuary rooms have healing zone tiles on floor
    if room.room_type == RoomType::Sanctuary {
        for y in 1..(h - 1) {
            for x in 1..(w - 1) {
                room.set_tile(x, y, Tile::HealingZone);
            }
        }
    }

    // Boss rooms are larger with clear center
    if room.room_type == RoomType::Boss && room.width >= 9 {
        // Add some internal structure later
    }
}

/// Place entrance and exit doors connecting rooms.
pub fn place_connections(rooms: &mut [Room]) {
    let room_count = rooms.len();
    for (i, room) in rooms.iter_mut().enumerate() {
        let mid_y = room.height as i32 / 2;

        // First room: entrance only on west
        // Middle rooms: entrance on west, exit on east
        // Last room: entrance on west, no exit

        if i > 0 {
            // Has entrance from previous room
            room.set_tile(0, mid_y, Tile::Entrance);
        }

        if i < room_count - 1 {
            // Has exit to next room
            room.set_tile(room.width as i32 - 1, mid_y, Tile::Exit);
        }
    }
}

// ============================================================================
// Calendar-based dungeon generation
// ============================================================================

/// Generate a complete dungeon from calendar event data.
pub fn generate_dungeon_from_calendar(events: &[EventData], seed: u64) -> World {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let grouped = crate::calendar::group_by_date(events.to_vec());

    let mut rooms = Vec::new();

    for (index, (date, day_events)) in grouped.iter().enumerate() {
        let room = generate_room_from_events(*date, day_events, index, &mut rng);
        rooms.push(room);
    }

    // Place connections between rooms
    place_connections(&mut rooms);

    World::new(rooms)
}

/// Generate a single room from a day's calendar events.
pub fn generate_room_from_events(
    date: NaiveDate,
    events: &[EventData],
    index: usize,
    rng: &mut impl Rng,
) -> Room {
    // Calculate total intensity (duration + attendees)
    let total_intensity: u32 = events.iter().map(|e| e.intensity()).sum();
    let (width, height) = calculate_room_size_from_intensity(total_intensity);
    let room_type = determine_room_type_from_events(events);

    let mut room = Room::new(index, width, height, room_type, date);

    generate_layout(&mut room, rng);

    room
}

/// Calculate room dimensions from total event intensity.
/// Higher intensity (longer/busier meetings) = larger rooms.
pub fn calculate_room_size_from_intensity(intensity: u32) -> (u8, u8) {
    match intensity {
        0..=4 => (3, 3),    // Light day
        5..=10 => (5, 5),   // Normal day
        11..=20 => (7, 7),  // Busy day
        _ => (9, 9),        // Packed day
    }
}

/// Determine room type from calendar events.
pub fn determine_room_type_from_events(events: &[EventData]) -> RoomType {
    // Check for all-hands or large meetings (boss room)
    if events
        .iter()
        .any(|e| e.category == EventCategory::AllHands || e.attendee_count >= 10)
    {
        return RoomType::Boss;
    }

    // Count categories
    let mut one_on_one = 0;
    let mut focus_time = 0;
    let mut breaks = 0;

    for event in events {
        match event.category {
            EventCategory::OneOnOne => one_on_one += 1,
            EventCategory::FocusTime => focus_time += 1,
            EventCategory::Break => breaks += 1,
            EventCategory::Meeting | EventCategory::AllHands => {} // Normal meetings, all-hands handled above
        }
    }

    let total = events.len();
    if total == 0 {
        return RoomType::Normal;
    }

    // Focus time or break-heavy days = Sanctuary (restorative)
    if (focus_time + breaks) * 2 > total {
        return RoomType::Sanctuary;
    }

    // 1:1 heavy days = Treasure (networking/knowledge gain)
    if one_on_one * 2 > total {
        return RoomType::Treasure;
    }

    RoomType::Normal
}

// ============================================================================
// Email-based dungeon generation
// ============================================================================

/// Generate a complete dungeon from email data.
pub fn generate_dungeon_from_email(emails: &[crate::email::EmailData], seed: u64) -> World {
    use std::collections::HashMap;
    
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    
    // Group emails by date
    let mut grouped: HashMap<NaiveDate, Vec<&crate::email::EmailData>> = HashMap::new();
    for email in emails {
        let date = email.date.date_naive();
        grouped.entry(date).or_default().push(email);
    }
    
    // Sort by date
    let mut dates: Vec<_> = grouped.keys().copied().collect();
    dates.sort();
    
    let mut rooms = Vec::new();
    
    for (index, date) in dates.iter().enumerate() {
        let day_emails = &grouped[date];
        let room = generate_room_from_emails(date, day_emails, index, &mut rng);
        rooms.push(room);
    }
    
    // Place connections between rooms
    place_connections(&mut rooms);
    
    World::new(rooms)
}

/// Generate a single room from a day's emails.
pub fn generate_room_from_emails(
    date: &NaiveDate,
    emails: &[&crate::email::EmailData],
    index: usize,
    rng: &mut impl Rng,
) -> Room {
    // Calculate total intensity from emails
    let total_intensity: u32 = emails.iter().map(|e| e.intensity()).sum();
    let (width, height) = calculate_room_size_from_intensity(total_intensity);
    let room_type = determine_room_type_from_emails(emails);
    
    let mut room = Room::new(index, width, height, room_type, *date);
    
    generate_layout(&mut room, rng);
    
    room
}

/// Determine room type from email data.
pub fn determine_room_type_from_emails(emails: &[&crate::email::EmailData]) -> RoomType {
    use crate::email::EmailUrgency;
    
    // Urgent emails = boss room
    if emails.iter().any(|e| e.urgency == EmailUrgency::Urgent) {
        return RoomType::Boss;
    }
    
    // Count categories
    let mut low_priority = 0;
    let mut important = 0;
    
    for email in emails {
        match email.urgency {
            EmailUrgency::Low => low_priority += 1,
            EmailUrgency::Important => important += 1,
            _ => {}
        }
    }
    
    let total = emails.len();
    if total == 0 {
        return RoomType::Normal;
    }
    
    // Low priority / newsletter heavy = Sanctuary (light reading)
    if low_priority * 2 > total {
        return RoomType::Sanctuary;
    }
    
    // Important emails = Treasure (valuable info)
    if important * 2 > total {
        return RoomType::Treasure;
    }
    
    RoomType::Normal
}

// ============================================================================
// Weather-based dungeon generation
// ============================================================================

/// Generate a complete dungeon from weather data.
/// Weather creates a unique single-session dungeon with atmosphere modifiers.
pub fn generate_dungeon_from_weather(weather: &WeatherData, seed: u64) -> World {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let atmosphere = crate::weather::generate_atmosphere(weather);

    // Weather dungeons have 5-10 rooms based on difficulty
    let base_rooms = 5;
    let extra_rooms = (weather.difficulty_multiplier() * 3.0) as usize;
    let room_count = (base_rooms + extra_rooms).min(10);

    let mut rooms = Vec::new();
    let today = chrono::Utc::now().date_naive();

    for index in 0..room_count {
        let room = generate_room_from_weather(today, weather, &atmosphere, index, &mut rng);
        rooms.push(room);
    }

    // Place connections between rooms
    place_connections(&mut rooms);

    World::new(rooms)
}

/// Generate a single room based on weather conditions.
pub fn generate_room_from_weather(
    date: NaiveDate,
    weather: &WeatherData,
    atmosphere: &DungeonAtmosphere,
    index: usize,
    rng: &mut impl Rng,
) -> Room {
    // Use weather intensity for room size
    let intensity = weather.intensity();
    let (width, height) = calculate_room_size_from_intensity(intensity);

    // Determine room type from weather
    let room_type = determine_room_type_from_weather(weather, atmosphere, index, rng);

    let mut room = Room::new(index, width, height, room_type, date);

    generate_layout(&mut room, rng);

    room
}

/// Determine room type from weather conditions.
pub fn determine_room_type_from_weather(
    weather: &WeatherData,
    atmosphere: &DungeonAtmosphere,
    index: usize,
    rng: &mut impl Rng,
) -> RoomType {
    // Last room is always boss for storm/hail
    if index >= 4 && matches!(weather.condition, WeatherCondition::Storm | WeatherCondition::Hail) {
        return RoomType::Boss;
    }

    // Use weather condition bias with some randomness
    let bias = weather.condition.room_type_bias();
    let roll: f64 = rng.gen();

    // 40% chance to use weather bias, 60% varied based on atmosphere
    if roll < 0.4 {
        return bias;
    }

    match atmosphere {
        DungeonAtmosphere::Bright => {
            // Bright days have more treasures
            if roll < 0.6 { RoomType::Treasure } else { RoomType::Normal }
        }
        DungeonAtmosphere::Misty => {
            // Misty dungeons have libraries (mystery/knowledge)
            if roll < 0.6 { RoomType::Library } else { RoomType::Normal }
        }
        DungeonAtmosphere::Frozen => {
            // Frozen dungeons have sanctuaries (shelter)
            if roll < 0.6 { RoomType::Sanctuary } else { RoomType::Normal }
        }
        DungeonAtmosphere::Tempestuous => {
            // Tempestuous dungeons have boss encounters
            if roll < 0.55 { RoomType::Boss } else { RoomType::Normal }
        }
        DungeonAtmosphere::Dark | DungeonAtmosphere::Dim => {
            RoomType::Normal
        }
    }
}

#[cfg(test)]
mod weather_tests {
    use super::*;
    use chrono::Utc;

    fn make_weather(condition: WeatherCondition, temp: f64, humidity: u8, wind: f64) -> WeatherData {
        WeatherData {
            condition,
            temperature_c: temp,
            humidity,
            wind_speed_kph: wind,
            description: "Test".to_string(),
            location: "Test City".to_string(),
            fetched_at: Utc::now(),
        }
    }

    #[test]
    fn test_generate_dungeon_from_weather() {
        let weather = make_weather(WeatherCondition::Clear, 20.0, 50, 10.0);
        let world = generate_dungeon_from_weather(&weather, 42);

        // Should generate at least 5 rooms
        assert!(world.rooms.len() >= 5);
        // Should generate at most 10 rooms
        assert!(world.rooms.len() <= 10);
    }

    #[test]
    fn test_storm_creates_more_rooms() {
        let clear = make_weather(WeatherCondition::Clear, 20.0, 50, 10.0);
        let storm = make_weather(WeatherCondition::Storm, 20.0, 90, 60.0);

        let clear_world = generate_dungeon_from_weather(&clear, 42);
        let storm_world = generate_dungeon_from_weather(&storm, 42);

        // Storm should create more rooms due to higher difficulty
        assert!(storm_world.rooms.len() >= clear_world.rooms.len());
    }

    #[test]
    fn test_extreme_weather_has_boss_room() {
        let weather = make_weather(WeatherCondition::Storm, 35.0, 95, 70.0);
        let world = generate_dungeon_from_weather(&weather, 42);

        // Should have at least one boss room
        let boss_count = world.rooms.iter().filter(|r| r.room_type == RoomType::Boss).count();
        assert!(boss_count > 0);
    }

    #[test]
    fn test_weather_room_type_bias() {
        // Storm biases toward Boss
        assert_eq!(WeatherCondition::Storm.room_type_bias(), RoomType::Boss);
        // Fog biases toward Library
        assert_eq!(WeatherCondition::Fog.room_type_bias(), RoomType::Library);
        // Snow biases toward Sanctuary
        assert_eq!(WeatherCondition::Snow.room_type_bias(), RoomType::Sanctuary);
    }

    #[test]
    fn test_weather_intensity_affects_room_size() {
        let mild = make_weather(WeatherCondition::Clear, 20.0, 50, 5.0);
        let severe = make_weather(WeatherCondition::Storm, -5.0, 90, 60.0);

        // Severe weather should have higher intensity
        assert!(severe.intensity() > mild.intensity());
    }
}

#[cfg(test)]
mod email_tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use crate::email::{EmailData, EmailUrgency};
    
    fn make_email(subject: &str, urgency: EmailUrgency, is_read: bool) -> EmailData {
        EmailData {
            message_id: subject.to_string(),
            from: "test@example.com".to_string(),
            from_name: Some("Test User".to_string()),
            subject: subject.to_string(),
            date: Utc.with_ymd_and_hms(2026, 2, 15, 10, 0, 0).unwrap(),
            is_read,
            urgency,
            recipient_count: 1,
            is_reply: false,
            thread_depth: 0,
        }
    }
    
    #[test]
    fn test_urgent_email_creates_boss_room() {
        let emails = vec![make_email("URGENT: Server down", EmailUrgency::Urgent, false)];
        let refs: Vec<_> = emails.iter().collect();
        assert_eq!(determine_room_type_from_emails(&refs), RoomType::Boss);
    }
    
    #[test]
    fn test_low_priority_creates_sanctuary() {
        let emails = vec![
            make_email("Weekly Newsletter", EmailUrgency::Low, true),
            make_email("FYI: New policy", EmailUrgency::Low, true),
        ];
        let refs: Vec<_> = emails.iter().collect();
        assert_eq!(determine_room_type_from_emails(&refs), RoomType::Sanctuary);
    }
    
    #[test]
    fn test_important_creates_treasure() {
        let emails = vec![
            make_email("Important: Review needed", EmailUrgency::Important, false),
            make_email("Action Required", EmailUrgency::Important, false),
        ];
        let refs: Vec<_> = emails.iter().collect();
        assert_eq!(determine_room_type_from_emails(&refs), RoomType::Treasure);
    }
    
    #[test]
    fn test_mixed_emails_create_normal() {
        let emails = vec![
            make_email("Hello", EmailUrgency::Normal, true),
            make_email("Meeting notes", EmailUrgency::Normal, true),
        ];
        let refs: Vec<_> = emails.iter().collect();
        assert_eq!(determine_room_type_from_emails(&refs), RoomType::Normal);
    }
}

#[cfg(test)]
mod calendar_tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn make_event(summary: &str, duration_minutes: u32, attendees: u32) -> EventData {
        EventData {
            uid: summary.to_string(),
            start: Utc.with_ymd_and_hms(2026, 2, 15, 10, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2026, 2, 15, 11, 0, 0).unwrap(),
            summary: summary.to_string(),
            description: None,
            location: None,
            category: EventCategory::from_event_text(summary, None),
            duration_minutes,
            attendee_count: attendees,
        }
    }

    #[test]
    fn test_room_size_from_intensity() {
        assert_eq!(calculate_room_size_from_intensity(2), (3, 3));
        assert_eq!(calculate_room_size_from_intensity(7), (5, 5));
        assert_eq!(calculate_room_size_from_intensity(15), (7, 7));
        assert_eq!(calculate_room_size_from_intensity(30), (9, 9));
    }

    #[test]
    fn test_all_hands_creates_boss_room() {
        let events = vec![make_event("All-Hands Meeting", 60, 50)];
        assert_eq!(
            determine_room_type_from_events(&events),
            RoomType::Boss
        );
    }

    #[test]
    fn test_large_meeting_creates_boss_room() {
        let events = vec![make_event("Sprint Planning", 90, 15)];
        assert_eq!(
            determine_room_type_from_events(&events),
            RoomType::Boss
        );
    }

    #[test]
    fn test_focus_time_creates_sanctuary() {
        let events = vec![
            make_event("Focus Time", 120, 1),
            make_event("Deep Work", 60, 1),
        ];
        assert_eq!(
            determine_room_type_from_events(&events),
            RoomType::Sanctuary
        );
    }

    #[test]
    fn test_one_on_ones_create_treasure() {
        let events = vec![
            make_event("1:1 with Bob", 30, 2),
            make_event("1:1 with Alice", 30, 2),
            make_event("Team Standup", 15, 5),
        ];
        assert_eq!(
            determine_room_type_from_events(&events),
            RoomType::Treasure
        );
    }

    #[test]
    fn test_mixed_day_creates_normal() {
        let events = vec![
            make_event("Sprint Planning", 60, 8),
            make_event("Code Review", 30, 3),
            make_event("Standup", 15, 5),
        ];
        assert_eq!(
            determine_room_type_from_events(&events),
            RoomType::Normal
        );
    }

    #[test]
    fn test_generate_dungeon_from_calendar() {
        let events = vec![
            EventData {
                uid: "1".to_string(),
                start: Utc.with_ymd_and_hms(2026, 2, 15, 10, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 2, 15, 11, 0, 0).unwrap(),
                summary: "Meeting".to_string(),
                description: None,
                location: None,
                category: EventCategory::Meeting,
                duration_minutes: 60,
                attendee_count: 5,
            },
            EventData {
                uid: "2".to_string(),
                start: Utc.with_ymd_and_hms(2026, 2, 16, 9, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2026, 2, 16, 10, 0, 0).unwrap(),
                summary: "1:1".to_string(),
                description: None,
                location: None,
                category: EventCategory::OneOnOne,
                duration_minutes: 60,
                attendee_count: 2,
            },
        ];

        let world = generate_dungeon_from_calendar(&events, 42);
        assert_eq!(world.rooms.len(), 2);
    }
}
