use plotters::style::full_palette::GREY;
use plotters::style::{Color, FontDesc, FontFamily, FontStyle, WHITE};

use crate::render::{self, Render};

use super::Chart;

use super::*;
type Result<T> = std::result::Result<T, crate::render::Error>;

impl<C: ChartType> Render for Chart<C, C::DataPoint> {
    type Error = crate::render::Error;
    fn render<DB: DrawingBackend>(&self, c: &mut ChartBuilder<DB>) -> Result<()> {
        let margins = self.info.margins();
        c.margin_left(margins.x)
            .margin_right(margins.x)
            .margin_bottom(margins.y)
            .margin_top(margins.y);
        let tfont = FontDesc::new(FontFamily::SansSerif, 12.0, FontStyle::Normal);
        let chwidth = tfont
            .box_size("  ")
            .map_err(|e| render::Error::FontLoading(e.to_string()))?
            .0;
        let mut chart = self.extra.render_datasets(&self.info, c)?;
        chart
            .configure_series_labels()
            .position(plotters::prelude::SeriesLabelPosition::UpperRight)
            .label_font(tfont.clone())
            .background_style(WHITE.mix(0.8))
            .border_style(GREY.mix(0.6).stroke_width(1))
            .margin(chwidth * 4)
            .draw()?;
        Ok(())
    }
}
impl Render for Charts {
    type Error = crate::render::Error;
    fn render<DB: DrawingBackend>(&self, r: &mut ChartBuilder<DB>) -> Result<()> {
        match &self {
            Charts::XYScatter(c) => c.render(r),
            Charts::Bar(c) => c.render(r),
        }
    }
}
