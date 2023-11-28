use std::num::ParseFloatError;
use crate::record::{Record, TimeResolution};
use chrono::{Datelike, DateTime, LocalResult, NaiveDateTime, Timelike, TimeZone, Utc};
use chrono_tz::Europe::Helsinki;
use chrono_tz::Tz;
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};
use serde::de::Error;

#[derive(Debug, Copy, Clone, Deserialize)]
pub enum OomiStatus {
    Mitattu,
    Laskettu
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct OomiRecord {
    #[serde(alias = "Tunti", deserialize_with = "from_oomi_date")]
    pub date_time: DateTime<Utc>,

    #[serde(
        alias = "Energia yhteensä (kWh)",
        deserialize_with = "oomi_from_decimal_comma"
    )]
    pub energy: Decimal,

    #[serde(alias = "Status")]
    pub status: Option<OomiStatus>,

    #[serde(alias = "Lämpötila (°C)", deserialize_with = "from_f32_comma", default)]
    pub temperature: Option<f32>,

    pub resolution: TimeResolution,
}

impl Default for OomiRecord {
    fn default() -> Self {
        OomiRecord {
            date_time: Default::default(),
            energy: Default::default(),
            status: None,
            temperature: None,
            resolution: TimeResolution::PT1H,
        }
    }
}

fn from_oomi_date<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    let date = NaiveDateTime::parse_from_str(s,"%d.%m.%Y %H:%M")
        .map_err(|e| D::Error::custom(format!("Unable to parse date: '{}'", s)))?;

    let helsinki_time = match Helsinki.from_local_datetime(&date) {
        LocalResult::None => Err(D::Error::custom(format!("Unable to parse date: '{}'", s))),
        LocalResult::Single(a) => Ok(a),
        LocalResult::Ambiguous(a, b) => {
            // try to to recover daylight saving time

            let dst_fuck = if date.month() > 6 {
                eprintln!("Ambiguous date time: {}. Ranges from {} to {}. Assume as normal time", s, a, b);
                let offset = date.with_hour(date.hour()+1).unwrap();
                Helsinki.from_local_datetime(&offset).unwrap()
            } else {
                eprintln!("Ambiguous date time: {}. Ranges from {} to {}. Assume as daylight saving time", s, a, b);
                let offset = date.with_hour(date.hour()-1).unwrap();
                Helsinki.from_local_datetime(&offset).unwrap()
            };
            Ok(dst_fuck)
            // Err(D::Error::custom(format!("Ambigious date time: {}. Ranges from {} to {}", s, a, b)))
        }
    }?;

    let utc = helsinki_time.with_timezone(&Utc);
    Ok(utc)
}

fn from_f32_comma<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    let parsed = s
        .replace(",", ".")
        .parse::<f32>();
    match parsed {
        Ok(d) => Ok(Some(d)),
        Err(_) => Ok(None)
    }

}

fn oomi_from_decimal_comma<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    let mut split = s.split(",");
    let whole = split.next().unwrap().parse::<i16>().unwrap();
    let scale = 100;
    let frac = split.next().unwrap().parse::<i64>().unwrap() * scale;

    let whole_d = Decimal::from(whole);
    let frac_d = Decimal::new(frac, 2);

    Ok(whole_d + frac_d)
}

impl Record for OomiRecord {
    fn resolution(&self) -> TimeResolution {
        self.resolution
    }

    fn date_time(&self) -> DateTime<Utc> {
        self.date_time
    }

    fn energy(&self) -> Decimal {
        self.energy
    }

    fn temperature(&self) -> Option<f32> {
        self.temperature
    }
}
