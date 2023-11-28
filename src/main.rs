use chrono::prelude::*;
use chrono_tz::Europe::Helsinki;
use csv::Writer;
use plotters::chart::ChartBuilder;
use plotters::prelude::*;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use serde::Serialize;
use std::error::Error;
use std::fmt::Debug;
use std::path::PathBuf;

use clap::Parser;
use rust_decimal::Decimal;

use eleparserlib::{CumulativeComparisonData, plotter};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(long)]
    pub file_path: String,

    #[arg(value_enum, long)]
    pub file_type: ConsumptionFileType,

    #[arg(long)]
    pub start_date: NaiveDate,

    #[arg(long)]
    pub end_date: Option<NaiveDate>,

    #[arg(long)]
    pub timezone: Option<chrono_tz::Tz>,

    #[arg(long)]
    pub sample: Option<usize>,

}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ConsumptionFileType {
    Oomi,
    Fingrid,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    let file_path = PathBuf::from(&args.file_path);

    let timezone = args.timezone.unwrap_or(Helsinki);
    let start = timezone
        .from_local_datetime(&args.start_date.and_time(NaiveTime::from_hms(0, 0, 0)))
        .unwrap();
    let end = args
        .end_date
        .map(|d| {
            timezone
                .from_local_date(&d)
                .and_time(NaiveTime::from_hms(23, 59, 59))
                .unwrap()
        })
        .unwrap_or(Local::now().with_timezone(&timezone));

    let cumulative_series = eleparserlib::get_data(&file_path, &start.with_timezone(&Utc), &end.with_timezone(&Utc))?;

    let max_price = cumulative_series
        .iter()
        .map(|c| c.market_price_for_hour)
        .max()
        .and_then(|d| d.to_f64())
        .unwrap();

    let max_set = cumulative_series
        .iter()
        .map(|c| c.cumulative_set_price)
        .max()
        .and_then(|d| d.to_f64())
        .unwrap();

    let max_market = cumulative_series
        .iter()
        .map(|c| c.cumulative_market_price)
        .max()
        .and_then(|d| d.to_f64())
        .unwrap();

    let max_val = max_set.max(max_market);

    let set_series = cumulative_series.iter().map(|s| {
        (
            // s.date_time.signed_duration_since(start).num_days().to_f32().unwrap(),
            s.date_time,
            s.cumulative_set_price.to_f64().unwrap(),
        )
    });
    let market_series = cumulative_series.iter().map(|s| {
        (
            // s.date_time.signed_duration_since(start).num_days().to_f32().unwrap(),
            s.date_time,
            s.cumulative_market_price.to_f64().unwrap(),
        )
    });
    let hourly_price_series = cumulative_series.iter().map(|s| {
        (
            s.date_time,
            s.market_price_for_hour.to_f64().unwrap(),
        )
    });

    let sampler = args.sample.unwrap_or(1);
    plotter::draw_png(&start, &end, max_val, set_series.step_by(sampler), market_series.step_by(sampler))?;


    write_csv(&cumulative_series)?;
    println!("Written image to test.png");
    Ok(())
}

fn write_csv(data: &[CumulativeComparisonData]) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path("comparison.csv")?;
    for datum in data {
        writer.serialize(datum)?
    }
    Ok(())
}
