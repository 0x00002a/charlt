mod charts;
mod render;

use plotters::prelude::{Cartesian2d, ChartBuilder, ChartContext, DrawingBackend, Ranged};
use serde::Deserialize;

pub use charts::*;
pub use render::*;

use crate::render::{CssColour, FontInfo};

#[derive(Deserialize, Debug)]
#[serde(remote = "plotters::style::RGBAColor")]
struct RGBAColorDef(u8, u8, u8, f64);

#[derive(Clone, Debug, Deserialize)]
pub struct DatasetMeta {
    name: String,
    #[serde(alias = "color")]
    colour: Option<CssColour>,
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
#[derive(Clone, Debug, Deserialize)]
pub struct ChartInfo<Pt: Clone> {
    datasets: Vec<Dataset<Pt>>,
    font: Option<FontInfo>,
    margins: Option<XY<Option<f64>>>,
    caption: Option<String>,
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

    pub fn caption(&self) -> String {
        self.caption.to_owned().unwrap_or("".to_owned())
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
