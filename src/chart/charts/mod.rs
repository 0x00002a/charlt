pub mod bar;
pub mod xyscatter;

use kurbo::{Line, Rect};
use serde::Deserialize;

use crate::{
    render::{self, Colour},
    utils::RoundMul,
};

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

fn step_adjust(area: &Rect, steps: &XY<u32>) -> Rect {
    Rect::new(
        area.min_x(),
        area.max_y() - area.height().ceil_mul(steps.y as f64),
        area.min_x() + area.width().ceil_mul(steps.x as f64),
        area.max_y(),
    )
}

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

#[derive(PartialEq, Clone, Debug)]
struct StepLabel<T = f64> {
    value: String,
    offset: T,
}

impl<T> StepLabel<T> {
    fn new(value: impl ToString, offset: T) -> Self {
        Self {
            value: value.to_string(),
            offset,
        }
    }
}
impl Into<u64> for StepLabel<f64> {
    fn into(self) -> u64 {
        self.offset.ceil() as u64
    }
}

fn decide_steps(len: f64, min_val: f64, max_val: f64, step: u32) -> Vec<StepLabel<f64>> {
    let range = max_val - min_val;
    let offset_step = len / (range / step as f64);

    (min_val.floor_mul(step as f64) as u64..(max_val.ceil_mul(step as f64) as u64 + step as u64))
        .step_by(step as usize)
        .enumerate()
        .map(|(i, s)| StepLabel::new(s as f64, offset_step * i as f64))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decide_steps() {
        let inputs = vec![(100.0, 0.0, 500.0, 100), (500.0, 0.0, 100.0, 20)];
        let expected = vec![
            vec![
                StepLabel::new(0.0, 0.0),
                StepLabel::new(100.0, 20.0),
                StepLabel::new(200.0, 40.0),
                StepLabel::new(300.0, 60.0),
                StepLabel::new(400.0, 80.0),
                StepLabel::new(500.0, 100.0),
            ],
            vec![
                StepLabel::new(0.0, 0.0),
                StepLabel::new(20.0, 100.0),
                StepLabel::new(40.0, 200.0),
                StepLabel::new(60.0, 300.0),
                StepLabel::new(80.0, 400.0),
                StepLabel::new(100.0, 500.0),
            ],
        ];
        for i in 0..inputs.len() {
            let (len, min_val, max_val, min_gap) = inputs[i];
            let output = decide_steps(len, min_val, max_val, min_gap);
            for v in &expected[i] {
                assert!(output.contains(v), "{:?} contains {:?}", output, v);
            }
        }
    }
    #[test]
    fn test_step_adjust() {
        let inputs = vec![(9.0, 9.0), (9.0, 9.0)];
        let muls = vec![(5u32, 5u32), (1u32, 1u32)];
        let expected = vec![(10.0, 10.0), (9.0, 9.0)];
        for i in 0..inputs.len() {
            let steps = XY::new(muls[i].0, muls[i].1);
            let area = Rect::new(0.0, 0.0, inputs[i].0, inputs[i].1);
            let adjusted = step_adjust(&area, &steps);
            assert_eq!(
                (adjusted.width(), adjusted.height()),
                expected[i],
                "non-matching: {}",
                adjusted
            );
        }
    }
}
