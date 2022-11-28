use anyhow::anyhow;
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::{self, Source};
use geo::Rect;

use crate::render;
use crate::render::Render;

use super::Chart;

use super::*;

impl<C: ChartType> Render for Chart<C, C::DataPoint> {
    fn render(&self, area: &Rect) -> Result<Vec<render::Entity>, render::Error> {
        let font = match self
            .font
            .clone()
            .map(|f| {
                source::SystemSource::new()
                    .select_family_by_name(&f)?
                    .fonts()
                    .get(0)
                    .map(|h| h.clone())
                    .ok_or(anyhow!("font family is empty"))
            })
            .unwrap_or_else(|| {
                Ok(source::SystemSource::new()
                    .select_best_match(&[FamilyName::SansSerif], &Properties::new())?)
            })
            .and_then(|f| Ok(f.load()?))
        {
            Err(e) => Err(render::Error::FontLoading(e)),
            Ok(v) => Ok(v),
        }?;
        let plot_shapes = self.extra.render_datasets(&self.datasets, area, &font);
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
