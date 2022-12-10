
use std::ops::Range;


use plotters::coord::ranged1d::{NoDefaultFormatting, ValueFormatter};
use plotters::prelude::{Cartesian2d, ChartContext, Ranged};
use plotters::style::{FontFamily, TextStyle};
use plotters::{element::Drawable};
use plotters::{
    prelude::{
        ChartBuilder, DrawingBackend, Rectangle,
    },
    style::{Color, ShapeStyle, WHITE},
};
use serde::Deserialize;

use super::{legend_for, Result};
use crate::{
    chart::{ChartInfo, ChartType, Dataset},
};

pub type BarPoint = f64;
#[derive(Clone, Debug, Deserialize)]
pub struct BarChart {
    /// Spacing between block groups
    spacing: Option<f64>,
    /// Categories of blocks, appear along the x axis
    categories: Vec<String>,
    /// Draw grid lines for the x axis? default: true
    lines: Option<bool>,
    /// Label for the y axis
    axis: Option<String>,
}
trait ToElement<T: Drawable<DB>, DB: DrawingBackend> {
    fn to_element<S: Into<ShapeStyle>>(self, style: S) -> T;
}

impl<DB: DrawingBackend> ToElement<plotters::element::Rectangle<kurbo::Point>, DB> for kurbo::Rect {
    fn to_element<S: Into<ShapeStyle>>(self, s: S) -> plotters::element::Rectangle<kurbo::Point> {
        plotters::element::Rectangle::new([(self.x0, self.y0).into(), (self.x1, self.y1).into()], s)
    }
}

impl<DB: DrawingBackend> ToElement<plotters::element::Rectangle<(f64, f64)>, DB> for kurbo::Rect {
    fn to_element<S: Into<ShapeStyle>>(self, s: S) -> plotters::element::Rectangle<(f64, f64)> {
        plotters::element::Rectangle::new([(self.x0, self.y0), (self.x1, self.y1)], s)
    }
}

impl BarChart {
    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            spacing: None,
            categories: Vec::default(),
            lines: None,
            axis: None,
        }
    }
    fn lines(&self) -> bool {
        self.lines.to_owned().unwrap_or(true)
    }

    fn spacing(&self) -> f64 {
        self.spacing.to_owned().unwrap_or(5.0)
    }
}

pub enum BarSegment {
    Normal { cat: u64, num: u64 },
    End,
}

pub struct BarSegments {
    blocks: u64,
    cat_names: Vec<String>,
    spacing: u64,
}

impl BarSegments {
    fn new<I: Iterator<Item = S>, S: Into<String>>(blocks: u64, spacing: u64, iter: I) -> Self {
        let cat_names = iter.map(|s| s.into()).collect();
        Self {
            blocks,
            cat_names,
            spacing,
        }
    }
    fn cats(&self) -> u64 {
        self.cat_names.len() as u64
    }
}
impl ValueFormatter<BarSegment> for BarSegments {
    fn format(value: &BarSegment) -> String {
        match value {
            BarSegment::Normal { cat, .. } => cat.to_string(),
            BarSegment::End => "".to_owned(),
        }
    }

    fn format_ext(&self, value: &BarSegment) -> String {
        match value {
            BarSegment::Normal { cat, .. } => self.cat_names[*cat as usize].clone(),
            BarSegment::End => "".to_owned(),
        }
    }
}

impl Ranged for BarSegments {
    type FormatOption = NoDefaultFormatting;

    type ValueType = BarSegment;

    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        match value {
            BarSegment::Normal { cat, num, .. } => {
                let range = (limit.1 - limit.0) as f64;
                let spacing = self.spacing as f64;
                let blocks = (self.cats() * self.blocks) as f64;
                let block_w = (range - (spacing * 2.0 * self.cats() as f64)) / blocks;
                let block_gap = block_w * self.blocks as f64 + spacing * 2.0;
                let x = spacing + *cat as f64 * block_gap + block_w * (*num as f64);
                limit.0 + x as i32
            }
            BarSegment::End => limit.1,
        }
    }

    fn key_points<Hint: plotters::coord::ranged1d::KeyPointHint>(
        &self,
        _hint: Hint,
    ) -> Vec<Self::ValueType> {
        self.cat_names
            .iter()
            .enumerate()
            .map(|(cn, _c)| BarSegment::Normal {
                cat: cn as u64,
                num: (self.blocks as f64 / 2.0).round() as u64,
            })
            .collect()
    }

    fn range(&self) -> std::ops::Range<Self::ValueType> {
        Range {
            start: BarSegment::Normal { cat: 0, num: 0 },
            end: BarSegment::End,
        }
    }
}

impl ChartType for BarChart {
    type DataPoint = BarPoint;
    type X = BarSegments;
    type Y = plotters::coord::types::RangedCoordu64;

    fn render_datasets<'a, 'b, DB: DrawingBackend>(
        &self,
        info: &ChartInfo<f64>,
        c: &mut ChartBuilder<'a, 'b, DB>,
    ) -> Result<ChartContext<'a, DB, Cartesian2d<Self::X, Self::Y>>> {
        let fiinfo = info.font();
        let tfont: TextStyle = fiinfo.into_text_style();
        let max_val = max_val(&info.datasets);
        let nb_blocks = info.datasets.len();
        let mut chart = c
            .set_left_and_bottom_label_area_size(50)
            .margin(10)
            .caption(info.caption(), FontFamily::SansSerif)
            .build_cartesian_2d(
                BarSegments::new(
                    nb_blocks as u64,
                    self.spacing() as u64,
                    self.categories.iter(),
                ),
                0u64..(max_val) as u64,
            )?;
        let mut mesh = chart.configure_mesh();
        mesh.disable_x_mesh()
            .bold_line_style(&WHITE.mix(0.3))
            .y_desc(self.axis.to_owned().unwrap_or("".to_owned()))
            .y_label_style(tfont.clone());
        if !self.lines() {
            mesh.disable_y_mesh();
        }
        mesh.draw()?;
        for (nset, dset) in info.datasets.iter().enumerate() {
            let colour = dset.extra.colour;
            chart
                .draw_series((0..self.categories.len()).map(|ncat| {
                    Rectangle::new(
                        [
                            (
                                BarSegment::Normal {
                                    cat: ncat as u64,
                                    num: nset as u64,
                                },
                                0u64,
                            ),
                            (
                                BarSegment::Normal {
                                    cat: ncat as u64,
                                    num: nset as u64 + 1,
                                },
                                dset.values[ncat] as u64,
                            ),
                        ],
                        colour.filled(),
                    )
                }))?
                .label(dset.extra.name.clone())
                .legend(move |pt| legend_for(pt, colour));
        }
        Ok(chart)
    }
}

fn max_val(datasets: &Vec<Dataset<f64>>) -> f64 {
    datasets
        .iter()
        .flat_map(|dset| dset.values.iter().map(|v| v.ceil() as u64))
        .max()
        .unwrap() as f64
}

#[cfg(test)]
mod tests {

    
}
