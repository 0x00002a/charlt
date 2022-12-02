use kurbo::Rect;
use piet::RenderContext;

use crate::render;
use crate::render::Render;

use super::Chart;

use super::*;

impl<C: ChartType> Render for Chart<C, C::DataPoint> {
    type Error = render::Error;
    fn render<P: RenderContext>(&self, area: &Rect, r: &mut P) -> Result<(), render::Error> {
        self.extra.render_datasets(&self.info, area, r)
    }
}
impl Render for Charts {
    type Error = render::Error;
    fn render<P: RenderContext>(&self, area: &Rect, r: &mut P) -> Result<(), render::Error> {
        match &self {
            Charts::XYScatter(c) => c.render(area, r),
            Charts::Bar(c) => c.render(area, r),
        }
    }
}
