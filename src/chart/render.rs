use crate::render::Render;

use super::Chart;

use super::*;

impl<C: ChartType> Render for Chart<C> {
    fn render(&self) -> Vec<Entity> {
        let plot = self
            .datasets
            .iter()
            .flat_map(|dp| {
                self.extra
                    .render_series(&dp.values)
                    .into_iter()
                    .map(|e| Entity {
                        colour: dp.colour,
                        shape: e,
                    })
            })
            .collect();
        plot
    }
}
