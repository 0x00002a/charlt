mod charts;
mod render;
use kurbo::Rect;
use plotters::prelude::{
    Cartesian2d, ChartBuilder, ChartContext, CoordTranslate, DrawingBackend, Ranged,
};
use serde::Deserialize;

pub use charts::*;
pub use render::*;

use crate::render::{Colour, FontInfo};

#[derive(Clone, Debug, Deserialize)]
pub struct DatasetMeta {
    name: String,
    #[serde(with = "serde_colour", alias = "color")]
    colour: Colour,
    #[serde(default = "default_line_thickness")]
    thickness: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct XY<T> {
    x: T,
    y: T,
}

impl<T> XY<T> {
    pub fn new<T1: Into<T>, T2: Into<T>>(x: T1, y: T2) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }
    pub fn map<R>(&self, f: impl Fn(&T) -> R) -> XY<R> {
        XY {
            x: f(&self.x),
            y: f(&self.y),
        }
    }
}
impl From<XY<f64>> for kurbo::Point {
    fn from(pt: XY<f64>) -> Self {
        Self::new(pt.x, pt.y)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Dataset<T: Clone> {
    values: Vec<T>,
    #[serde(flatten)]
    extra: DatasetMeta,
}
fn default_line_thickness() -> f64 {
    return 1.5;
}
#[derive(Clone, Debug, Deserialize)]
pub struct ChartInfo<Pt: Clone> {
    datasets: Vec<Dataset<Pt>>,
    font: Option<FontInfo>,
    margins: Option<XY<Option<f64>>>,
}
impl<Pt: Clone> ChartInfo<Pt> {
    fn font(&self) -> FontInfo {
        self.font.to_owned().unwrap_or_default()
    }

    fn margins(&self) -> XY<f64> {
        const DEFAULT: XY<f64> = {
            let x = 5.0;
            let y = 10.0;
            XY { x, y }
        };
        self.margins
            .to_owned()
            .map(|xy| XY {
                x: xy.x.unwrap_or(DEFAULT.x),
                y: xy.y.unwrap_or(DEFAULT.y),
            })
            .unwrap_or(DEFAULT)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Chart<C, Pt: Clone> {
    #[serde(flatten)]
    extra: C,
    #[serde(flatten)]
    info: ChartInfo<Pt>,
}
pub trait ChartType: Clone {
    type DataPoint: Clone;
    type X: Ranged;
    type Y: Ranged;
    fn render_datasets<'a, 'b, DB: DrawingBackend>(
        &self,
        info: &ChartInfo<Self::DataPoint>,
        c: &mut ChartBuilder<'a, 'b, DB>,
    ) -> Result<ChartContext<'a, DB, Cartesian2d<Self::X, Self::Y>>, crate::render::Error>;
}
mod serde_colour {
    use serde::{de::Error, Deserialize, Deserializer};

    use crate::render::Colour;
    use css_color_parser::Color;

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Colour, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let c = s
            .parse::<Color>()
            .map_err(|e| D::Error::custom(e.to_string()))?;
        Ok(plotters::style::RGBAColor(c.r, c.g, c.b, c.a as f64))
    }
}
