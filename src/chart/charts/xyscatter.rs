use std::f64::consts::PI;

use crate::{
    chart::{ChartInfo, ChartType, Dataset, DatasetMeta},
    render::{self, FontInfo, RenderContextExt, TextInfo},
};

use super::{decide_steps, mk_grids, Result, StepLabel, XY};
use kurbo::{Affine, BezPath, Point, Rect, Shape, Size, TranslateScale};
use piet::{RenderContext, Text, TextAlignment, TextLayout, TextLayoutBuilder};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct XYScatter {
    axis: XY<String>,
    steps: XY<u32>,
    grid: Option<XY<bool>>,
    margin: Option<XY<f64>>,
}

impl XYScatter {
    fn margin(&self) -> XY<f64> {
        self.margin.to_owned().unwrap_or(XY { x: 8.0, y: 10.0 })
    }
    fn mk_labels<R: RenderContext>(
        &self,
        steps: &XY<Vec<StepLabel<f64>>>,
        xylines: &XY<f64>,
        lbl_font: &FontInfo,
        origin: &Point,
        r: &mut R,
    ) -> Result<(f64, f64)> {
        let margin = self.margin();
        let mut build_texts = |steps: &Vec<StepLabel<f64>>,
                               f: &dyn Fn(f64) -> TextInfo|
         -> Result<Vec<Size>> {
            steps
                .iter()
                .map(|coord| {
                    let content = coord.value.to_owned();
                    let info = f(coord.offset.to_owned() as f64);
                    let s = r
                        .render_text((0.0, 0.0), &info.content(content).font(lbl_font.to_owned()))?
                        .size();
                    Ok(s)
                })
                .collect()
        };
        let y_offset = build_texts(&steps.x, &|x| {
            TextInfo::default()
                .transform(Affine::translate((x + origin.x, xylines.y + margin.y)))
                .alignment(TextAlignment::Center)
        })?
        .into_iter()
        .map(|s| s.width.ceil() as u64)
        .max()
        .unwrap();
        let x_offset = build_texts(&steps.y, &|y| {
            TextInfo::default()
                .alignment(TextAlignment::End)
                .transform(Affine::translate((xylines.x - margin.x, xylines.y - y)))
        })?
        .into_iter()
        .map(|s| s.width.ceil() as u64)
        .max()
        .unwrap();
        Ok((x_offset as f64 + 10.0, y_offset as f64 + 4.0))
    }
    fn calc_paths(
        &self,
        datasets: &Vec<Dataset<XY<f64>>>,
        area: &Rect,
    ) -> Result<Vec<(DatasetMeta, BezPath)>> {
        let paths: Vec<_> = datasets
            .iter()
            .map(|point| {
                (
                    point.extra.to_owned(),
                    point.values.iter().fold(BezPath::new(), |mut path, pt| {
                        if path.elements().len() == 0 {
                            path.move_to(pt.clone())
                        } else {
                            path.line_to(pt.clone());
                        }
                        path
                    }),
                )
            })
            .collect();
        let bounds = paths
            .iter()
            .map(|(_, b)| b.bounding_box())
            .reduce(|b, r| b.union(r))
            .ok_or(render::Error::EmptyDataset)?;
        let scale_x = area.width() / bounds.max_x();
        let scale_y = area.height() / bounds.max_y();

        let paths: Vec<_> = paths
            .into_iter()
            .map(|(c, mut p)| {
                p.apply_affine(Affine::FLIP_Y);
                p.apply_affine(Affine::scale_non_uniform(scale_x, scale_y));
                (
                    c,
                    TranslateScale::translate(area.center() - p.bounding_box().center()) * p,
                )
            })
            .collect();
        Ok(paths)
    }
    fn steps(&self, datasets: &Vec<Dataset<XY<f64>>>, area: &Rect) -> XY<Vec<StepLabel>> {
        let (min_x, min_y, max_x, max_y) = datasets
            .iter()
            .map(|s| {
                let max_x = s.values.iter().map(|v| v.x as u64).max().unwrap();
                let max_y = s.values.iter().map(|v| v.y as u64).max().unwrap();
                let min_x = s.values.iter().map(|v| v.x as u64).min().unwrap();
                let min_y = s.values.iter().map(|v| v.y as u64).min().unwrap();
                (min_x, min_y, max_x, max_y)
            })
            .reduce(|(lmix, lmiy, lmx, lmy), (rmix, rmiy, rmx, rmy)| {
                (lmix.min(rmix), lmiy.min(rmiy), lmx.max(rmx), lmy.max(rmy))
            })
            .unwrap();
        let steps_x = decide_steps(area.width(), min_x as f64, max_x as f64, self.steps.x);
        let steps_y = decide_steps(area.height(), min_y as f64, max_y as f64, self.steps.y);
        XY::new(steps_x, steps_y)
    }
    fn render_into<R: RenderContext>(
        &self,
        datasets: &Vec<Dataset<XY<f64>>>,
        area: &kurbo::Rect,
        label_font: &FontInfo,
        r: &mut R,
    ) -> Result<()> {
        let steps = self.steps(datasets, area);
        let xylines = XY {
            x: area.min_x(),
            y: area.max_y(),
        };
        let (x_offset, y_offset) = self.mk_labels(
            &steps,
            &xylines,
            &label_font,
            &Point::new(area.min_x(), area.min_y()),
            r,
        )?;

        r.render_text(
            (area.center().x, area.max_y() + y_offset + self.margin().y),
            &TextInfo::new(self.axis.x.to_owned()).alignment(TextAlignment::Center),
        )?;
        r.render_text(
            (xylines.x - (x_offset + self.margin().x), area.center().y),
            &TextInfo::new(self.axis.y.to_owned())
                .font(label_font.to_owned())
                .transform(Affine::translate((0.0, 0.0)))
                .transform(Affine::rotate(-PI / 2.0))
                .alignment(TextAlignment::Center),
        )?;

        let grid_steps = steps.map(|s| s.iter().map(|s| s.offset.ceil() as u64).collect());
        for line in mk_grids(
            &self.grid.clone().unwrap_or(XY { x: false, y: true }),
            &grid_steps,
            area,
        ) {
            let b = r.solid_brush(piet::Color::GRAY);
            r.stroke(line, &b, 1.0);
        }

        let axis = mk_grids(
            &XY::new(true, true),
            &XY::new(vec![grid_steps.x[0]], vec![grid_steps.y[0].to_owned()]),
            area,
        )
        .into_iter()
        .fold(BezPath::default(), |mut p, l| {
            if p.elements().len() == 0 {
                p.move_to(l.p0);
                p.line_to(l.p0);
            } else {
                p.line_to(l.p0);
                p.line_to(l.p1);
            };
            p
        });
        let b = r.solid_brush(piet::Color::BLACK);
        r.stroke(axis, &b, 2.0);

        for (c, p) in self.calc_paths(datasets, area)? {
            let b = r.solid_brush(c.colour.into());
            r.stroke(p, &b, c.thickness);
        }
        Ok(())
    }
}

