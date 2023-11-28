use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use crate::record::{Record, TimeResolution};

use serde::{Deserialize, Deserializer, Serialize};
use serde::de::Error;


#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct FingridRecord {
    // #[serde(alias="Mittauspisteen tunnus")]
    // pub id: i32,
    //
    // #[serde(alias="Tuotteen tyyppi")]
    // pub product_type: i32,
    //
    #[serde(alias="Resoluutio")]
    pub resolution: TimeResolution,
    //
    // #[serde(alias="Yksikkötyyppi")]
    // pub unit_type: &'static str,
    //
    // #[serde(alias="Lukeman tyyppi")]
    // pub reading_type: &'static str,

    #[serde(alias="Alkuaika")]
    pub date_time: DateTime<Utc>,

    #[serde(alias="Määrä", deserialize_with = "from_decimal_comma")]
    pub energy: Decimal
}

fn from_decimal_comma<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
    where
        D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    let mut split = s.split(",");
    let whole = split.next().unwrap().parse::<i16>().unwrap();
    let scale = 100000;
    let frac = split.next().unwrap().parse::<i64>().unwrap() * scale;

    let whole_d = Decimal::from(whole);
    let frac_d = Decimal::new(frac, 11);

    Ok(whole_d + frac_d)
}

impl Record for FingridRecord {
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
        None
    }

}
