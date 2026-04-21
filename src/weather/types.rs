//! Weather data types for dungeon generation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::world::RoomType;

/// Weather conditions from API data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WeatherCondition {
    /// Clear sky
    #[default]
    Clear,
    /// Overcast or partly cloudy
    Cloudy,
    /// Light to moderate rain
    Rain,
    /// Thunderstorm or heavy rain
    Storm,
    /// Snow or sleet
    Snow,
    /// Fog or mist
    Fog,
    /// Strong wind without precipitation
    Windy,
    /// Hail or ice pellets
    Hail,
}

/// Dungeon atmosphere derived from weather.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DungeonAtmosphere {
    /// Clear weather - well lit
    #[default]
    Bright,
    /// Cloudy - standard lighting
    Dim,
    /// Rain - reduced visibility
    Dark,
    /// Fog - severely limited visibility
    Misty,
    /// Storm - chaotic, flickering
    Tempestuous,
    /// Snow/ice - cold hazards
    Frozen,
}

/// Data extracted from weather API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    /// Current weather condition
    pub condition: WeatherCondition,
    /// Temperature in Celsius
    pub temperature_c: f64,
    /// Humidity percentage (0-100)
    pub humidity: u8,
    /// Wind speed in km/h
    pub wind_speed_kph: f64,
    /// Human-readable description
    pub description: String,
    /// Location name
    pub location: String,
    /// When this data was fetched
    pub fetched_at: DateTime<Utc>,
}

/// Errors that can occur during weather fetching/parsing.
#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Failed to fetch weather data: {0}")]
    FetchFailed(String),

    #[error("Failed to parse weather response: {0}")]
    ParseFailed(String),

    #[error("No weather data available")]
    NoData,

    #[error("City not found: {0}")]
    CityNotFound(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

impl WeatherCondition {
    /// Map weather condition to dungeon atmosphere.
    pub fn to_atmosphere(&self) -> DungeonAtmosphere {
        match self {
            WeatherCondition::Clear => DungeonAtmosphere::Bright,
            WeatherCondition::Cloudy => DungeonAtmosphere::Dim,
            WeatherCondition::Rain => DungeonAtmosphere::Dark,
            WeatherCondition::Storm => DungeonAtmosphere::Tempestuous,
            WeatherCondition::Snow => DungeonAtmosphere::Frozen,
            WeatherCondition::Fog => DungeonAtmosphere::Misty,
            WeatherCondition::Windy => DungeonAtmosphere::Dim,
            WeatherCondition::Hail => DungeonAtmosphere::Frozen,
        }
    }

    /// Get enemy spawn modifier (multiplier for enemy count).
    /// Storms spawn more enemies, fog reduces spawns.
    pub fn enemy_spawn_modifier(&self) -> f64 {
        match self {
            WeatherCondition::Clear => 1.0,
            WeatherCondition::Cloudy => 1.1,
            WeatherCondition::Rain => 1.2,
            WeatherCondition::Storm => 1.5,
            WeatherCondition::Snow => 1.1,
            WeatherCondition::Fog => 0.8, // Fewer enemies, but harder to see
            WeatherCondition::Windy => 1.2,
            WeatherCondition::Hail => 1.4,
        }
    }

    /// Get room type bias based on weather.
    /// Returns preferred room types for this weather.
    pub fn room_type_bias(&self) -> RoomType {
        match self {
            WeatherCondition::Clear => RoomType::Normal,
            WeatherCondition::Cloudy => RoomType::Normal,
            WeatherCondition::Rain => RoomType::Normal, // Would be water room if we had it
            WeatherCondition::Storm => RoomType::Boss,
            WeatherCondition::Snow => RoomType::Sanctuary, // Ice preservation = sanctuaries
            WeatherCondition::Fog => RoomType::Library, // Mystery/knowledge
            WeatherCondition::Windy => RoomType::Normal,
            WeatherCondition::Hail => RoomType::Treasure, // Rare weather = treasure
        }
    }

    /// Parse from Open-Meteo WMO weather code.
    /// See: https://open-meteo.com/en/docs
    pub fn from_wmo_code(code: u8) -> Self {
        match code {
            0 => WeatherCondition::Clear,                    // Clear sky
            1..=3 => WeatherCondition::Cloudy,               // Mainly clear, partly cloudy, overcast
            45 | 48 => WeatherCondition::Fog,                // Fog, depositing rime fog
            51..=55 => WeatherCondition::Rain,               // Drizzle
            56..=57 => WeatherCondition::Snow,               // Freezing drizzle
            61..=65 => WeatherCondition::Rain,               // Rain
            66..=67 => WeatherCondition::Snow,               // Freezing rain
            71..=75 => WeatherCondition::Snow,               // Snow fall
            77 => WeatherCondition::Snow,                    // Snow grains
            80..=82 => WeatherCondition::Rain,               // Rain showers
            85..=86 => WeatherCondition::Snow,               // Snow showers
            95 => WeatherCondition::Storm,                   // Thunderstorm
            96 | 99 => WeatherCondition::Hail,               // Thunderstorm with hail
            _ => WeatherCondition::Clear,                    // Unknown -> default to clear
        }
    }
}

impl DungeonAtmosphere {
    /// Get field of view radius modifier.
    /// Positive = better visibility, negative = reduced.
    pub fn fov_radius_modifier(&self) -> i32 {
        match self {
            DungeonAtmosphere::Bright => 2,
            DungeonAtmosphere::Dim => 0,
            DungeonAtmosphere::Dark => -2,
            DungeonAtmosphere::Misty => -4,
            DungeonAtmosphere::Tempestuous => -1,
            DungeonAtmosphere::Frozen => 0,
        }
    }

    /// Get description for UI display.
    pub fn description(&self) -> &'static str {
        match self {
            DungeonAtmosphere::Bright => "The dungeon is well-lit",
            DungeonAtmosphere::Dim => "The dungeon is dimly lit",
            DungeonAtmosphere::Dark => "Darkness pervades the dungeon",
            DungeonAtmosphere::Misty => "Thick mist obscures your vision",
            DungeonAtmosphere::Tempestuous => "The air crackles with energy",
            DungeonAtmosphere::Frozen => "A bitter cold fills the air",
        }
    }
}

