pub mod bar;
pub mod xyscatter;

use kurbo::{Line, Rect};
use more_asserts::debug_assert_le;
use plotters::prelude::Rectangle;
use serde::Deserialize;

#[cfg(test)]
use super::DatasetMeta;
#[cfg(test)]
use crate::render::Colour;

use crate::{render, utils::RoundMul};

use self::{bar::BarPoint, xyscatter::XYScatter};
use super::{Chart, Dataset, XY};

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Charts {
    #[serde(rename = "xy-scatter")]
    XYScatter(Chart<XYScatter, XY<f64>>),
    #[serde(rename = "bar")]
    Bar(Chart<bar::BarChart, BarPoint>),
}

type Result<T> = std::result::Result<T, render::Error>;

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
fn legend_for<C: plotters::style::Color>(
    (x, y): (i32, i32),
    c: C,
) -> plotters::element::Rectangle<(i32, i32)> {
    Rectangle::new([(x - 5, y - 5), (x + 20, y + 5)], c.filled())
}

fn decide_steps(len: f64, min_val: f64, max_val: f64, step: u32) -> Vec<StepLabel<f64>> {
    let range = max_val.ceil_mul(step as f64) - min_val.floor_mul(step as f64);
    let offset_step = len / (range / step as f64);

    let vs: Vec<_> = (min_val.floor_mul(step as f64) as i64
        ..(max_val.ceil_mul(step as f64) as i64 + step as i64))
        .step_by(step as usize)
        .enumerate()
        .map(|(i, s)| StepLabel::new(s as f64, offset_step * i as f64))
        .collect();
    debug_assert_le!(
        vs.iter().map(|s| s.offset.ceil() as u64).max().unwrap() as f64,
        len,
        "offset outside bounds: len {} min_val {} max_val {} step {} offset_step {}",
        len,
        min_val,
        max_val,
        step,
        offset_step
    );
    vs
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
}
