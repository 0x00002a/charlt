use std::f64::consts::PI;
use std::ops::Range;

use kurbo::{Affine, Point, Rect};
use plotters::coord::ranged1d::{DefaultValueFormatOption, NoDefaultFormatting, ValueFormatter};
use plotters::prelude::{Ranged, SegmentValue};
use plotters::style::{FontFamily, TextStyle};
use plotters::{element::Drawable, prelude::IntoSegmentedCoord};
use plotters::{
    prelude::{
        BitMapBackend, ChartBuilder, DrawingBackend, IntoDrawingArea, LabelAreaPosition,
        PathElement, Rectangle,
    },
    style::{Color, IntoFont, RGBColor, ShapeStyle, WHITE},
};
use serde::Deserialize;

use super::{decide_steps, Result, StepLabel, XY};
use crate::{
    chart::{ChartInfo, ChartType, Dataset, DatasetMeta},
    render::{self, Colour, FontInfo},
};

pub type BarPoint = f64;
#[derive(Clone, Debug, Deserialize)]
pub struct BarChart {
    /// Spacing between block groups
    spacing: Option<f64>,
    /// Categories of blocks, appear along the x axis
    categories: Vec<String>,
    /// Step size for the y axis
    step: u32,
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
            step: 10,
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

    fn series<DB: DrawingBackend>(
        &self,
        datasets: &Vec<super::Dataset<f64>>,
        info: &DrawingInfo,
    ) -> Result<Vec<Rectangle<(f64, f64)>>> {
        let blocks = datasets
            .iter()
            .enumerate()
            .flat_map(|(set_num, dset)| {
                dset.values.iter().enumerate().map(move |(i, v)| {
                    ToElement::<Rectangle<(f64, f64)>, DB>::to_element(
                        info.block_rect(set_num, i, v.to_owned()),
                        dset.extra.colour,
                    )
                })
            })
            .collect();
        Ok(blocks)
    }
}

enum BarSegment {
    Normal { cat: u64, num: u64 },
    End,
}

struct BarSegments {
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
            BarSegment::Normal { cat, num } => self.cat_names[*cat as usize].clone(),
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
                let block_w = (range - (spacing * self.cats() as f64)) / blocks;
                let block_gap = block_w * self.blocks as f64 + spacing;
                let x = *cat as f64 * block_gap + block_w * (*num as f64);
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
            .map(|(cn, c)| BarSegment::Normal {
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

    const NAME: &'static str = "bar";

    fn render_datasets<DB: DrawingBackend>(
        &self,
        info: &ChartInfo<f64>,
        area: &kurbo::Rect,
        c: &mut ChartBuilder<DB>,
    ) -> Result<()> {
        let dinfo = DrawingInfo::new(&info.datasets, area.to_owned(), self.spacing())?;
        let fiinfo = info.font();
        let tfont: TextStyle = fiinfo.into_text_style();
        let mut chart = c
            .set_left_and_bottom_label_area_size(40)
            .margin(10)
            .build_cartesian_2d(
                BarSegments::new(
                    dinfo.nb_blocks as u64,
                    self.spacing() as u64,
                    self.categories.iter(),
                ),
                0u64..(dinfo.max_val + 1000.0) as u64,
            )?;
        chart
            .configure_mesh()
            .disable_x_mesh()
            .bold_line_style(&WHITE.mix(0.3))
            .y_desc(self.axis.to_owned().unwrap_or("".to_owned()))
            .y_label_style(tfont.clone())
            .draw()?;
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
                .label(dset.extra.name.clone());
        }

        chart
            .configure_series_labels()
            .position(plotters::prelude::SeriesLabelPosition::UpperRight)
            .label_font(tfont.clone())
            .draw()?;
        Ok(())
    }
}

struct DrawingInfo {
    block_w: f64,
    nb_cats: f64,
    max_val: f64,
    spacing: f64,
    area: Rect,
    nb_blocks: f64,
}
fn max_val(datasets: &Vec<Dataset<f64>>) -> f64 {
    datasets
        .iter()
        .flat_map(|dset| dset.values.iter().map(|v| v.ceil() as u64))
        .max()
        .unwrap() as f64
}
impl DrawingInfo {
    fn block_gap(&self) -> f64 {
        self.block_w * self.nb_blocks + self.spacing * 2.0
    }
    fn block_h(&self, v: f64) -> f64 {
        (self.area.height() / self.max_val) * v
    }
    fn new(datasets: &Vec<Dataset<f64>>, area: Rect, spacing: f64) -> Result<Self> {
        let nb_cats = datasets
            .iter()
            .map(|dset| dset.values.len())
            .fold(Ok(0), |r, l| {
                r.and_then(|rl| {
                    if rl != 0 && rl != l {
                        Err(render::Error::InvalidDatasets(
                            "datasets must all be the same size".to_owned(),
                        ))
                    } else {
                        Ok(l)
                    }
                })
            })? as f64;
        let nb_blocks = datasets.len() as f64;
        let free_width = area.width() - (nb_cats) * spacing * 2.0;
        if free_width < nb_blocks * nb_cats {
            return Err(render::Error::NotEnoughSpace(
                nb_blocks,
                free_width,
                "free width for blocks".to_owned(),
            ));
        }
        let max_val = max_val(datasets);
        let block_w = free_width / (nb_blocks * nb_cats);
        Ok(Self {
            block_w,
            nb_cats,
            max_val,
            spacing,
            area,
            nb_blocks,
        })
    }
    fn block_rect(&self, dataset: usize, num: usize, v: f64) -> Rect {
        let start_x = num as f64 * self.block_gap()
            + dataset as f64 * self.block_w
            + self.area.min_x()
            + self.spacing;
        let start_y = self.area.min_y();

        let r = Rect::new(
            start_x,
            start_y,
            start_x + self.block_w,
            start_y + self.block_h(v),
        );
        assert!(
            self.area.union(r).area().floor() <= self.area.area().floor(),
            "block ({:?}) inside draw area ({:?})",
            r,
            self.area
        );
        r
    }
    fn cat_xbounds(&self, cat: usize) -> (Point, Point) {
        let start_x = cat as f64 * self.block_gap();
        let end_x = start_x + self.nb_blocks * self.block_w;
        ((start_x, 0.0).into(), (end_x, 0.0).into())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
}
