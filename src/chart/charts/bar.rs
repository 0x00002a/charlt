use kurbo::{Affine, Line, Point, Rect, Shape};
use piet::{RenderContext, Text, TextAlignment, TextLayout, TextLayoutBuilder};
use serde::Deserialize;

use super::{decide_steps, mk_grids, step_adjust, Result, StepLabel, XY};
use crate::{
    chart::{ChartInfo, ChartType, Dataset, DatasetMeta},
    render::{self, Colour, FontInfo, RenderContextExt, TextInfo},
};

pub type BarPoint = f64;
#[derive(Clone, Debug, Deserialize)]
pub struct BarChart {
    spacing: Option<f64>,
    categories: Vec<String>,
    step: u32,
}
struct DrawingInfo {
    block_w: f64,
    inner_len: f64,
    max_val: f64,
    spacing: f64,
    area: Rect,
    nb_blocks: f64,
}
impl DrawingInfo {
    fn block_gap(&self) -> f64 {
        self.block_w * self.inner_len + self.spacing
    }
    fn block_h(&self, v: f64) -> f64 {
        (self.area.height() / self.max_val) * v
    }
    fn new(datasets: &Vec<Dataset<f64>>, area: Rect, spacing: f64) -> Result<Self> {
        let inner_len = datasets
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
        let nb_blocks = datasets.len() as f64 * inner_len;
        let free_width = area.width() - (datasets.len() as f64 - 1.0) * spacing;
        if free_width < nb_blocks {
            return Err(render::Error::NotEnoughSpace(
                nb_blocks,
                free_width,
                "free width for blocks".to_owned(),
            ));
        }
        let block_w = free_width / nb_blocks;
        let max_val = datasets
            .iter()
            .flat_map(|dset| dset.values.iter().map(|v| v.ceil() as u64))
            .max()
            .unwrap() as f64;
        Ok(Self {
            block_w,
            inner_len,
            max_val,
            spacing,
            area,
            nb_blocks,
        })
    }
    fn block_rect(&self, dataset: usize, num: usize, v: f64) -> Rect {
        let start_x =
            num as f64 * self.block_gap() + dataset as f64 * self.block_w + self.area.min_x();
        let start_y = self.area.min_y();
        Rect::new(
            start_x,
            start_y,
            start_x + self.block_w,
            start_y + self.block_h(v),
        )
    }
    fn cat_xbounds(&self, cat: usize) -> (Point, Point) {
        let start_x = cat as f64 * self.block_gap();
        let end_x = start_x + self.inner_len * self.block_w;
        ((start_x, 0.0).into(), (end_x, 0.0).into())
    }
}

