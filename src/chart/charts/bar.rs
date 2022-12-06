use std::f64::consts::PI;

use kurbo::{Affine, Point, Rect};
use plotters::prelude::SegmentValue;
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
    render::{self, Colour, FontInfo, TextInfo},
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
    fn calc_labels(
        &self,
        font: &FontInfo,
        info: &DrawingInfo,
        margins: &XY<f64>,
    ) -> Result<Vec<TextInfo>> {
        if info.nb_cats != self.categories.len() as f64 {
            return Err(render::Error::InvalidDatasets(format!(
                "categories and number of blocks do not match {} != {}",
                self.categories.len(),
                info.nb_cats
            )));
        }
        let mut out = Vec::with_capacity(self.categories.len());
        for (group, cat) in self.categories.iter().enumerate() {
            let (xstart, xend) = info.cat_xbounds(group);
            out.push(
                TextInfo::new(cat.to_owned())
                    .font(font.to_owned())
                    .transform(Affine::translate((
                        xstart.midpoint(xend).x,
                        info.area.max_y() + margins.y,
                    ))),
            )
        }
        Ok(out)
    }
}

struct BarSeries {}

impl Iterator for BarSeries {
    type Item = Rectangle<(f32, f32)>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
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
        let mut chart = //c.build_cartesian_2d(0..70u64, 0u64..dinfo.max_val as u64)?;
            c.build_cartesian_2d(self.categories.into_segmented(), 0u64..dinfo.max_val as u64)?.set_secondary_coord((0..(dinfo.nb_cats * dinfo.nb_blocks).ceil() as u64), 0..dinfo.max_val as u64);
        /*chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(&WHITE.mix(0.3))
        .y_desc(self.axis.to_owned().unwrap_or("".to_owned()))
        .draw()?;*/
        /*chart.draw_series(info.datasets.iter().enumerate().flat_map(|(n_cat, dset)| {
                    let mut out = Vec::new();
                    for n in 0..dset.values.len() {
                        let bar = dinfo.block_rect(n_cat, n, dset.values[n]);
                        out.push(bar);
                    }
                    out.into_iter()
        }));*/
        for dset in &info.datasets {
            chart.draw_secondary_series(dset.values.iter().flat_map(|c| {
                let mut out = Vec::new();
                for ncat in 0..self.categories.len() {
                    let start_x = dinfo.block_gap() * ncat as f64;
                    let end_x = start_x + dinfo.block_w * dinfo.nb_blocks;
                    out.push(Rectangle::new(
                        [
                            (start_x.ceil() as u64, 0u64),
                            (end_x.ceil() as u64, *c as u64),
                        ],
                        dset.extra.colour,
                    ))
                    //Rectangle::new([(c.into(), 5u64), (c.into(), 1u64)], plotters::style::BLACK)
                }
                out.into_iter()
            }))?;
        }

        /*chart.draw_series(self.categories.iter().enumerate().flat_map(|(ncat, c)| {
            let mut rects = Vec::new();
            for n in 0..info.datasets.len() {
                rects.push(Rectangle::new(
                    [
                        (c.into(), 0u64, ncat as u64),
                        (
                            c.into(),
                            info.datasets[n].values[ncat] as u64,
                            (ncat + 1) as u64,
                        ),
                    ],
                    info.datasets[n].extra.colour,
                ))
            }
            //Rectangle::new([(c.into(), 5u64), (c.into(), 1u64)], plotters::style::BLACK)
            rects.into_iter()
        }))?;*/
        //chart.draw_series(self.series(&info.datasets, &dinfo)?.iter())?;
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
