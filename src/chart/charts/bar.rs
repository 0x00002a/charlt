use kurbo::{Affine, Rect};
use piet::RenderContext;
use serde::Deserialize;

use super::Result;
use crate::{
    chart::{ChartType, DatasetMeta},
    render::{self, FontInfo, RenderContextExt},
};

pub type BarPoint = f64;
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct BarChart {
    spacing: Option<f64>,
}

impl BarChart {
    #[allow(unused)]
    pub fn new() -> Self {
        Self { spacing: None }
    }

    fn spacing(&self) -> f64 {
        self.spacing.to_owned().unwrap_or(10.0)
    }
    fn calc_drawing_info(&self, _datasets: &Vec<super::Dataset<f64>>, _area: &Rect) {}
    fn calc_blocks(
        &self,
        datasets: &Vec<super::Dataset<f64>>,
        area: &Rect,
    ) -> Result<Vec<(DatasetMeta, Vec<Rect>)>> {
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
        let free_width = area.width() - (datasets.len() as f64 - 1.0) * self.spacing();
        if free_width < nb_blocks {
            return Err(render::Error::NotEnoughSpace(nb_blocks, free_width));
        }
        let block_w = free_width / nb_blocks;
        let max_val = datasets
            .iter()
            .flat_map(|dset| dset.values.iter().map(|v| v.ceil() as u64))
            .max()
            .unwrap() as f64;
        let block_h = |v| (area.height() / max_val) * v;
        let block_gap = block_w * inner_len + self.spacing();
        let blocks = datasets
            .iter()
            .enumerate()
            .map(|(set_num, dset)| {
                (
                    dset.extra.clone(),
                    dset.values
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let start_x = i as f64 * block_gap + set_num as f64 * block_w;
                            let start_y = area.min_y();
                            Rect::new(
                                start_x,
                                start_y,
                                start_x + block_w,
                                start_y + block_h(v.to_owned()),
                            )
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        Ok(blocks)
    }
}
impl ChartType for BarChart {
    type DataPoint = BarPoint;

    const NAME: &'static str = "bar";

    fn render_datasets<R: RenderContext>(
        &self,
        datasets: &Vec<super::Dataset<Self::DataPoint>>,
        area: &kurbo::Rect,
        _lbl_font: &FontInfo,
        r: &mut R,
    ) -> Result<()> {
        r.with_restore(|r| {
            let to_mid = Affine::translate(area.center().to_vec2());
            r.transform(to_mid * Affine::FLIP_Y * to_mid.inverse());

            if datasets.len() == 0 {
                return Ok(());
            }
            for (c, blocks) in self.calc_blocks(datasets, area)? {
                let b = r.solid_brush(c.colour.into());
                for block in blocks {
                    r.fill(block, &b);
                }
            }
            Ok(())
        })?
    }
}

#[cfg(test)]
mod tests {
    use crate::chart::charts::to_dataset;

    use super::*;

    #[test]
    fn test_bar_allocation() {
        let datasets = to_dataset(&vec![vec![0.0, 10.0], vec![2.0, 5.0]]);
        let mut chart = BarChart::new();
        let area = Rect::new(0.0, 0.0, 12.0, 12.0);
        let spacing = 2.0;
        chart.spacing = Some(spacing);
        let block_w = (area.width() - spacing) / 4.0 as f64;
        let blocks = chart.calc_blocks(&datasets, &area).unwrap();
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
        let rects = blocks.into_iter().map(|(_, rs)| rs).collect::<Vec<_>>();
        for i in 0..expected.len() {
            for n in 0..expected[i].len() {
                assert_eq!(rects[i][n], expected[i][n], "rect: [{}][{}]", i, n);
            }
        }
    }
}
