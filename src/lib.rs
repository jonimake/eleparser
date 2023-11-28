use crate::bins::Bins;
use chrono::{DateTime, Duration, Timelike, Utc};
use chrono_tz::Europe::Helsinki;
use chrono_tz::Tz;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::ops::Sub;
use std::path::Path;
use itertools::Itertools;

use crate::parser::{EnergyParser, FingridParser, Parser};
use crate::record::fingrid::FingridRecord;
use crate::record::RecordWithPrice;
use crate::record::{Record, TimeResolution};

mod bins;
mod datebin;
mod parser;
mod priceclient;
mod record;

pub mod plotter;

#[derive(Debug, Copy, Clone, Serialize)]
pub struct CumulativeComparisonData {
    pub date_time: DateTime<Utc>,
    pub market_price_for_hour: Decimal,
    pub energy: Decimal,
    pub cumulative_market_price: Decimal,
    pub cumulative_set_price: Decimal,
}
pub fn cumulative_price_by_day(
    records: Vec<RecordWithPrice<FingridRecord>>,
    contract_price: Decimal,
) -> Vec<CumulativeComparisonData> {
    let mut current_market_sum = Decimal::from(0);
    let mut current_constant_sum = Decimal::from(0);
    records
        .iter()
        .map(|r| {
            let market_price_kwh = r.price;
            let cumulative_market = current_market_sum + (market_price_kwh * r.record.energy);
            let cumulative_constant = current_constant_sum + (contract_price * r.record.energy);
            let data = CumulativeComparisonData {
                date_time: r.record.date_time,
                cumulative_market_price: cumulative_market,
                market_price_for_hour: market_price_kwh,
                cumulative_set_price: cumulative_constant,
                energy: r.record.energy,
            };
            current_market_sum = cumulative_market;
            current_constant_sum = cumulative_constant;
            data
        })
        .map(|r| CumulativeComparisonData {
            cumulative_market_price: r.cumulative_market_price / Decimal::from(100),
            cumulative_set_price: r.cumulative_set_price / Decimal::from(100),
            ..r
        })
        .collect()
}

fn print<T>(s: impl Iterator<Item = T>)
where
    T: Debug,
{
    for f in s {
        println!("{:?}", f);
    }
}

fn with_prices<'a>(
    min: &DateTime<Utc>,
    max: &DateTime<Utc>,
    bins: &'a Bins<FingridRecord>,
) -> Result<Vec<RecordWithPrice<'a, FingridRecord>>, Box<dyn Error>> {
    let adjusted_start = *min - Duration::hours(1);
    let prices = priceclient::get_prices(&adjusted_start, max)?;
    let mut hourly_prices: HashMap<DateTime<Utc>, Decimal> = HashMap::new();
    for hourly_price in prices {
        hourly_prices.insert(hourly_price.time, hourly_price.price);
    }

    let records = bins.bins.iter().flat_map(|b| b.records());
    let mut records_with_prices = records
        .map(|r| {
            let time = r.date_time().with_timezone(&Utc);

            let adjusted_time = match r.resolution {
                TimeResolution::PT15M => {
                    let price_time = match time.minute() {
                        0 => time - Duration::hours(1),
                        _ => time.with_minute(0).unwrap(),
                    };
                    price_time
                }
                TimeResolution::PT1H => time - Duration::hours(1),
            };
            let price = hourly_prices.get(&adjusted_time).unwrap_or_else(|| {
                panic!("No price data found for {adjusted_time:?} (original {time:?})")
            });
            RecordWithPrice::new(r, *price)
        })
        .collect::<Vec<_>>();
    records_with_prices.sort_by(|a, b| a.record.date_time().cmp(&b.record.date_time()));
    // bins.bins.iter().map(|record|)
    Ok(records_with_prices)
}

pub fn get_data(
    file_path: &Path,
    start_time: &DateTime<Utc>,
    end_time: &DateTime<Utc>,
) -> Result<Vec<CumulativeComparisonData>, Box<dyn Error>> {
    let fingriddata = Parser::<FingridRecord>::parse(file_path);

    let fingriddata = fingriddata
        .into_iter()
        .filter(|d| d.date_time <= *end_time && d.date_time >= *start_time);

    let bins: Bins<FingridRecord> = Bins::from(fingriddata);
    let record_prices = with_prices(start_time, end_time, &bins)?;
    let cumulative_series = cumulative_price_by_day(record_prices, Decimal::from(7));
    Ok(cumulative_series)
}
