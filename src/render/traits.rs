use plotters::prelude::{ChartBuilder, DrawingBackend};

pub trait Render {
    type Error: std::error::Error;
    fn render<DB: DrawingBackend>(&self, r: &mut ChartBuilder<DB>) -> Result<(), Self::Error>;
}
