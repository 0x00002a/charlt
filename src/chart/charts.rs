use std::f64::consts::PI;

use kurbo::{Affine, BezPath, Line, Point, Rect, Shape, Size, TranslateScale, Vec2};
use piet::{RenderContext, Text, TextAlignment, TextLayout, TextLayoutBuilder};
use rlua::{FromLua, Value};
use serde::Deserialize;

use crate::render::{self, colours, FontInfo, RenderContextExt, TextInfo};

use super::{Chart, ChartType, DataPoint};

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Charts {
    #[serde(rename = "xy-scatter")]
    XYScatter(Chart<XYScatter, XYPoint<f64>>),
    #[serde(rename = "bar")]
    Bar(Chart<BarChart, BarPoint>),
}

#[derive(Clone, Debug, Deserialize)]
pub struct XYPoint<T> {
    x: T,
    y: T,
}

impl<T> XYPoint<T> {
    pub fn new<T1: Into<T>, T2: Into<T>>(x: T1, y: T2) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }
}
impl From<XYPoint<f64>> for kurbo::Point {
    fn from(pt: XYPoint<f64>) -> Self {
        Self::new(pt.x, pt.y)
    }
}

type BarPoint = f64;
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct BarChart {}
impl ChartType for BarChart {
    type DataPoint = BarPoint;

    const NAME: &'static str = "bar";

    fn render_datasets<R: RenderContext>(
        &self,
        datasets: &Vec<super::DataPoint<Self::DataPoint>>,
        area: &kurbo::Rect,
        _: &FontInfo,
        _: &mut R,
    ) -> Result<(), crate::render::Error> {
        todo!()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct XYScatter {
    pub axis: XYPoint<String>,
    pub steps: XYPoint<u32>,
    pub grid: Option<XYPoint<bool>>,
}

fn mk_grids<R: RenderContext>(
    grid: &XYPoint<bool>,
    steps: &XYPoint<Vec<u64>>,
    bounds: &Rect,
    r: &mut R,
) {
    let mut do_iter = |steps: &Vec<u64>, f: &dyn Fn(f64) -> ((f64, f64), (f64, f64))| {
        let line_w = 1.0;
        for pt in steps {
            let (x, y) = f(pt.to_owned() as f64);
            let line = Line::new(x, y);
            println!("line: {} -> {}", line);
            let b = r.solid_brush(piet::Color::GRAY);
            r.stroke(line, &b, line_w);
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
                (bounds.min_x(), y + bounds.min_y()),
                (bounds.max_x(), y + bounds.min_y()),
            )
        });
    }
}
impl XYScatter {
    fn mk_labels<R: RenderContext>(
        &self,
        steps: &XYPoint<Vec<u64>>,
        xylines: &XYPoint<f64>,
        lbl_font: &FontInfo,
        origin: &Point,
        r: &mut R,
    ) -> Result<(f64, f64), render::Error> {
        let margin = 0.0;
        let mut build_texts = |steps: &Vec<u64>,
                               f: &dyn Fn(f64, &mut R) -> Result<Point, render::Error>|
         -> Result<Vec<Size>, render::Error> {
            steps
                .iter()
                .map(|coord| {
                    let content = coord.to_string();
                    let pt: Vec2 = f(coord.to_owned() as f64, r)?.to_vec2();
                    let s = r
                        .render_text(
                            pt.to_point(),
                            &TextInfo::new(content)
                                .font(lbl_font.to_owned())
                                .alignment(TextAlignment::Center),
                        )?
                        .size();
                    Ok(s)
                })
                .collect()
        };
        let x_offset = build_texts(&steps.x, &|x, _| {
            Ok(Point::new(x - margin + origin.x, xylines.y))
        })?
        .into_iter()
        .map(|s| s.height.ceil() as u64)
        .max()
        .unwrap();
        let y_offset = build_texts(&steps.y, &|y, r| {
            Ok(Point::new(
                xylines.x
                    - r.text_bounds(&TextInfo::new(y.to_string()).font(lbl_font.to_owned()))?
                        .width,
                xylines.y - y,
            ))
        })?
        .into_iter()
        .map(|s| s.width.ceil() as u64)
        .max()
        .unwrap();
        Ok((-(x_offset as f64) - 1.0, y_offset as f64 + 1.0))
    }
    fn render_into<R: RenderContext>(
        &self,
        datasets: &Vec<DataPoint<XYPoint<f64>>>,
        area: &kurbo::Rect,
        label_font: &FontInfo,
        r: &mut R,
    ) -> Result<(), render::Error> {
        let paths: Vec<_> = datasets
            .iter()
            .map(|point| {
                (
                    point.colour.to_owned(),
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

        let line_w = 3.0;

        let (step_x, step_y) = (self.steps.x as f64, self.steps.y as f64);
        let steps_y: Vec<_> = (0..(area.height() + step_y) as u64)
            .step_by(step_y as usize)
            .collect();
        let steps_x: Vec<_> = (0..(area.width() + step_x) as u64)
            .step_by(step_x as usize)
            .collect();

        let steps = XYPoint {
            x: steps_x,
            y: steps_y,
        };
        let xylines = XYPoint {
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

        let _ = r
            .render_text(
                (area.center().x, area.max_y() + y_offset).into(),
                &TextInfo::new(self.axis.y.to_owned()),
            )?
            .size()
            .width;
        r.save()?;
        let render_pt = area.center() + (x_offset - area.center().x, 0.0);
        r.transform(Affine::translate(render_pt.to_vec2()));
        r.transform(Affine::rotate(-PI / 2.0));
        r.transform(Affine::translate(render_pt.to_vec2() * -1.0));
        let _ = r
            .render_text(
                render_pt,
                &TextInfo::new(self.axis.x.to_owned()).font(label_font.to_owned()),
            )?
            .size()
            .height;
        r.restore()?;

        mk_grids(
            &self.grid.clone().unwrap_or(XYPoint { x: false, y: true }),
            &steps,
            area,
            r,
        );

        for (c, p) in paths {
            let b = r.solid_brush(c.into());
            r.stroke(p, &b, line_w);
        }
        Ok(())
    }
}

impl ChartType for XYScatter {
    type DataPoint = XYPoint<f64>;
    const NAME: &'static str = "xy-scatter";

    fn render_datasets<R: RenderContext>(
        &self,
        datasets: &Vec<DataPoint<XYPoint<f64>>>,
        area: &kurbo::Rect,
        label_font: &FontInfo,
        r: &mut R,
    ) -> Result<(), render::Error> {
        let fam = label_font.family.to_owned().to_family(r)?;
        let margin = 20.0;
        let char_dims = r
            .text()
            .new_text_layout("X")
            .font(fam, label_font.size)
            .build()?
            .size();
        let inner = Rect::new(
            area.x0 + char_dims.height + char_dims.width * 4.0 + margin,
            area.y0 + margin,
            area.x1 - margin,
            area.y1 - char_dims.height * 3.0 - margin,
        );
        self.render_into(datasets, &inner, label_font, r)
    }
}

#[cfg(test)]
mod tests {

    use crate::{chart::DataPoint, render::Colour};

    use super::*;
}
