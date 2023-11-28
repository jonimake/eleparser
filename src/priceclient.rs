use chrono::{DateTime, Duration, NaiveDateTime, SecondsFormat, Utc};
use reqwest::Error;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::Deserialize;
use chrono_intervals::{Grouping, IntervalGenerator};

#[derive(Deserialize, Debug)]
struct ApiResponse {
    prices: Vec<PriceData>
}

#[derive(Deserialize, Debug)]
struct PriceData {
    date: DateTime<Utc>,
    value: f32,
}

/// Price for the following hour
pub struct HourlyPrice {
    pub time: DateTime<Utc>,
    pub price: Decimal,
}

impl From<&PriceData> for HourlyPrice {
    fn from(p: &PriceData) -> Self {
        // let time = NaiveDateTime::from_timestamp(p.date, 0);
        let time = p.date;
        let euros_per_kwh = Decimal::from_f32(p.value).unwrap() / Decimal::from(1000);
        let cents_per_kwh = euros_per_kwh * Decimal::from(100);
        HourlyPrice {
            time,
            price: cents_per_kwh,
        }
    }
}
const URL: &str = "https://sahkotin.fi/prices";
pub fn get_prices(
    start: &DateTime<Utc>,
    end: &DateTime<Utc>,
// ) -> Vec<HourlyPrice> {
) -> Result<Vec<HourlyPrice>, reqwest::Error> {
    let years = end.years_since(*start).unwrap();

    let interval_gen = IntervalGenerator::new()
        .with_grouping(Grouping::PerMonth)
        .with_precision(Duration::days(1))
        .without_extended_begin()
        .without_extended_end();

    // let intervals = interval_gen.get_intervals(*start, *end); todo split into multiple queries if timespan is greater than a year
    let intervals = [(start, end)];

    let result: Result<Vec<Vec<HourlyPrice>>, reqwest::Error> = intervals
        .iter()
        .map(|(interval_start, interval_end)| {
            let prices = get_year_prices(interval_start, interval_end);
            prices
        }).collect();

    result.map(|v| {
        v.into_iter().flatten().collect()
    })
}

pub fn get_year_prices(
    start: &DateTime<Utc>,
    end: &DateTime<Utc>,
) -> Result<Vec<HourlyPrice>, Error> {
    let start = start.to_rfc3339_opts(SecondsFormat::Millis, true);
    let end = end.to_rfc3339_opts(SecondsFormat::Millis, true);
    println!("{}, {}", &start, &end);
    let client = reqwest::blocking::Client::new();
    let req = client.get(URL).query(&[("start", &start), ("end", &end)]);

    println!("{:?}", req.try_clone().unwrap().build().unwrap());
    // println!("{:?}", req);
    let response = req.send()?.json::<ApiResponse>()?;
    let v = response.prices
        .iter()
        .map(HourlyPrice::from)
        .collect::<Vec<_>>();
    Ok(v)
}
