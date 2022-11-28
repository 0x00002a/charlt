mod fromlua;
mod traits;

pub use fromlua::*;
use geo::{Coord, Rect};
use serde::Deserialize;
pub use traits::*;
pub mod svg;

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Colour {
    RGB(u8, u8, u8),
    HEX(String),
}
pub mod colours {
    use super::Colour;

    pub const BLACK: Colour = Colour::RGB(0, 0, 0);
    pub const GREY: Colour = Colour::RGB(128, 128, 128);
}

pub enum Shape {
    Geo(geo::Geometry),
    Text {
        pos: Coord,
        content: String,
        rotation: Option<f64>,
    },
}

impl From<geo::Geometry> for Shape {
    fn from(g: geo::Geometry) -> Self {
        Shape::Geo(g)
    }
}

pub struct Entity {
    pub colour: Colour,
    pub shape: Shape,
}

impl Entity {
    pub fn new(colour: Colour, shape: Shape) -> Self {
        Self { colour, shape }
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("not enough space to render")]
    NotEnoughSpace { needed: Rect, provided: Rect },
}

impl Colour {
    fn to_hex(&self) -> String {
        match &self {
            Colour::RGB(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
            Colour::HEX(h) => h.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn colour_rgb_to_hex() {
        assert_eq!(Colour::RGB(0, 255, 5).to_hex(), "#00ff05");
    }
}