impl ChartType for XYScatter {
    type DataPoint = XY<f64>;
    const NAME: &'static str = "xy-scatter";

    fn render_datasets<R: RenderContext>(
        &self,
        info: &ChartInfo<Self::DataPoint>,
        area: &kurbo::Rect,
        r: &mut R,
    ) -> Result<()> {
        let label_font = &info.font();
        let datasets = &info.datasets;
        let fam = label_font.family.to_owned().to_family(r)?;
        let margin = Into::<Point>::into(self.margin()) + (20.0, 20.0);
        let char_dims = r
            .text()
            .new_text_layout("X")
            .font(fam, label_font.size)
            .build()?
            .size();

        let inner = Rect::new(
            area.x0 + char_dims.height + char_dims.width * 4.0 + margin.x,
            area.y0 + margin.y,
            area.x1 - margin.x,
            area.y1 - char_dims.height * 3.0 - margin.y,
        );
        let step_max: XY<f64> = self.steps(datasets, &inner).map(|sl| {
            sl.iter()
                .map(Clone::clone)
                .map(Into::<u64>::into)
                .max()
                .unwrap() as f64
        });
        self.render_into(
            datasets,
            &Rect::new(inner.x0, inner.y0, step_max.x, step_max.y),
            label_font,
            r,
        )
    }
}

#[cfg(test)]
mod tests {
    use kurbo::{Line, Shape};

    use crate::chart::charts::to_dataset;

    use super::*;

    #[test]
    fn grids_with_uneven_offset() {
        let steps = XY::new(vec![0, 100, 200], vec![0, 10, 20]);
        let areas = vec![
            Rect::new(0.0, 0.0, 200.0, 20.0),
            Rect::new(30.0, 22.0, 250.0, 47.0),
        ];
        for area in areas {
            let grid_draw = XY::new(false, true);
            let grid = mk_grids(&grid_draw, &steps, &area);
            for line in &grid {
                assert_eq!(line.p0.y, line.p1.y);
            }
            let botline = Line::new((area.min_x(), area.max_y()), (area.max_x(), area.max_y()));
            assert!(
                grid.contains(&botline),
                "grid doesn't have bottom line {:?}: {:?}",
                botline,
                grid
            );
        }
    }
    #[test]
    fn paths_with_offset() {
        let datasets = to_dataset(&vec![vec![
            XY::new(0.0, 0.0),
            XY::new(10.0, 500.0),
            XY::new(20.0, 551.0),
        ]]);
        let chart = XYScatter {
            axis: XY::new("x", "y"),
            steps: XY::new(10 as u32, 10 as u32),
            grid: None,
            margin: None,
        };
        let areas = vec![
            Rect::new(0.0, 0.0, 500.0, 500.0),
            Rect::new(50.0, 88.0, 400.0, 300.0),
        ];
        for area in areas {
            let paths = chart.calc_paths(&datasets, &area).unwrap();
            for (_, bez) in paths {
                assert!(bez.bounding_box().area() <= area.area());
            }
        }
    }
}
