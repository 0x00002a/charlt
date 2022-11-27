mod charts;
mod fromlua;
mod render;
use std::ops::{Deref, DerefMut};

pub use charts::*;
pub use fromlua::*;
pub use render::*;

use crate::render::{Colour, Entity};
pub struct DataPoint<T> {
    name: String,
    values: Vec<T>,
    colour: Colour,
}

pub struct Chart<C: ChartType> {
    pub datasets: Vec<DataPoint<C::DataPoint>>,
    pub extra: C,
}
trait ChartType {
    type DataPoint;
    const NAME: &'static str;
    fn render_series(&self, datasets: &Vec<Self::DataPoint>) -> Vec<geo::Geometry>;
}

impl<C: ChartType> Chart<C> {
    pub fn chart_type(&self) -> &str {
        &C::NAME
    }
}
