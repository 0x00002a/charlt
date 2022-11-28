mod charts;
mod fromlua;
mod render;
use std::ops::{Deref, DerefMut};

pub use charts::*;
pub use fromlua::*;
pub use render::*;

use crate::{
    render::{Colour, Entity},
    utils::Holds,
};
#[derive(Clone, Debug)]
pub struct DataPoint<T: Clone> {
    name: String,
    values: Vec<T>,
    colour: Colour,
}
#[derive(Clone, Debug)]
pub struct Chart<C: ChartType> {
    pub datasets: Vec<DataPoint<C::DataPoint>>,
    pub extra: C,
}
pub trait ChartType: Clone {
    type DataPoint: Clone;
    const NAME: &'static str;
    fn render_datasets(
        &self,
        datasets: &Vec<DataPoint<Self::DataPoint>>,
        area: &geo::Rect,
    ) -> Vec<Entity>;
}

impl<C: ChartType> Chart<C> {
    pub fn chart_type(&self) -> &str {
        &C::NAME
    }
}
