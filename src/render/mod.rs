mod traits;

use std::fmt::Display;

use kurbo::Affine;
use piet::{FontFamily, RenderContext, Text};
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
#[serde(untagged)]
pub enum FontType {
    #[serde(skip)]
    Family(FontFamily),
    Named(String),
}
impl From<String> for FontType {
    fn from(s: String) -> Self {
        Self::Named(s)
    }
}
impl From<FontFamily> for FontType {
    fn from(f: FontFamily) -> Self {
        Self::Family(f)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct FontInfo {
    pub family: FontType,
    pub size: f64,
}
impl FontType {
    pub fn to_family<R: RenderContext>(self, r: &mut R) -> Result<FontFamily, Error> {
        match self {
            FontType::Family(f) => Ok(f),
            FontType::Named(n) => r.text().font_family(&n).ok_or(Error::FontLoading(n)),
        }
    }
}

impl Default for FontInfo {
    fn default() -> Self {
        Self {
            family: FontFamily::SYSTEM_UI.into(),
            size: 12.0,
        }
    }
}
impl Display for FontType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            FontType::Family(fam) => write!(f, "calculated family '{}'", fam.name()),
            FontType::Named(n) => write!(f, "named family '{}'", n),
        }
    }
}

pub struct TextInfo {
    font: Option<FontInfo>,
    content: String,
    colour: Option<piet::Color>,
    alignment: Option<piet::TextAlignment>,
    affine: Affine,
}

impl Default for TextInfo {
    fn default() -> Self {
        Self::new(String::default())
    }
}

impl TextInfo {
    pub fn new(content: String) -> Self {
        Self {
            content,
            font: None,
            colour: None,
            alignment: None,
            affine: Affine::default(),
        }
    }
    pub fn content<S: AsRef<str>>(mut self, c: S) -> Self {
        self.content = c.as_ref().to_owned();
        self
    }
    pub fn transform(mut self, t: Affine) -> Self {
        self.affine *= t;
        self
    }
    pub fn colour<C: Into<piet::Color>>(mut self, c: C) -> Self {
        self.colour = Some(c.into());
        self
    }
    pub fn font(mut self, f: FontInfo) -> Self {
        self.font = Some(f);
        self
    }
    pub fn alignment(mut self, a: piet::TextAlignment) -> Self {
        self.alignment = Some(a);
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to load font {0}")]
    FontLoading(String),
    #[error("failed to build text {0}")]
    TextBuild(piet::Error),
    #[error("empty dataset")]
    EmptyDataset,
    #[error("piet error: {0}")]
    Piet(piet::Error),
    #[error("not enough space, need at least {0} got {0}")]
    NotEnoughSpace(f64, f64),
    #[error("datasets are invalid {0}")]
    InvalidDatasets(String),
}
unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl From<piet::Error> for Error {
    fn from(e: piet::Error) -> Self {
        Self::Piet(e)
    }
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
