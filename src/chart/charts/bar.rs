use std::f64::consts::PI;

use kurbo::{Affine, Point, Rect};
use plotters::prelude::{ChartBuilder, DrawingBackend};
use serde::Deserialize;

use super::{decide_steps, Result, StepLabel, XY};
use crate::{
    chart::{ChartInfo, ChartType, Dataset, DatasetMeta},
    render::{self, Colour, FontInfo, TextInfo},
};

pub type BarPoint = f64;
#[derive(Clone, Debug, Deserialize)]
pub struct BarChart {
    /// Spacing between block groups
    spacing: Option<f64>,
    /// Categories of blocks, appear along the x axis
    categories: Vec<String>,
    /// Step size for the y axis
    step: u32,
    /// Draw grid lines for the x axis? default: true
    lines: Option<bool>,
    /// Label for the y axis
    axis: Option<String>,
}
struct DrawingInfo {
    block_w: f64,
    nb_cats: f64,
    max_val: f64,
    spacing: f64,
    area: Rect,
    nb_blocks: f64,
}
fn max_val(datasets: &Vec<Dataset<f64>>) -> f64 {
    datasets
        .iter()
        .flat_map(|dset| dset.values.iter().map(|v| v.ceil() as u64))
        .max()
        .unwrap() as f64
}
impl DrawingInfo {
    fn block_gap(&self) -> f64 {
        self.block_w * self.nb_blocks + self.spacing * 2.0
    }
    fn block_h(&self, v: f64) -> f64 {
        (self.area.height() / self.max_val) * v
    }
    fn new(datasets: &Vec<Dataset<f64>>, area: Rect, spacing: f64) -> Result<Self> {
        let nb_cats = datasets
            .iter()
            .map(|dset| dset.values.len())
            .fold(Ok(0), |r, l| {
                r.and_then(|rl| {
                    if rl != 0 && rl != l {
                        Err(render::Error::InvalidDatasets(
                            "datasets must all be the same size".to_owned(),
                        ))
                    } else {
                        Ok(l)
                    }
                })
            })? as f64;
        let nb_blocks = datasets.len() as f64;
        let free_width = area.width() - (nb_cats) * spacing * 2.0;
        if free_width < nb_blocks * nb_cats {
            return Err(render::Error::NotEnoughSpace(
                nb_blocks,
                free_width,
                "free width for blocks".to_owned(),
            ));
        }
        let max_val = max_val(datasets);
        let block_w = free_width / (nb_blocks * nb_cats);
        Ok(Self {
            block_w,
            nb_cats,
            max_val,
            spacing,
            area,
            nb_blocks,
        })
    }
    fn block_rect(&self, dataset: usize, num: usize, v: f64) -> Rect {
        let start_x = num as f64 * self.block_gap()
            + dataset as f64 * self.block_w
            + self.area.min_x()
            + self.spacing;
        let start_y = self.area.min_y();

        let r = Rect::new(
            start_x,
            start_y,
            start_x + self.block_w,
            start_y + self.block_h(v),
        );
        assert!(
            self.area.union(r).area().floor() <= self.area.area().floor(),
            "block ({:?}) inside draw area ({:?})",
            r,
            self.area
        );
        r
    }
    fn cat_xbounds(&self, cat: usize) -> (Point, Point) {
        let start_x = cat as f64 * self.block_gap();
        let end_x = start_x + self.nb_blocks * self.block_w;
        ((start_x, 0.0).into(), (end_x, 0.0).into())
    }
}

impl BarChart {
    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            spacing: None,
            categories: Vec::default(),
            step: 10,
            lines: None,
            axis: None,
        }
    }
    fn lines(&self) -> bool {
        self.lines.to_owned().unwrap_or(true)
    }

    fn spacing(&self) -> f64 {
        self.spacing.to_owned().unwrap_or(5.0)
    }
}
impl ChartType for BarChart {
    type DataPoint = BarPoint;

    const NAME: &'static str = "bar";

    fn render_datasets<DB: DrawingBackend>(
        &self,
        info: &ChartInfo<f64>,
        area: &kurbo::Rect,
        c: &mut ChartBuilder<DB>,
    ) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
}
