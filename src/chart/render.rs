use crate::render::Render;

use super::Chart;

use super::*;

impl<C: ChartType> Render for Chart<C> {
    fn render(&self) -> Vec<Entity> {
        self.extra
            .render_datasets(&self.datasets.iter().map(|s| s.values).collect())
            .into_iter()
            .enumerate()
            .map(|(n, s)| Entity {
                colour: dp.colour,
                shape: e,
            });
        let plot = self.datasets.iter().flat_map(|dp| {}).collect();
        plot
    }
}
