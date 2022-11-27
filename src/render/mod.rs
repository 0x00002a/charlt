mod fromlua;
mod traits;

pub use fromlua::*;
use geo::Rect;
pub use traits::*;
pub mod svg;

#[derive(Clone, Debug)]
pub enum Colour {
    RGB(u8, u8, u8),
    HEX(String),
}
pub struct Entity {
    pub colour: Colour,
    pub shape: geo::Geometry,
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
