use kurbo::{Affine, Circle, Point, Rect};
use piet::{RenderContext, Text, TextAlignment, TextLayout, TextLayoutBuilder};

use crate::render::{self, RenderContextExt};
use crate::render::{Render, TextInfo};

use super::Chart;

use super::*;
type Result<T> = std::result::Result<T, render::Error>;

fn render_legend(
    sets: &Vec<DatasetMeta>,
    font: &FontInfo,
    r: &mut impl RenderContext,
) -> Result<()> {
    let mut ys = Vec::with_capacity(sets.len());
    let mut max_x: f64 = 0.0;
    let mut last_y = 0.0;
    let mut texts = Vec::with_capacity(sets.len());
    for s in sets {
        let txt = r.text().new_text_layout(s.name.to_owned()).build()?;
        let size = txt.size();
        ys.push(last_y);
        texts.push(
            TextInfo::new(s.name.to_owned())
                .font(font.to_owned())
                .alignment(TextAlignment::Start)
                .colour(Colour::BLACK),
        );
        max_x = max_x.max(size.width);
        last_y += size.height;
    }
    let ch_h = if ys.len() > 1 { ys[1] - ys[0] } else { last_y };
    for (n, y) in ys.iter().enumerate() {
        let pod_b = r.solid_brush(sets[n].colour.with_alpha(0.5));
        let dot_b = r.solid_brush(Colour::BLACK);
        let margin = ch_h * 0.6;
        r.fill(
            Rect::new(-ch_h, y - margin, max_x + ch_h * 0.3, y + margin).to_rounded_rect(3.0),
            &pod_b,
        );
        let dot_r = ch_h * 0.2;
        r.fill(
            Circle::new((-dot_r - ch_h * 0.3, y.to_owned()), dot_r),
            &dot_b,
        );
        r.render_text(Point::new(0.0, y.to_owned()), &texts[n])?;
    }
    Ok(())
}

impl<C: ChartType> Render for Chart<C, C::DataPoint> {
    type Error = render::Error;
    fn render<P: RenderContext>(&self, area: &Rect, r: &mut P) -> Result<()> {
        self.extra.render_datasets(&self.info, area, r)?;
        r.with_restore(|r| {
            r.transform(Affine::translate((area.width() * 0.9, area.height() * 0.1)));
            render_legend(
                &self
                    .info
                    .datasets
                    .iter()
                    .map(|d| d.extra.to_owned())
                    .collect(),
                &self.info.font(),
                r,
            )
        })??;
        Ok(())
    }
}
impl Render for Charts {
    type Error = render::Error;
    fn render<P: RenderContext>(&self, area: &Rect, r: &mut P) -> Result<()> {
        match &self {
            Charts::XYScatter(c) => c.render(area, r),
            Charts::Bar(c) => c.render(area, r),
        }
    }
}
