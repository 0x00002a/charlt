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
pub struct DataPoint<T: Clone> {
    name: String,
    values: Vec<T>,
    colour: Colour,
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
    );
}
