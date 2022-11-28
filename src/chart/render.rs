use geo::Rect;

use crate::render;
use crate::render::Render;

use super::Chart;

use super::*;

impl<C: ChartType> Render for Chart<C, C::DataPoint> {
    fn render(&self, area: &Rect) -> Result<Vec<render::Entity>, render::Error> {
        let plot_shapes = self.extra.render_datasets(&self.datasets, area);
        Ok(plot_shapes)
    }
}
impl Render for Charts {
    fn render(&self, area: &Rect) -> Result<Vec<render::Entity>, render::Error> {
        match &self {
            Charts::XYScatter(c) => c.render(area),
            Charts::Bar(c) => c.render(area),
        }
    }
}
