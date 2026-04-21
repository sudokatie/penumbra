//! Weather data fetching and parsing from Open-Meteo API.

use chrono::Utc;
use serde::Deserialize;

use super::types::{DungeonAtmosphere, WeatherCondition, WeatherData, WeatherError};

/// Open-Meteo forecast API response structure.
#[derive(Debug, Deserialize)]
struct ForecastResponse {
    current: Option<CurrentWeather>,
}

#[derive(Debug, Deserialize)]
struct CurrentWeather {
    temperature_2m: f64,
    relative_humidity_2m: u8,
    weather_code: u8,
    wind_speed_10m: f64,
}

/// Open-Meteo geocoding API response structure.
#[derive(Debug, Deserialize)]
struct GeocodingResponse {
    results: Option<Vec<GeocodingResult>>,
}

#[derive(Debug, Deserialize)]
struct GeocodingResult {
    latitude: f64,
    longitude: f64,
    name: String,
    country: Option<String>,
}

/// Fetch weather data for given latitude/longitude.
/// Uses Open-Meteo API (free, no API key needed).
pub fn fetch_weather(lat: f64, lon: f64) -> Result<WeatherData, WeatherError> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m",
        lat, lon
    );

    let response = ureq::get(&url)
        .call()
        .map_err(|e| WeatherError::FetchFailed(e.to_string()))?;

    let body = response
        .into_string()
        .map_err(|e| WeatherError::FetchFailed(e.to_string()))?;

    parse_weather_response(&body, &format!("{:.2}, {:.2}", lat, lon))
}

/// Fetch weather data for a city name.
/// First geocodes the city, then fetches weather.
pub fn fetch_weather_by_city(city: &str) -> Result<WeatherData, WeatherError> {
    // Geocode the city first
    let (lat, lon, location) = geocode_city(city)?;

    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m",
        lat, lon
    );

    let response = ureq::get(&url)
        .call()
        .map_err(|e| WeatherError::FetchFailed(e.to_string()))?;

    let body = response
        .into_string()
        .map_err(|e| WeatherError::FetchFailed(e.to_string()))?;

    parse_weather_response(&body, &location)
}

/// Geocode a city name to lat/lon coordinates.
fn geocode_city(city: &str) -> Result<(f64, f64, String), WeatherError> {
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1",
        urlencoding::encode(city)
    );

    let response = ureq::get(&url)
        .call()
        .map_err(|e| WeatherError::FetchFailed(e.to_string()))?;

    let body = response
        .into_string()
        .map_err(|e| WeatherError::FetchFailed(e.to_string()))?;

    let geo: GeocodingResponse =
        serde_json::from_str(&body).map_err(|e| WeatherError::ParseFailed(e.to_string()))?;

    let result = geo
        .results
        .and_then(|r| r.into_iter().next())
        .ok_or_else(|| WeatherError::CityNotFound(city.to_string()))?;

    let location = match result.country {
        Some(country) => format!("{}, {}", result.name, country),
        None => result.name,
    };

    Ok((result.latitude, result.longitude, location))
}

/// Parse Open-Meteo JSON response into WeatherData.
pub fn parse_weather_response(json: &str, location: &str) -> Result<WeatherData, WeatherError> {
    let response: ForecastResponse =
        serde_json::from_str(json).map_err(|e| WeatherError::ParseFailed(e.to_string()))?;

    let current = response.current.ok_or(WeatherError::NoData)?;

    let condition = WeatherCondition::from_wmo_code(current.weather_code);
    let description = wmo_code_description(current.weather_code);

    Ok(WeatherData {
        condition,
        temperature_c: current.temperature_2m,
        humidity: current.relative_humidity_2m,
        wind_speed_kph: current.wind_speed_10m,
        description,
        location: location.to_string(),
        fetched_at: Utc::now(),
    })
}

