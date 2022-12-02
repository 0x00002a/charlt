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
    #[serde(with = "serde_colour")]
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
mod serde_colour {
    use serde::{Deserialize, Deserializer};

    use crate::render::Colour;

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
        Colour::from_hex_str(&s).map_err(serde::de::Error::custom)
    }
}
