use anyhow::Result;
use kurbo::{Affine, Point, Rect, Size};
use plotters::prelude::{ChartBuilder, DrawingBackend};

use super::Error;

pub trait Render {
    type Error: std::error::Error;
    fn render<DB: DrawingBackend>(
        &self,
        area: &Rect,
        r: &mut ChartBuilder<DB>,
    ) -> Result<(), Self::Error>;
}
