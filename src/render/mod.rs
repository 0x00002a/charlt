mod traits;
pub use traits::*;

pub enum Colour {
    RGB(u8, u8, u8),
    HEX(String),
}
pub struct Entity {
    colour: Colour,
    shape: geo::Geometry,
}

impl Colour {}
