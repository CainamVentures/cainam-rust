pub mod api;
pub mod error;

use std::fmt;
use serde::{Deserialize, Serialize};
use crate::types::error::BirdeyeError;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TimeInterval {
    #[serde(rename = "5m")]
    FiveMinutes,
    #[serde(rename = "15m")]
    FifteenMinutes,
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "4h")]
    FourHours,
    #[serde(rename = "1d")]
    OneDay,
    #[serde(rename = "1w")]
    OneWeek,
    #[serde(rename = "1M")]
    OneMonth,
}

impl fmt::Display for TimeInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeInterval::FiveMinutes => write!(f, "5m"),
            TimeInterval::FifteenMinutes => write!(f, "15m"),
            TimeInterval::OneHour => write!(f, "1h"),
            TimeInterval::FourHours => write!(f, "4h"),
            TimeInterval::OneDay => write!(f, "1d"),
            TimeInterval::OneWeek => write!(f, "1w"),
            TimeInterval::OneMonth => write!(f, "1M"),
        }
    }
}

impl std::str::FromStr for TimeInterval {
    type Err = BirdeyeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "5m" => Ok(TimeInterval::FiveMinutes),
            "15m" => Ok(TimeInterval::FifteenMinutes),
            "1h" => Ok(TimeInterval::OneHour),
            "4h" => Ok(TimeInterval::FourHours),
            "1d" => Ok(TimeInterval::OneDay),
            "1w" => Ok(TimeInterval::OneWeek),
            "1m" => Ok(TimeInterval::OneMonth),
            _ => Err(BirdeyeError::InvalidResponse(format!(
                "Invalid time interval: {}. Valid intervals are: 5m, 15m, 1h, 4h, 1d, 1w, 1M",
                s
            ))),
        }
    }
} 