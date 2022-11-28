use geo::Scale;
use rlua::{FromLua, Value};

use crate::render::colours;
use crate::render::{Entity, Render, Shape};

use super::{Chart, ChartType, DataPoint};

#[derive(Clone, Debug)]
pub enum Charts {
    XYScatter(Chart<XYScatter>),
    Bar(Chart<BarChart>),
}

#[derive(Clone, Debug)]
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
impl From<XYPoint<f64>> for geo::Coord {
    fn from(me: XYPoint<f64>) -> Self {
        geo::Coord { x: me.x, y: me.y }
    }
}

type BarPoint = f64;
#[derive(Clone, Copy, Debug)]
pub struct BarChart {}
impl ChartType for BarChart {
    type DataPoint = BarPoint;

    const NAME: &'static str = "bar";

    fn render_datasets(
        &self,
        datasets: &Vec<super::DataPoint<Self::DataPoint>>,
        area: &geo::Rect,
    ) -> Vec<Entity> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct XYScatter {
    pub axis: XYPoint<String>,
}

impl ChartType for XYScatter {
    type DataPoint = XYPoint<f64>;
    const NAME: &'static str = "xy-scatter";

    fn render_datasets(
        &self,
        datasets: &Vec<DataPoint<XYPoint<f64>>>,
        area: &geo::Rect,
    ) -> Vec<Entity> {
        let mut ds = Vec::new();
        let mut max_x: f64 = 0.0;
        let mut max_y: f64 = 0.0;
        for point in datasets {
            let sets = &point.values;
            let mut out = Vec::new();
            for n in 1..sets.len() {
                let curr_pt = sets[n].clone();
                let last_pt = sets[n - 1].clone();
                max_x = max_x.max(last_pt.x);
                max_y = max_y.max(last_pt.y);
                max_x = max_x.max(curr_pt.x);
                max_y = max_y.max(curr_pt.y);
                let pt = geo::Line::new(last_pt, curr_pt);
                out.push(pt.into());
            }
            ds.push((point.colour.clone(), geo::GeometryCollection::new_from(out)));
        }
        let scale_x = area.width() / max_x;
        let scale_y = area.height() / max_y;
        let mut ds: Vec<_> = ds
            .into_iter()
            .map(|(c, s)| {
                Entity::new(
                    c,
                    geo::Geometry::GeometryCollection(s.scale_xy(scale_x, scale_y)).into(),
                )
            })
            .collect();
        ds.push(Entity::new(
            colours::BLACK,
            Shape::Text {
                pos: area.center() + geo::Coord::from((0.0, area.height())) / 2.0,
                content: self.axis.x.to_string(),
            },
        ));
        ds.push(Entity::new(
            colours::BLACK,
            Shape::Text {
                pos: area.center() + geo::Coord::from((area.width(), 0.0)) / 2.0,
                content: self.axis.y.to_string(),
            },
        ));
        ds
    }
}

impl<'lua, T: FromLua<'lua>> FromLua<'lua> for XYPoint<T> {
    fn from_lua(lua_value: rlua::Value<'lua>, lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        match lua_value {
            Value::Table(t) => Ok(Self {
                x: t.get("x")?,
                y: t.get("y")?,
            }),
            _ => Err(rlua::Error::FromLuaConversionError {
                from: lua_value.type_name(),
                to: "XYPoint",
                message: Some("not a table?".to_owned()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {

    use geo::Rect;

    use crate::{chart::DataPoint, render::Colour};

    use super::*;
    #[test]
    fn render_gives_all_elements() {
        let mut datasets = Vec::new();
        datasets.push(DataPoint {
            name: "test".to_owned(),
            colour: Colour::RGB(0, 0, 255),
            values: vec![
                XYPoint { x: 0.0, y: 10.0 },
                XYPoint { x: 1.0, y: 20.0 },
                XYPoint { x: 2.0, y: 30.0 },
            ],
        });
        let c = Chart {
            datasets,
            extra: XYScatter {
                axis: XYPoint::new("h", "m"),
            },
        };
        let rendered = c.render(&Rect::new((0.0, 0.0), (10.0, 50.0))).unwrap();
        assert_eq!(rendered.len(), 2);
    }
}
