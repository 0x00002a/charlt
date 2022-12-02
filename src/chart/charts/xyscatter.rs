use std::f64::consts::PI;

use crate::{
    chart::{ChartInfo, ChartType, Dataset, DatasetMeta},
    render::{self, FontInfo, RenderContextExt, TextInfo},
    utils::RoundMul,
};

use super::{mk_grids, Result, XY};
use kurbo::{Affine, BezPath, Line, Point, Rect, Shape, Size, TranslateScale};
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
        self.margin.to_owned().unwrap_or(XY { x: 4.0, y: 10.0 })
    }
    fn mk_labels<R: RenderContext>(
        &self,
        steps: &XY<Vec<u64>>,
        xylines: &XY<f64>,
        lbl_font: &FontInfo,
        origin: &Point,
        r: &mut R,
    ) -> Result<(f64, f64)> {
        let margin = self.margin();
        let mut build_texts = |steps: &Vec<u64>,
                               f: &dyn Fn(f64) -> TextInfo|
         -> Result<Vec<Size>> {
            steps
                .iter()
                .map(|coord| {
                    let content = coord.to_string();
                    let info = f(coord.to_owned() as f64);
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
        Ok((x_offset as f64, y_offset as f64))
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
    fn calc_steps(&self, area: &Rect) -> XY<Vec<u64>> {
        let (step_x, step_y) = (self.steps.x as f64, self.steps.y as f64);
        let steps_y: Vec<_> = (0..(area.height() + step_y) as u64)
            .step_by(step_y as usize)
            .collect();
        let steps_x: Vec<_> = (0..(area.width() + step_x) as u64)
            .step_by(step_x as usize)
            .collect();
        XY::new(steps_x, steps_y)
    }
    fn render_into<R: RenderContext>(
        &self,
        datasets: &Vec<Dataset<XY<f64>>>,
        area: &kurbo::Rect,
        label_font: &FontInfo,
        r: &mut R,
    ) -> Result<()> {
        let steps = self.calc_steps(area);
        let steps_x = steps.x.clone();
        let steps_y = steps.y.clone();

        let steps = XY {
            x: steps_x,
            y: steps_y,
        };
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
            (area.center().x, area.max_y() + y_offset),
            &TextInfo::new(self.axis.y.to_owned()).alignment(TextAlignment::Center),
        )?;
        r.render_text(
            (xylines.x, area.center().y),
            &TextInfo::new(self.axis.x.to_owned())
                .font(label_font.to_owned())
                .transform(Affine::translate((-(self.margin().x + x_offset), 0.0)))
                .transform(Affine::rotate(-PI / 2.0))
                .alignment(TextAlignment::Center),
        )?;

        for line in mk_grids(
            &self.grid.clone().unwrap_or(XY { x: false, y: true }),
            &steps,
            area,
        ) {
            let b = r.solid_brush(piet::Color::GRAY);
            r.stroke(line, &b, 1.0);
        }

        let axis = mk_grids(
            &XY::new(true, true),
            &XY::new(vec![steps.x[0]], vec![steps.y[0].to_owned()]),
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
    fn step_adjust(&self, area: &Rect) -> Rect {
        step_adjust(area, &self.steps)
    }
}
fn step_adjust(area: &Rect, steps: &XY<u32>) -> Rect {
    Rect::new(
        area.min_x(),
        area.max_y() - area.height().ceil_mul(steps.y as f64),
        area.min_x() + area.width().ceil_mul(steps.x as f64),
        area.max_y(),
    )
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
        self.render_into(datasets, &self.step_adjust(&inner), label_font, r)
    }
}

#[cfg(test)]
mod tests {
    use kurbo::Shape;

    use crate::chart::charts::to_dataset;

    use super::*;

    #[test]
    fn test_step_adjust() {
        let steps = XY::new(5 as u32, 5 as u32);
        let area = Rect::new(0.0, 0.0, 9.0, 9.0);
        let adjusted = step_adjust(&area, &steps);
        assert_eq!(adjusted.width(), 10.0);
        assert_eq!(adjusted.height(), 10.0);
    }
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