impl WeatherData {
    /// Calculate difficulty multiplier based on weather extremes.
    /// Extreme weather = harder dungeon.
    pub fn difficulty_multiplier(&self) -> f64 {
        let mut multiplier = 1.0;

        // Temperature extremes increase difficulty
        if self.temperature_c < -10.0 || self.temperature_c > 35.0 {
            multiplier += 0.3;
        } else if self.temperature_c < 0.0 || self.temperature_c > 30.0 {
            multiplier += 0.15;
        }

        // High humidity is oppressive
        if self.humidity > 90 {
            multiplier += 0.2;
        } else if self.humidity > 75 {
            multiplier += 0.1;
        }

        // High wind adds chaos
        if self.wind_speed_kph > 50.0 {
            multiplier += 0.3;
        } else if self.wind_speed_kph > 30.0 {
            multiplier += 0.15;
        }

        // Weather condition modifier
        multiplier *= self.condition.enemy_spawn_modifier();

        multiplier
    }

    /// Calculate intensity for room sizing (like other data sources).
    pub fn intensity(&self) -> u32 {
        // Base intensity from condition
        let condition_intensity = match self.condition {
            WeatherCondition::Clear => 5,
            WeatherCondition::Cloudy => 7,
            WeatherCondition::Rain => 10,
            WeatherCondition::Storm => 20,
            WeatherCondition::Snow => 12,
            WeatherCondition::Fog => 8,
            WeatherCondition::Windy => 9,
            WeatherCondition::Hail => 18,
        };

        // Add modifiers
        let temp_mod = if self.temperature_c < 0.0 || self.temperature_c > 30.0 {
            3
        } else {
            0
        };
        let wind_mod = (self.wind_speed_kph / 20.0) as u32;

        condition_intensity + temp_mod + wind_mod
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_to_atmosphere() {
        assert_eq!(WeatherCondition::Clear.to_atmosphere(), DungeonAtmosphere::Bright);
        assert_eq!(WeatherCondition::Cloudy.to_atmosphere(), DungeonAtmosphere::Dim);
        assert_eq!(WeatherCondition::Rain.to_atmosphere(), DungeonAtmosphere::Dark);
        assert_eq!(WeatherCondition::Storm.to_atmosphere(), DungeonAtmosphere::Tempestuous);
        assert_eq!(WeatherCondition::Snow.to_atmosphere(), DungeonAtmosphere::Frozen);
        assert_eq!(WeatherCondition::Fog.to_atmosphere(), DungeonAtmosphere::Misty);
        assert_eq!(WeatherCondition::Windy.to_atmosphere(), DungeonAtmosphere::Dim);
        assert_eq!(WeatherCondition::Hail.to_atmosphere(), DungeonAtmosphere::Frozen);
    }

    #[test]
    fn test_enemy_spawn_modifier() {
        // Storm spawns most enemies
        assert!(WeatherCondition::Storm.enemy_spawn_modifier() > 1.0);
        // Fog spawns fewer
        assert!(WeatherCondition::Fog.enemy_spawn_modifier() < 1.0);
        // Clear is baseline
        assert_eq!(WeatherCondition::Clear.enemy_spawn_modifier(), 1.0);
    }

    #[test]
    fn test_room_type_bias() {
        assert_eq!(WeatherCondition::Storm.room_type_bias(), RoomType::Boss);
        assert_eq!(WeatherCondition::Fog.room_type_bias(), RoomType::Library);
        assert_eq!(WeatherCondition::Snow.room_type_bias(), RoomType::Sanctuary);
        assert_eq!(WeatherCondition::Hail.room_type_bias(), RoomType::Treasure);
    }

    #[test]
    fn test_fov_radius_modifier() {
        assert_eq!(DungeonAtmosphere::Bright.fov_radius_modifier(), 2);
        assert_eq!(DungeonAtmosphere::Dim.fov_radius_modifier(), 0);
        assert_eq!(DungeonAtmosphere::Dark.fov_radius_modifier(), -2);
        assert_eq!(DungeonAtmosphere::Misty.fov_radius_modifier(), -4);
        assert_eq!(DungeonAtmosphere::Tempestuous.fov_radius_modifier(), -1);
        assert_eq!(DungeonAtmosphere::Frozen.fov_radius_modifier(), 0);
    }

    #[test]
    fn test_wmo_code_parsing() {
        assert_eq!(WeatherCondition::from_wmo_code(0), WeatherCondition::Clear);
        assert_eq!(WeatherCondition::from_wmo_code(3), WeatherCondition::Cloudy);
        assert_eq!(WeatherCondition::from_wmo_code(45), WeatherCondition::Fog);
        assert_eq!(WeatherCondition::from_wmo_code(63), WeatherCondition::Rain);
        assert_eq!(WeatherCondition::from_wmo_code(73), WeatherCondition::Snow);
        assert_eq!(WeatherCondition::from_wmo_code(95), WeatherCondition::Storm);
        assert_eq!(WeatherCondition::from_wmo_code(99), WeatherCondition::Hail);
    }

    #[test]
    fn test_difficulty_multiplier_normal() {
        let weather = WeatherData {
            condition: WeatherCondition::Clear,
            temperature_c: 20.0,
            humidity: 50,
            wind_speed_kph: 10.0,
            description: "Clear sky".to_string(),
            location: "Test City".to_string(),
            fetched_at: Utc::now(),
        };
        // Normal conditions should be close to 1.0
        assert!((weather.difficulty_multiplier() - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_difficulty_multiplier_extreme() {
        let weather = WeatherData {
            condition: WeatherCondition::Storm,
            temperature_c: -15.0,
            humidity: 95,
            wind_speed_kph: 60.0,
            description: "Blizzard".to_string(),
            location: "Test City".to_string(),
            fetched_at: Utc::now(),
        };
        // Extreme conditions should have high multiplier
        assert!(weather.difficulty_multiplier() > 2.0);
    }

    #[test]
    fn test_weather_intensity() {
        let clear = WeatherData {
            condition: WeatherCondition::Clear,
            temperature_c: 20.0,
            humidity: 50,
            wind_speed_kph: 5.0,
            description: "Clear".to_string(),
            location: "Test".to_string(),
            fetched_at: Utc::now(),
        };

        let storm = WeatherData {
            condition: WeatherCondition::Storm,
            temperature_c: 5.0,
            humidity: 90,
            wind_speed_kph: 50.0,
            description: "Storm".to_string(),
            location: "Test".to_string(),
            fetched_at: Utc::now(),
        };

        // Storm should have higher intensity
        assert!(storm.intensity() > clear.intensity());
    }

    #[test]
    fn test_atmosphere_description() {
        assert!(!DungeonAtmosphere::Bright.description().is_empty());
        assert!(!DungeonAtmosphere::Misty.description().is_empty());
    }
}
