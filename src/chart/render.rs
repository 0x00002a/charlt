use kurbo::{Affine, Circle, Point, Rect};

use crate::render::{Render, TextInfo};

use super::Chart;

use super::*;
type Result<T> = std::result::Result<T, crate::render::Error>;

impl<C: ChartType> Render for Chart<C, C::DataPoint> {
    type Error = crate::render::Error;
    fn render<DB: DrawingBackend>(&self, area: &Rect, c: &mut ChartBuilder<DB>) -> Result<()> {
        self.extra.render_datasets(&self.info, area, c)
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
