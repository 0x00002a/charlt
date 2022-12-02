pub mod bar;
pub mod xyscatter;

use std::{f64::consts::PI, rc::Rc};

use kurbo::{Affine, BezPath, Line, Point, Rect, Shape, Size, TranslateScale, Vec2};
use piet::{RenderContext, Text, TextAlignment, TextLayout, TextLayoutBuilder};
use rlua::{FromLua, Value};
use scopeguard::defer;
use serde::Deserialize;

use crate::{
    render::{self, colours, Colour, FontInfo, RenderContextExt, TextInfo},
    utils::RoundMul,
};

use self::{
    bar::BarPoint,
    xyscatter::{XYPoint, XYScatter},
};

use super::{Chart, ChartType, Dataset, DatasetMeta};

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Charts {
    #[serde(rename = "xy-scatter")]
    XYScatter(Chart<XYScatter, XYPoint<f64>>),
    #[serde(rename = "bar")]
    Bar(Chart<bar::BarChart, BarPoint>),
}

type Result<T> = std::result::Result<T, render::Error>;

fn to_dataset<T: Clone>(vs: &Vec<Vec<T>>) -> Vec<Dataset<T>> {
    vs.iter()
        .map(|p| Dataset {
            extra: DatasetMeta {
                name: "testpt".to_owned(),
                colour: Colour::RGB(0, 0, 0),
                thickness: 0.0,
            },
            values: p.clone(),
        })
        .collect()
}
