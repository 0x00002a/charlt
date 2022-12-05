use std::f64::consts::PI;

use crate::{
    chart::{ChartInfo, ChartType, Dataset, DatasetMeta},
    render::{self, FontInfo, TextInfo},
};

use super::{decide_steps, Result, StepLabel, XY};
use kurbo::{Affine, BezPath, Point, Rect, Shape, Size, TranslateScale};
use plotters::prelude::{ChartBuilder, DrawingBackend};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct XYScatter {
    /// Labels for the axis
    axis: XY<String>,
    /// Step sizes
    steps: XY<u32>,
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
    const NAME: &'static str = "xy-scatter";

    fn render_datasets<DB: DrawingBackend>(
        &self,
        info: &ChartInfo<Self::DataPoint>,
        area: &kurbo::Rect,
        r: &mut ChartBuilder<DB>,
    ) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use kurbo::{Line, Shape};

    use super::*;
}
