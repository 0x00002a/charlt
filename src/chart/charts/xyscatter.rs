use crate::{
    chart::{ChartInfo, ChartType},
    palette::colours,
};

use super::{legend_for, Result, XY};

use plotters::{
    prelude::{Cartesian2d, ChartBuilder, ChartContext, DrawingBackend},
    series::LineSeries,
    style::{Color, FontFamily, WHITE},
};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct XYScatter {
    /// Labels for the axis
    axis: XY<String>,
    /// Draw grid lines? default: {x: true, y: false}
    grid: Option<XY<bool>>,
    /// Margin around plot (between plot and labels)
    margin: Option<XY<f64>>,
}

impl XYScatter {
    fn margin(&self) -> XY<f64> {
        self.margin.to_owned().unwrap_or(XY { x: 8.0, y: 10.0 })
    }
}

impl ChartType for XYScatter {
    type DataPoint = XY<f64>;
    type X = plotters::coord::types::RangedCoordf64;
    type Y = plotters::coord::types::RangedCoordf64;

    fn render_datasets<'a, 'b, DB: DrawingBackend>(
        &self,
        info: &ChartInfo<Self::DataPoint>,
        c: &mut ChartBuilder<'a, 'b, DB>,
    ) -> Result<ChartContext<'a, DB, Cartesian2d<Self::X, Self::Y>>> {
        let fiinfo = info.font();
        let tfont = fiinfo.into_text_style();

        let (min_x, min_y, max_x, max_y) = info
            .datasets
            .iter()
            .map(|s| {
                let max_x = s.values.iter().map(|v| v.x as u64).max().unwrap();
                let max_y = s.values.iter().map(|v| v.y as u64).max().unwrap();
                let min_x = s.values.iter().map(|v| v.x as u64).min().unwrap();
                let min_y = s.values.iter().map(|v| v.y as u64).min().unwrap();
                (min_x, min_y, max_x, max_y)
            })
            .reduce(|(lmix, lmiy, lmx, lmy), (rmix, rmiy, rmx, rmy)| {
                (lmix.min(rmix), lmiy.min(rmiy), lmx.max(rmx), lmy.max(rmy))
            })
            .unwrap();
        let margin = self.margin();
        let mut chart = c
            .set_left_and_bottom_label_area_size(50)
            .caption(info.caption(), FontFamily::SansSerif)
            .margin_left(margin.x)
            .margin_bottom(margin.y)
            .margin_right(10.0 + margin.x)
            .margin_top(10.0 + margin.y)
            .build_cartesian_2d(min_x as f64..max_x as f64, min_y as f64..max_y as f64)?;
        let mut mesh = chart.configure_mesh();
        let grid = self.grid.clone().unwrap_or(XY::new(false, true));
        if !grid.x {
            mesh.disable_x_mesh();
        }
        if !grid.y {
            mesh.disable_y_mesh();
        }
        mesh.bold_line_style(&WHITE.mix(0.3))
            .x_desc(self.axis.x.clone())
            .y_desc(self.axis.y.clone())
            .label_style(tfont.clone())
            .draw()?;
        let mut citer = colours();
        for dset in &info.datasets {
            let c = dset
                .extra
                .colour
                .map(|c| c.as_rgba())
                .unwrap_or_else(|| citer.next().unwrap().to_rgba());
            chart
                .draw_series(LineSeries::new(dset.values.iter().map(|v| (v.x, v.y)), c))?
                .label(dset.extra.name.clone())
                .legend(move |pt| legend_for(pt, c));
        }
        Ok(chart)
    }
}

#[cfg(test)]
mod tests {}
