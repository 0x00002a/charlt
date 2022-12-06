mod traits;

use std::fmt::{Debug, Display};

use kurbo::Affine;
use plotters::style::FontFamily;
use serde::Deserialize;
pub use traits::*;

pub type Colour = plotters::style::RGBAColor;

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum FontType {
    #[serde(skip)]
    Family(FontFamily<'static>),
    Named(String),
}
impl From<String> for FontType {
    fn from(s: String) -> Self {
        Self::Named(s)
    }
}
impl From<FontFamily<'static>> for FontType {
    fn from(f: FontFamily<'static>) -> Self {
        Self::Family(f)
    }
}
impl Debug for FontType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Family(arg0) => f.debug_tuple("Family").field(&arg0.as_str()).finish(),
            Self::Named(arg0) => f.debug_tuple("Named").field(arg0).finish(),
        }
    }
}
impl Default for FontInfo {
    fn default() -> Self {
        Self {
            family: FontFamily::SansSerif.into(),
            size: 12f64,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct FontInfo {
    pub family: FontType,
    pub size: f64,
}
impl Display for FontType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            FontType::Family(fam) => write!(f, "calculated family '{}'", fam.as_str()),
            FontType::Named(n) => write!(f, "named family '{}'", n),
        }
    }
}

pub struct TextInfo {
    font: Option<FontInfo>,
    content: String,
    colour: Option<Colour>,
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
    pub fn colour<C: Into<Colour>>(mut self, c: C) -> Self {
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
    #[error("empty dataset")]
    EmptyDataset,
    #[error("not enough space, need at least {0} got {1}: {2}")]
    NotEnoughSpace(f64, f64, String),
    #[error("datasets are invalid {0}")]
    InvalidDatasets(String),
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
