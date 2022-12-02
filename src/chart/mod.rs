mod charts;
mod render;
use kurbo::Rect;
use piet::RenderContext;
use serde::Deserialize;

pub use charts::*;
pub use render::*;

use crate::render::{Colour, FontInfo};

#[derive(Clone, Debug, Deserialize)]
pub struct DatasetMeta {
    name: String,
    colour: Colour,
    #[serde(default = "default_line_thickness")]
    thickness: f64,
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
pub struct Chart<C, Pt: Clone> {
    pub datasets: Vec<Dataset<Pt>>,

    #[serde(flatten)]
    pub extra: C,

    pub font: Option<FontInfo>,
}
pub trait ChartType: Clone {
    type DataPoint: Clone;
    const NAME: &'static str;
    fn render_datasets<R: RenderContext>(
        &self,
        datasets: &Vec<Dataset<Self::DataPoint>>,
        area: &Rect,
        label_font: &FontInfo,
        r: &mut R,
    ) -> Result<(), crate::render::Error>;
}
