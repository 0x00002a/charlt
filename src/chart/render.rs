use crate::render::Render;

use super::Chart;

use super::*;

impl<C: ChartType> Render for Chart<C> {
    fn render(&self) -> Vec<Entity> {
        let plot_shapes = self
            .extra
            .render_datasets(&self.datasets.iter().map(|s| s.values).collect());
        let shapes_rendered = plot_shapes
            .into_iter()
            .zip(self.datasets.iter())
            .flat_map(|(shapes, s)| {
                shapes.into_iter().map(|shape| Entity {
                    colour: s.colour,
                    shape: shape,
                })
            })
            .collect();
        shapes_rendered
    }
}
