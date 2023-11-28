use std::marker::PhantomData;
use std::path::Path;
use serde::Deserialize;

use crate::record::fingrid::FingridRecord;
use crate::record::oomi::OomiRecord;
use crate::record::Record;
pub trait EnergyParser<T>
where T: Record
{
    fn parse(file_path: &Path) -> Vec<T>;
}
pub struct Parser<T>
{
    _phantom: PhantomData<T>
}

pub type FingridParser = Parser<FingridRecord>;
pub type OomiParser = Parser<OomiRecord>;

impl EnergyParser<FingridRecord> for Parser<FingridRecord> {

    fn parse(file_path: &Path) -> Vec<FingridRecord> {
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .has_headers(true)
            .from_path(file_path)
            .unwrap();

        let records: Vec<_> = reader
            .deserialize()
            .map(|r| {
                let record: FingridRecord = r.unwrap();
                record
            })
            .collect();

        records
    }
}

impl EnergyParser<OomiRecord> for Parser<OomiRecord> {
    fn parse(file_path: &Path) -> Vec<OomiRecord> {
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .has_headers(true)
            .from_path(file_path)
            .unwrap();

        let records: Vec<_> = reader
            .deserialize()
            .map(|r| {
                let record: OomiRecord = r.unwrap();
                record
            })
            .collect();

        records
    }
}
