mod fromlua;
mod traits;

use font_kit::font::Font;
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
        font: Option<Font>,
    },
}

impl Shape {
    fn text_with_font<P: Into<Coord>, C: ToString>(pos: P, content: C, font: Font) -> Self {
        Shape::Text {
            pos: pos.into(),
            content: content.to_string(),
            rotation: None,
            font: Some(font),
        }
    }
    fn text<P: Into<Coord>, C: ToString>(pos: P, content: C) -> Self {
        Shape::Text {
            pos: pos.into(),
            content: content.to_string(),
            rotation: None,
            font: None,
        }
    }
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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not enough space to render")]
    FontLoading(anyhow::Error),
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
