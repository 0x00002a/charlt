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
    fn render_datasets<P: Holds<Item = Vec<Self::DataPoint>>>(&self, datasets: &Vec<P>) -> Vec<P>;
}

impl<C: ChartType> Chart<C> {
    pub fn chart_type(&self) -> &str {
        &C::NAME
    }
}
impl<L, R> Holds for (L, R) {
    type Item = R;
    type Out<T> = (L, T);

    fn map<T, F: FnOnce(Self::Item) -> T>(self, f: F) -> Self::Out<T> {
        (self.0, f(self.1))
    }
}
