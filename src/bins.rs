use std::collections::HashMap;
use std::fmt::Debug;
use std::iter::Map;
use std::ops::Deref;
use std::slice::Iter;
use chrono::{Date, DateTime, Utc};
use chrono_tz::Tz;
use rust_decimal::Decimal;
use crate::datebin::DateBin;
use crate::record::{PricedRecord, Record};

pub struct Bins<R>
    where
        R: Record,
{
    pub bins: Vec<DateBin<R>>,
}

impl<R> IntoIterator for Bins<R>
where R: Record
{
    type Item = DateBin<R>;
    type IntoIter = std::vec::IntoIter<DateBin<R>>;

    fn into_iter(self) -> Self::IntoIter {
        self.bins.into_iter()
    }
}
impl<'a, R> IntoIterator for &'a Bins<R>
    where R: Record
{
    type Item = &'a DateBin<R>;
    type IntoIter = std::slice::Iter<'a, DateBin<R>>;

    fn into_iter(self) -> Self::IntoIter {
        self.bins.as_slice().iter()
    }
}


impl<T, R> From<T> for Bins<R>
    where
        T: Iterator<Item = R>,
        R: Record,
{
    fn from(records: T) -> Self {
        let mut map: HashMap<Date<Utc>, Vec<R>> = HashMap::default();
        for x in records {
            let date = x.date_time().date();
            map.entry(date)
                .and_modify(|v| v.push(x))
                .or_default();
        }

        let mut s = map
            .into_iter()
            .map(|(date, records)| DateBin::new(date, records))
            .collect::<Vec<_>>();
        s.sort_by(|a, b| a.date.cmp(&b.date));
        Bins { bins: s }
    }
}
impl<R> Deref for Bins<R>
where
    R: Record
{
    type Target = Vec<DateBin<R>>;

    fn deref(&self) -> &Self::Target {
        &self.bins
    }
}