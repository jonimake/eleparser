use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use crate::record::PricedRecord;

pub struct HourComparison<'a, T>
where T: PricedRecord + 'a
{
    time: DateTime<Utc>,
    spot_price: Decimal,
    compared_price: Decimal,
    record: &'a T
}


pub struct DayComparison<'a, T>
where T: PricedRecord + 'a
{
    comparisons: &'a [T]
}

impl<'a, T> DayComparison<'a, T>
where T: PricedRecord + 'a
{
    pub fn compare(&self, compare_to_price: &Decimal) {
        let daily_energy: Decimal = self.comparisons.iter().map(|c| c.energy()).sum();
        let priced_hourly = self.comparisons.iter().fold(Decimal::from(0), |accum, item| {
            let accum = accum + (item.energy() * item.price());
            accum
        });
    }

    pub fn get_compared(&self, comparison_price: &Decimal) {
        unimplemented!()
    }
}