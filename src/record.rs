use std::fmt::{Display, Formatter};
use std::str::FromStr;

use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::{Deserialize, Serialize};

pub mod fingrid;
pub mod oomi;

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum TimeResolution {
    PT15M,
    PT1H,
}
pub trait Record: Sized {
    fn resolution(&self) -> TimeResolution;
    fn date_time(&self) -> DateTime<Utc>;
    fn energy(&self) -> Decimal;
    fn temperature(&self) -> Option<f32>;
}

pub trait PricedRecord: Record + Sized {
    fn price(&self) -> Decimal;
}

#[derive(Debug, Copy, Clone)]
pub struct RecordWithPrice<'a, T>
where
    T: Record,
{
    pub record: &'a T,
    pub price: Decimal,
}

impl<'a, T> RecordWithPrice<'a, T>
where
    T: Record,
{
    pub fn new(record: &'a T, price: Decimal) -> Self {
        RecordWithPrice { record, price }
    }
}

impl<T> Display for &RecordWithPrice<'_, T>
where
    T: Record,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let date = self.record.date_time();
        let price = self.price;
        write!(f, "date: {}, price: {}", date, price)
    }
}
