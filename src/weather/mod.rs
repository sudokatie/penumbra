//! Weather data source for dungeon generation.
//!
//! Fetches current weather data from Open-Meteo API and transforms it
//! into dungeon atmosphere and modifiers.

pub mod parser;
pub mod types;

pub use parser::{fetch_weather, fetch_weather_by_city, generate_atmosphere, parse_weather_response};
pub use types::{DungeonAtmosphere, WeatherCondition, WeatherData, WeatherError};