impl BarChart {
    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            spacing: None,
            categories: Vec::default(),
            step: 10,
        }
    }

    fn spacing(&self) -> f64 {
        self.spacing.to_owned().unwrap_or(10.0)
    }
    fn calc_blocks(
        &self,
        datasets: &Vec<super::Dataset<f64>>,
        info: &DrawingInfo,
    ) -> Result<Vec<(DatasetMeta, Vec<Rect>)>> {
        let blocks = datasets
            .iter()
            .enumerate()
            .map(|(set_num, dset)| {
                (
                    dset.extra.clone(),
                    dset.values
                        .iter()
                        .enumerate()
                        .map(|(i, v)| info.block_rect(set_num, i, v.to_owned()))
                        .collect::<Vec<_>>(),
                )
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
        if info.inner_len != self.categories.len() as f64 {
            return Err(render::Error::InvalidDatasets(format!(
                "categories and number of blocks do not match {} != {}",
                self.categories.len(),
                info.nb_blocks
            )));
        }
        let mut out = Vec::with_capacity(self.categories.len());
        for (group, cat) in self.categories.iter().enumerate() {
            let (xstart, xend) = info.cat_xbounds(group);
            out.push(
                TextInfo::new(cat.to_owned())
                    .font(font.to_owned())
                    .alignment(TextAlignment::Center)
                    .transform(Affine::translate((
                        xstart.midpoint(xend).x,
                        info.area.max_y() + margins.y,
                    ))),
            )
        }
        let steps: Vec<_> = decide_steps(info.area.height(), 0.0, info.max_val, self.step as u64)
            .into_iter()
            .map(|lbl| StepLabel::new(lbl.value, Point::new(0.0, lbl.offset)))
            .collect();
        for lbl in steps {
            out.push(
                TextInfo::new(lbl.value.to_string())
                    .alignment(TextAlignment::End)
                    .font(font.to_owned())
                    .transform(Affine::translate((
                        lbl.offset.x - margins.x,
                        info.area.max_y() - lbl.offset.y,
                    ))),
            )
        }
        Ok(out)
    }
}
impl ChartType for BarChart {
    type DataPoint = BarPoint;

    const NAME: &'static str = "bar";

    fn render_datasets<R: RenderContext>(
        &self,
        info: &ChartInfo<f64>,
        area: &kurbo::Rect,
        r: &mut R,
    ) -> Result<()> {
        let datasets = &info.datasets;
        let label_font = &info.font();
        let fam = label_font.family.to_owned().to_family(r)?;
        let margin = Into::<Point>::into(info.margins()) + (20.0, 20.0);
        let char_dims = r
            .text()
            .new_text_layout("X")
            .font(fam, label_font.size)
            .build()?
            .size();

        let inner = Rect::new(
            (area.x0 + char_dims.height + char_dims.width * 4.0 + margin.x).floor(),
            (area.y0 + margin.y).floor(),
            (area.x1 - margin.x).floor(),
            (area.y1 - char_dims.height * 3.0 - margin.y).floor(),
        );
        let area = step_adjust(&inner, &XY::new(1 as u32, self.step));
        let draw_info = DrawingInfo::new(datasets, area.clone(), self.spacing())?;
        r.with_restore(|r| -> Result<()> {
            let to_mid = Affine::translate(area.center().to_vec2());
            r.transform(to_mid * Affine::FLIP_Y * to_mid.inverse());

            if datasets.len() == 0 {
                return Ok(());
            }
            for (c, blocks) in self.calc_blocks(datasets, &draw_info)? {
                let b = r.solid_brush(c.colour.into());
                for block in blocks {
                    r.fill(block, &b);
                }
            }

            Ok(())
        })??;

        for line in mk_grids(&XY::new(true, true), &XY::new(vec![0], vec![0]), &area) {
            let b = r.solid_brush(Colour::BLACK);
            r.stroke(line, &b, 2.0);
        }

        for txt in self.calc_labels(&info.font(), &draw_info, &info.margins())? {
            r.render_text((area.min_x(), 0.0), &txt)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::chart::charts::to_dataset;

    use super::*;

    #[test]
    fn test_cat_bounds() {
        let area = Rect::new(0.0, 0.0, 12.0, 12.0);
        let datasets = to_dataset(&vec![vec![0.0, 10.0], vec![2.0, 5.0]]);
        let spacing = 2.0;
        let info = DrawingInfo::new(&datasets, area.clone(), spacing).unwrap();
        assert_eq!(
            info.cat_xbounds(0),
            ((0.0, 0.0).into(), (info.block_w * 2.0, 0.0).into())
        );
        assert_eq!(
            info.cat_xbounds(1),
            (
                (info.block_gap(), 0.0).into(),
                (info.block_w * 2.0 + info.block_gap(), 0.0).into()
            )
        );
    }
    #[test]
    fn test_bar_allocation() {
        let datasets = to_dataset(&vec![vec![0.0, 10.0], vec![2.0, 5.0]]);
        let mut chart = BarChart::new();
        let area = Rect::new(0.0, 0.0, 12.0, 12.0);
        let spacing = 2.0;
        chart.spacing = Some(spacing);
        let block_w = (area.width() - spacing) / 4.0 as f64;
        let info = DrawingInfo::new(&datasets, area.clone(), spacing).unwrap();
        let block_h = |v| area.height() / 10.0 * v;
        let expected = vec![
            vec![
                Rect::new(0.0, 0.0, block_w, block_h(0.0)),
                Rect::new(
                    block_w * 2.0 + spacing,
                    0.0,
                    block_w * 3.0 + spacing,
                    block_h(10.0),
                ),
            ],
            vec![
                Rect::new(block_w, 0.0, block_w * 2.0, block_h(2.0)),
                Rect::new(
                    block_w * 3.0 + spacing,
                    0.0,
                    block_w * 4.0 + spacing,
                    block_h(5.0),
                ),
            ],
        ];
        for i in 0..expected.len() {
            for n in 0..expected[i].len() {
                let rect = info.block_rect(i, n, datasets[i].values[n]);
                assert_eq!(rect, expected[i][n], "rect: [{}][{}]", i, n);
            }
        }
    }
}
