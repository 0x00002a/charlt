mod traits;
pub use traits::*;
pub mod svg;

pub enum Colour {
    RGB(u8, u8, u8),
    HEX(String),
}
pub struct Entity {
    colour: Colour,
    shape: geo::Geometry,
}

impl Colour {
    fn to_hex(&self) -> String {
        match &self {
            Colour::RGB(r, g, b) => format!("#{:2x}{:2x}{:2x}", r, g, b),
            Colour::HEX(h) => h.to_owned(),
        }
    }
}
