mod charts;
mod fromlua;
mod render;
use kurbo::Rect;
use piet::RenderContext;
use serde::{
    de::{self, DeserializeOwned},
    Deserialize,
};
use std::ops::{Deref, DerefMut};

pub use charts::*;
pub use fromlua::*;
pub use render::*;

use crate::{
    render::{Colour, FontInfo},
    utils::Holds,
};

#[derive(Clone, Debug, Deserialize)]
pub struct DataPointMeta {
    name: String,
    colour: Colour,
    #[serde(default = "default_line_thickness")]
    thickness: f64,
}
#[derive(Clone, Debug, Deserialize)]
pub struct DataPoint<T: Clone> {
    values: Vec<T>,
    #[serde(flatten)]
    extra: DataPointMeta,
}
fn default_line_thickness() -> f64 {
    return 1.5;
}
#[derive(Clone, Debug, Deserialize)]
pub struct Chart<C, Pt: Clone> {
    pub datasets: Vec<DataPoint<Pt>>,

    #[serde(flatten)]
    pub extra: C,

    pub font: Option<FontInfo>,
}
pub trait ChartType: Clone {
    type DataPoint: Clone;
    const NAME: &'static str;
    fn render_datasets<R: RenderContext>(
        &self,
        datasets: &Vec<DataPoint<Self::DataPoint>>,
        area: &Rect,
        label_font: &FontInfo,
        r: &mut R,
    ) -> Result<(), crate::render::Error>;
}
