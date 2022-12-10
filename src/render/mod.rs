mod traits;

use std::fmt::Debug;

use plotters::style::{FontFamily, TextStyle};
use serde::Deserialize;
pub use traits::*;

pub type Colour = plotters::style::RGBAColor;

impl Default for FontInfo {
    fn default() -> Self {
        Self {
            family: FontFamily::SansSerif.into(),
            size: 12f64,
        }
    }
}
#[derive(Clone)]
pub struct FontStore(String);
impl FontStore {
    pub fn family<'a>(&'a self) -> FontFamily<'a> {
        FontFamily::Name(self.0.as_str())
    }
}

#[derive(Clone, Deserialize)]
pub enum FontType {
    #[serde(with = "font_family_serde")]
    Store(FontStore),
    #[serde(skip)]
    Family(FontFamily<'static>),
}
impl From<FontFamily<'static>> for FontType {
    fn from(f: FontFamily<'static>) -> Self {
        Self::Family(f)
    }
}
impl Debug for FontType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(arg0) => f.write_str(arg0.0.as_str()),
            Self::Family(arg0) => f.write_str(arg0.as_str()),
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct FontInfo {
    pub family: FontType,
    pub size: f64,
}
impl FontInfo {
    pub fn into_text_style<'a>(&'a self) -> TextStyle<'a> {
        (
            match &self.family {
                FontType::Store(s) => s.family(),
                FontType::Family(f) => f.clone(),
            },
            self.size,
        )
            .into()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to load font {0}")]
    FontLoading(String),
    #[error("plotter drawing error: {0}")]
    PlottersDraw(String),
}
impl<E: std::error::Error + Send + Sync> From<plotters::drawing::DrawingAreaErrorKind<E>>
    for Error
{
    fn from(e: plotters::drawing::DrawingAreaErrorKind<E>) -> Self {
        Self::PlottersDraw(e.to_string())
    }
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

mod font_family_serde {

    use serde::{de::Error, Deserialize, Deserializer};

    use css_color_parser::Color;

    use super::FontStore;

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<FontStore, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let _c = s
            .parse::<Color>()
            .map_err(|e| D::Error::custom(e.to_string()))?;
        Ok(FontStore(s))
    }
}
