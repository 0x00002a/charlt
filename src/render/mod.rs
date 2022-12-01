mod traits;

use piet::FontFamily;
use serde::Deserialize;
pub use traits::*;

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

impl From<Colour> for piet::Color {
    fn from(c: Colour) -> Self {
        match c {
            Colour::RGB(r, g, b) => Self::rgb8(r, g, b),
            Colour::HEX(h) => Self::from_hex_str(&h).unwrap(),
        }
    }
}
impl From<piet::Color> for Colour {
    fn from(c: piet::Color) -> Self {
        let (r, g, b, _) = c.as_rgba8();
        Self::RGB(r, g, b)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct FontInfo {
    family: String,
    size: f64,
}
impl Default for FontInfo {
    fn default() -> Self {
        Self {
            family: FontFamily::SYSTEM_UI.name().to_owned(),
            size: 12.0,
        }
    }
}

pub struct TextInfo {
    font: Option<FontInfo>,
    content: String,
    colour: Option<piet::Color>,
}

impl TextInfo {
    pub fn new(content: String) -> Self {
        Self {
            content,
            font: None,
            colour: None,
        }
    }
    pub fn colour<C: Into<piet::Color>>(mut self, c: C) -> Self {
        self.colour = Some(c.into());
        self
    }
    pub fn font(mut self, f: FontInfo) -> Self {
        self.font = Some(f);
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to load font {0}")]
    FontLoading(String),
    #[error("failed to build text {0}")]
    TextBuild(piet::Error),
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
