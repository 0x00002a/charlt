pub mod bar;
pub mod xyscatter;

use kurbo::{Line, Rect};
use serde::Deserialize;

use crate::render::{self, Colour};

use self::{bar::BarPoint, xyscatter::XYScatter};
use super::{Chart, Dataset, DatasetMeta, XY};

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Charts {
    #[serde(rename = "xy-scatter")]
    XYScatter(Chart<XYScatter, XY<f64>>),
    #[serde(rename = "bar")]
    Bar(Chart<bar::BarChart, BarPoint>),
}

type Result<T> = std::result::Result<T, render::Error>;

#[allow(unused)]
fn to_dataset<T: Clone>(vs: &Vec<Vec<T>>) -> Vec<Dataset<T>> {
    vs.iter()
        .map(|p| Dataset {
            extra: DatasetMeta {
                name: "testpt".to_owned(),
                colour: Colour::BLACK,
                thickness: 0.0,
            },
            values: p.clone(),
        })
        .collect()
}

fn mk_grids(grid: &XY<bool>, steps: &XY<Vec<u64>>, bounds: &Rect) -> Vec<Line> {
    let mut out = Vec::new();
    let mut do_iter = |steps: &Vec<u64>, f: &dyn Fn(f64) -> ((f64, f64), (f64, f64))| {
        for pt in steps {
            let (x, y) = f(pt.to_owned() as f64);
            let line = Line::new(x, y);
            out.push(line);
        }
    };
    if grid.x {
        do_iter(&steps.x, &|x| {
            (
                (x + bounds.min_x(), bounds.min_y()),
                (x + bounds.min_x(), bounds.max_y()),
            )
        });
    }
    if grid.y {
        do_iter(&steps.y, &|y| {
            (
                (bounds.min_x(), bounds.max_y() - y),
                (bounds.max_x(), bounds.max_y() - y),
            )
        });
    }
    out
}
