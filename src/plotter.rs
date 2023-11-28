use std::error::Error;
use std::ops::Range;
use chrono::{Date, DateTime, TimeZone, Utc};
use plotters::coord::ranged1d::{DefaultFormatting, KeyPointHint};
use plotters::prelude::*;
use rust_decimal::prelude::ToPrimitive;

#[derive(Debug)]
struct WrappedUtc {
    datetime: DateTime<Utc>
}

impl Ranged for WrappedUtc {
    type FormatOption = DefaultFormatting;
    type ValueType = DateTime<Utc>;

    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        todo!()
    }

    fn key_points<Hint: KeyPointHint>(&self, hint: Hint) -> Vec<Self::ValueType> {
        todo!()
    }

    fn range(&self) -> Range<Self::ValueType> {
        todo!()
    }
}
impl DiscreteRanged for WrappedUtc {

    fn size(&self) -> usize {
        todo!()
    }

    fn index_of(&self, value: &DateTime<Utc>) -> Option<usize> {
        todo!()
    }

    fn from_index(&self, index: usize) -> Option<DateTime<Utc>> {
        todo!()
    }
}

pub fn draw_png(
    start: &DateTime<impl TimeZone>,
    end: &DateTime<impl TimeZone>,
    max_val: f64,
    set_series: impl Iterator<Item = (DateTime<Utc>, f64)>,
    market_series: impl Iterator<Item = (DateTime<Utc>, f64)>,
) -> Result<(), Box<dyn Error>> {
    let width = 2000.0;
    let height = width / (16.0 / 9.0);
    let height = f64::floor(height).to_i32().unwrap();
    let width = 1000;
    let height = 800;
    let root_area = BitMapBackend::new("test.png", (width, height)).into_drawing_area();

    root_area.fill(&WHITE).unwrap();

    let title = format!("Hintavertailu {} - {}", start.date().naive_local(), end.date().naive_local());

    let mut chart = ChartBuilder::on(&root_area)
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 60)
        .caption(title, ("sans-serif", 40))
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 60)
        .margin(50)
        .build_cartesian_2d(
            start.with_timezone(&Utc)..end.with_timezone(&Utc),
            0.0..max_val,
        )?;

    let font_style = FontDesc::new(FontFamily::SansSerif, 16.0, FontStyle::Normal);

    chart
        .configure_mesh()
        .y_desc("Kumulatiivinen hinta")
        .y_label_formatter(&|l| format!("{l} €"))
        .x_desc("Päivämäärä")
        // .y_label_offset(200)
        .y_label_style(font_style.clone())
        .x_label_style(font_style.clone())
        .x_label_formatter(&|x| x.date().naive_local().to_string())
        // .disable_x_mesh()
        // .disable_y_mesh()
        .draw()?;

    chart
        .draw_series(LineSeries::new(set_series,RED))?
        .label("6,99 €/kWh")
        .legend(|(x, y)| Rectangle::new([(x - 15, y + 1), (x, y)], RED));

    chart
        .draw_series(LineSeries::new(market_series, BLUE))?
        .label("Pörssihinta")
        .legend(|(x, y)| Rectangle::new([(x - 15, y + 1), (x, y)], BLUE));


    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperLeft)
        .margin(20)
        .legend_area_size(5)
        .border_style(BLACK)
        .background_style(WHITE)
        .label_font(("Calibri", 20))
        .draw()
        .unwrap();


    Ok(())
}
