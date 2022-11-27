use geo::Rect;

use crate::render;
use crate::render::Render;

use super::Chart;

use super::*;

impl<C: ChartType> Render for Chart<C> {
    fn render(&self, area: &Rect) -> Result<Vec<render::Entity>, render::Error> {
        let plot_shapes = self.extra.render_datasets(
            &self.datasets.iter().map(|s| s.values.clone()).collect(),
            area,
        );
        let shapes_rendered = plot_shapes
            .into_iter()
            .zip(self.datasets.iter())
            .flat_map(|(shapes, s)| {
                shapes.into_iter().map(|shape| Entity {
                    colour: s.colour.clone(),
                    shape: shape,
                })
            })
            .collect();
        Ok(shapes_rendered)
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
