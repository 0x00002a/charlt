pub mod bar;
pub mod xyscatter;

use plotters::prelude::Rectangle;
use serde::Deserialize;

use crate::render;

use self::{bar::BarPoint, xyscatter::XYScatter};
use super::{Chart, XY};

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Charts {
    #[serde(rename = "xy-scatter")]
    XYScatter(Chart<XYScatter, XY<f64>>),
    #[serde(rename = "bar")]
    Bar(Chart<bar::BarChart, BarPoint>),
}

type Result<T> = std::result::Result<T, render::Error>;

fn legend_for<C: plotters::style::Color>(
    (x, y): (i32, i32),
    c: C,
) -> plotters::element::Rectangle<(i32, i32)> {
    Rectangle::new([(x - 5, y - 5), (x + 20, y + 5)], c.filled())
}

#[cfg(test)]
mod tests {}
