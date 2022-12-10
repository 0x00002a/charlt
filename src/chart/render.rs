use kurbo::{Affine, Circle, Point, Rect};

use crate::render::Render;

use super::Chart;

use super::*;
type Result<T> = std::result::Result<T, crate::render::Error>;

impl<C: ChartType> Render for Chart<C, C::DataPoint> {
    type Error = crate::render::Error;
    fn render<DB: DrawingBackend>(&self, area: &Rect, c: &mut ChartBuilder<DB>) -> Result<()> {
        let tiinfo = self.info.font();
        let tfont = tiinfo.into_text_style();
        let mut chart = self.extra.render_datasets(&self.info, area, c)?;
        chart
            .configure_series_labels()
            .position(plotters::prelude::SeriesLabelPosition::UpperRight)
            .label_font(tfont.clone())
            .draw()?;
        Ok(())
    }
}
impl Render for Charts {
    type Error = crate::render::Error;
    fn render<DB: DrawingBackend>(&self, area: &Rect, r: &mut ChartBuilder<DB>) -> Result<()> {
        match &self {
            Charts::XYScatter(c) => c.render(area, r),
            Charts::Bar(c) => c.render(area, r),
        }
    }
}
