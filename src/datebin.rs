use std::fmt::{Display, Formatter};
use std::ops::{Deref, Index};

use chrono::{Date, TimeZone, Utc};
use rust_decimal::prelude::*;

use crate::record::{oomi, PricedRecord, Record, RecordWithPrice};

#[derive(Debug, Clone)]
pub struct DateBin<T>
where T: Record {
    pub date: Date<Utc>,
    /// Sorted by energy
    records: Vec<T>,
    median: usize,
}

impl<T> Display for DateBin<T>
where T: Record
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "date: {}, median: {}", self.date, self.median)
    }
}

impl<R> DateBin<R>
where R: Record {
    pub fn new(date: Date<Utc>, records: Vec<R>) -> Self {
        let mut energy: Vec<_> = records;
        energy.sort_by_key(|r| r.energy());
        let median = energy.len() / 2;
        DateBin {
            date,
            records: energy,
            median,
        }
    }

    pub fn records(&self) -> &[R] {
        self.records.as_slice()
    }

    pub fn energy_sum(&self) -> Decimal {
        self.records.iter().map(|e| e.energy()).sum()
    }

    pub fn hourly_average(&self) -> Decimal {
        self.energy_sum() / Decimal::from(24)
    }

    pub fn above_percentile(&self, percent: Decimal) -> impl Iterator<Item = &R> {
        let max = self
            .records
            .iter()
            .max_by(|a, b| a.energy().cmp(&b.energy()))
            .map(|r| r.energy())
            .unwrap_or(Decimal::MAX);

        self.records
            .iter()
            .filter(move |r| (r.energy() / max) > percent)
    }

    pub fn records_above_consumption(&self, consumption: Decimal) -> impl Iterator<Item = &R> {
        self.records.iter().filter(move |r| r.energy() > consumption)
    }

    pub fn median_energy_record(&self) -> Option<&R> {
        self.records().get(self.median)
    }

    pub fn nth_percentile(&self, percentile: Decimal) -> NthPercentile<R> {
        NthPercentile {
            bin: self,
            nth_percentile: percentile,
        }
    }
}

pub struct NthPercentile<'a, T>
where T: Record {
    pub bin: &'a DateBin<T>,
    pub nth_percentile: Decimal,
}

impl<T> Deref for NthPercentile<'_, T>
where T: Record {
    type Target = DateBin<T>;
    fn deref(&self) -> &Self::Target {
        self.bin
    }
}

impl<T> Display for NthPercentile<'_, T>
where T: Record {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let avg = self.hourly_average();
        let total = self.energy_sum();
        let above = self
            .above_percentile(self.nth_percentile)
            .map(|r| r.energy())
            .sum::<Decimal>();
        let date = self.date.naive_local();
        let median = self.median_energy_record().unwrap().energy();
        write!(f, "date: {}, total: {:.5} kWh, average: {:.5} kWh, median: {:.5} kWh, above {:.0}th percentile: {:.5} kWh)"
               , date
               , total
               , avg
               , median
               , self.nth_percentile * Decimal::from_f32(100.0).unwrap()
               , above)
    }
}


impl<T> Default for DateBin<T>
where T: Record {
    fn default() -> Self {
        DateBin {
            date: Utc.ymd(2000, 1, 1),
            records: Vec::default(),
            median: 0,
        }
    }
}
