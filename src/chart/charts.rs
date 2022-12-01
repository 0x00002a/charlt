use kurbo::{Affine, BezPath, Line, Point, Rect, Shape, TranslateScale};
use piet::{RenderContext, Text, TextLayout, TextLayoutBuilder};
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
    bounds: &XYPoint<(f64, f64)>,
    r: &mut R,
) {
    fn do_iter<R: RenderContext, F: Fn(f64) -> ((f64, f64), (f64, f64))>(
        steps: &Vec<u64>,
        f: F,
        r: &mut R,
    ) {
        let line_w = 2.0;
        for pt in steps {
            let (x, y) = f(pt.clone() as f64);
            let line = Line::new(x, y);
            let b = r.solid_brush(piet::Color::GRAY);
            r.stroke(line, &b, line_w);
        }
    }
    if grid.x {
        do_iter(&steps.x, |x| ((x, bounds.y.0), (x, bounds.y.1)), r);
    }
    if grid.y {
        do_iter(&steps.y, |y| ((bounds.x.0, y), (bounds.x.1, y)), r);
    }
}
impl XYScatter {
    fn mk_labels<R: RenderContext>(
        &self,
        steps: &XYPoint<Vec<u64>>,
        xylines: &XYPoint<f64>,
        lbl_font: &FontInfo,
        r: &mut R,
    ) -> Result<(f64, f64), render::Error> {
        type TVal<R> = (Point, <R as RenderContext>::TextLayout);
        let margin = 5.0;
        let mut build_texts =
            |steps: &Vec<u64>, f: &dyn Fn(f64) -> Point| -> Result<Vec<TVal<R>>, render::Error> {
                steps
                    .iter()
                    .map(|coord| {
                        let content = coord.to_string();
                        let pt: Point = f(coord.to_owned() as f64);
                        Ok((
                            pt,
                            r.text()
                                .new_text_layout(content)
                                .font(lbl_font.to_owned().family.to_family(r)?, lbl_font.size)
                                .build()?,
                        ))
                    })
                    .collect()
            };
        let texts_x = build_texts(&steps.x, &|x| Point::new(x - margin, xylines.y))?;
        let texts_y = build_texts(&steps.x, &|y| Point::new(xylines.x, xylines.y - y + margin))?;
        let y_offset = texts_y
            .iter()
            .map(|(_, t)| t.size().height.ceil() as u64)
            .max()
            .unwrap();
        let x_offset = texts_x
            .iter()
            .map(|(_, t)| t.size().width.ceil() as u64)
            .max()
            .unwrap();
        for (pt, txt) in texts_x.into_iter().chain(texts_y.into_iter()) {
            r.draw_text(&txt, pt);
        }
        Ok((-(x_offset as f64) - 1.0, y_offset as f64 + 1.0))
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
                    TranslateScale::translate((0.0, area.height()).into()) * p,
                )
            })
            .collect();

        let line_w = 3.0;
        for (c, p) in paths {
            let b = r.solid_brush(c.into());
            r.stroke(p, &b, line_w);
        }

        let (step_x, step_y) = (self.steps.x as f64, self.steps.y as f64);
        let steps_y: Vec<_> = (0..area.height() as u64 + step_y as u64)
            .step_by(step_y as usize)
            .collect();
        let steps_x: Vec<_> = (0..area.width() as u64 + step_x as u64)
            .step_by(step_x as usize)
            .collect();

        let steps = XYPoint {
            x: steps_x,
            y: steps_y,
        };
        let xylines = XYPoint {
            x: 0.0,
            y: area.height(),
        };
        let (x_offset, y_offset) = self.mk_labels(&steps, &xylines, &label_font, r)?;
        r.render_text(
            area.center() + (x_offset - area.center().x, 0.0),
            &TextInfo::new(self.axis.y.clone()).font(label_font.clone()),
        )?;

        r.render_text(
            (area.center().x, area.max_y() + y_offset).into(),
            &TextInfo::new(self.axis.x.clone()).font(label_font.clone()),
        )?;
        mk_grids(
            &self.grid.clone().unwrap_or(XYPoint { x: false, y: true }),
            &steps,
            &XYPoint {
                x: (0.0, area.width()),
                y: (0.0, area.height()),
            },
            r,
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{chart::DataPoint, render::Colour};

    use super::*;
}