/// Generate dungeon atmosphere from weather data.
/// Combines condition with humidity and wind for nuanced atmosphere.
pub fn generate_atmosphere(weather: &WeatherData) -> DungeonAtmosphere {
    // Start with base atmosphere from condition
    let base = weather.condition.to_atmosphere();

    // High humidity can shift to Misty
    if weather.humidity > 90 && base != DungeonAtmosphere::Frozen {
        if matches!(base, DungeonAtmosphere::Dim | DungeonAtmosphere::Dark) {
            return DungeonAtmosphere::Misty;
        }
    }

    // Very high wind can shift to Tempestuous
    if weather.wind_speed_kph > 50.0 && base != DungeonAtmosphere::Frozen {
        return DungeonAtmosphere::Tempestuous;
    }

    // Extreme cold shifts to Frozen
    if weather.temperature_c < -10.0 {
        return DungeonAtmosphere::Frozen;
    }

    base
}

/// Get human-readable description for WMO weather code.
fn wmo_code_description(code: u8) -> String {
    match code {
        0 => "Clear sky".to_string(),
        1 => "Mainly clear".to_string(),
        2 => "Partly cloudy".to_string(),
        3 => "Overcast".to_string(),
        45 => "Fog".to_string(),
        48 => "Depositing rime fog".to_string(),
        51 => "Light drizzle".to_string(),
        53 => "Moderate drizzle".to_string(),
        55 => "Dense drizzle".to_string(),
        56 => "Light freezing drizzle".to_string(),
        57 => "Dense freezing drizzle".to_string(),
        61 => "Slight rain".to_string(),
        63 => "Moderate rain".to_string(),
        65 => "Heavy rain".to_string(),
        66 => "Light freezing rain".to_string(),
        67 => "Heavy freezing rain".to_string(),
        71 => "Slight snow fall".to_string(),
        73 => "Moderate snow fall".to_string(),
        75 => "Heavy snow fall".to_string(),
        77 => "Snow grains".to_string(),
        80 => "Slight rain showers".to_string(),
        81 => "Moderate rain showers".to_string(),
        82 => "Violent rain showers".to_string(),
        85 => "Slight snow showers".to_string(),
        86 => "Heavy snow showers".to_string(),
        95 => "Thunderstorm".to_string(),
        96 => "Thunderstorm with slight hail".to_string(),
        99 => "Thunderstorm with heavy hail".to_string(),
        _ => "Unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_FORECAST_JSON: &str = r#"{
        "latitude": 52.52,
        "longitude": 13.41,
        "current": {
            "time": "2026-04-21T12:00",
            "temperature_2m": 18.5,
            "relative_humidity_2m": 65,
            "weather_code": 3,
            "wind_speed_10m": 12.5
        }
    }"#;

    const SAMPLE_STORM_JSON: &str = r#"{
        "current": {
            "time": "2026-04-21T12:00",
            "temperature_2m": 22.0,
            "relative_humidity_2m": 85,
            "weather_code": 95,
            "wind_speed_10m": 45.0
        }
    }"#;

    const SAMPLE_SNOW_JSON: &str = r#"{
        "current": {
            "time": "2026-04-21T12:00",
            "temperature_2m": -5.0,
            "relative_humidity_2m": 70,
            "weather_code": 73,
            "wind_speed_10m": 20.0
        }
    }"#;

    const SAMPLE_FOG_JSON: &str = r#"{
        "current": {
            "time": "2026-04-21T12:00",
            "temperature_2m": 10.0,
            "relative_humidity_2m": 98,
            "weather_code": 45,
            "wind_speed_10m": 5.0
        }
    }"#;

    const SAMPLE_HAIL_JSON: &str = r#"{
        "current": {
            "time": "2026-04-21T12:00",
            "temperature_2m": 15.0,
            "relative_humidity_2m": 80,
            "weather_code": 99,
            "wind_speed_10m": 35.0
        }
    }"#;

    #[test]
    fn test_parse_forecast_response() {
        let weather = parse_weather_response(SAMPLE_FORECAST_JSON, "Berlin").unwrap();

        assert_eq!(weather.condition, WeatherCondition::Cloudy);
        assert!((weather.temperature_c - 18.5).abs() < 0.01);
        assert_eq!(weather.humidity, 65);
        assert!((weather.wind_speed_kph - 12.5).abs() < 0.01);
        assert_eq!(weather.location, "Berlin");
    }

    #[test]
    fn test_parse_storm() {
        let weather = parse_weather_response(SAMPLE_STORM_JSON, "Test").unwrap();

        assert_eq!(weather.condition, WeatherCondition::Storm);
        assert_eq!(weather.description, "Thunderstorm");
    }

    #[test]
    fn test_parse_snow() {
        let weather = parse_weather_response(SAMPLE_SNOW_JSON, "Test").unwrap();

        assert_eq!(weather.condition, WeatherCondition::Snow);
        assert!(weather.temperature_c < 0.0);
    }

    #[test]
    fn test_parse_fog() {
        let weather = parse_weather_response(SAMPLE_FOG_JSON, "Test").unwrap();

        assert_eq!(weather.condition, WeatherCondition::Fog);
        assert!(weather.humidity > 90);
    }

    #[test]
    fn test_parse_hail() {
        let weather = parse_weather_response(SAMPLE_HAIL_JSON, "Test").unwrap();

        assert_eq!(weather.condition, WeatherCondition::Hail);
    }

    #[test]
    fn test_parse_invalid_json() {
        let result = parse_weather_response("not json", "Test");
        assert!(matches!(result, Err(WeatherError::ParseFailed(_))));
    }

    #[test]
    fn test_parse_missing_current() {
        let json = r#"{"latitude": 52.52}"#;
        let result = parse_weather_response(json, "Test");
        assert!(matches!(result, Err(WeatherError::NoData)));
    }

    #[test]
    fn test_generate_atmosphere_basic() {
        let weather = WeatherData {
            condition: WeatherCondition::Clear,
            temperature_c: 20.0,
            humidity: 50,
            wind_speed_kph: 10.0,
            description: "Clear".to_string(),
            location: "Test".to_string(),
            fetched_at: Utc::now(),
        };

        assert_eq!(generate_atmosphere(&weather), DungeonAtmosphere::Bright);
    }

    #[test]
    fn test_generate_atmosphere_high_humidity() {
        let weather = WeatherData {
            condition: WeatherCondition::Cloudy,
            temperature_c: 15.0,
            humidity: 95,
            wind_speed_kph: 5.0,
            description: "Cloudy".to_string(),
            location: "Test".to_string(),
            fetched_at: Utc::now(),
        };

        // High humidity shifts Dim to Misty
        assert_eq!(generate_atmosphere(&weather), DungeonAtmosphere::Misty);
    }

    #[test]
    fn test_generate_atmosphere_high_wind() {
        let weather = WeatherData {
            condition: WeatherCondition::Cloudy,
            temperature_c: 15.0,
            humidity: 50,
            wind_speed_kph: 60.0,
            description: "Windy".to_string(),
            location: "Test".to_string(),
            fetched_at: Utc::now(),
        };

        // High wind makes it Tempestuous
        assert_eq!(generate_atmosphere(&weather), DungeonAtmosphere::Tempestuous);
    }

    #[test]
    fn test_generate_atmosphere_extreme_cold() {
        let weather = WeatherData {
            condition: WeatherCondition::Clear,
            temperature_c: -15.0,
            humidity: 40,
            wind_speed_kph: 10.0,
            description: "Cold".to_string(),
            location: "Test".to_string(),
            fetched_at: Utc::now(),
        };

        // Extreme cold always makes it Frozen
        assert_eq!(generate_atmosphere(&weather), DungeonAtmosphere::Frozen);
    }

    #[test]
    fn test_wmo_descriptions() {
        assert_eq!(wmo_code_description(0), "Clear sky");
        assert_eq!(wmo_code_description(95), "Thunderstorm");
        assert_eq!(wmo_code_description(45), "Fog");
        assert_eq!(wmo_code_description(73), "Moderate snow fall");
        assert_eq!(wmo_code_description(255), "Unknown");
    }

    #[test]
    fn test_difficulty_scales_with_extremes() {
        let mild = WeatherData {
            condition: WeatherCondition::Clear,
            temperature_c: 20.0,
            humidity: 50,
            wind_speed_kph: 10.0,
            description: "Clear".to_string(),
            location: "Test".to_string(),
            fetched_at: Utc::now(),
        };

        let extreme = WeatherData {
            condition: WeatherCondition::Storm,
            temperature_c: -20.0,
            humidity: 95,
            wind_speed_kph: 70.0,
            description: "Blizzard".to_string(),
            location: "Test".to_string(),
            fetched_at: Utc::now(),
        };

        assert!(extreme.difficulty_multiplier() > mild.difficulty_multiplier());
    }
}
