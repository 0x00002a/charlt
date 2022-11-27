use rlua::{FromLua, Value};

use crate::render::{Entity, Render};

use super::{Chart, ChartType};

#[derive(Clone, Debug)]
pub enum Charts {
    XYScatter(Chart<XYScatter>),
    Bar(Chart<BarChart>),
}

#[derive(Clone, Debug)]
pub struct XYPoint {
    x: f64,
    y: f64,
}
impl From<XYPoint> for geo::Coord {
    fn from(me: XYPoint) -> Self {
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
        datasets: &Vec<Vec<Self::DataPoint>>,
        area: &geo::Rect,
    ) -> Vec<Vec<geo::Geometry>> {
        todo!()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct XYScatter {}

impl ChartType for XYScatter {
    type DataPoint = XYPoint;
    const NAME: &'static str = "xy-scatter";

    fn render_datasets(
        &self,
        datasets: &Vec<Vec<Self::DataPoint>>,
        _: &geo::Rect,
    ) -> Vec<Vec<geo::Geometry>> {
        let ds = datasets
            .iter()
            .map(|sets| {
                let mut out = Vec::new();
                for n in 1..sets.len() {
                    let curr_pt = sets[n].clone();
                    let last_pt = sets[n - 1].clone();
                    let pt = geo::Line::new(last_pt, curr_pt);
                    out.push(pt.into());
                }
                out
            })
            .collect();
        ds
    }
}

impl<'lua> FromLua<'lua> for XYPoint {
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
            extra: XYScatter {},
        };
        let rendered = c.render();
        assert_eq!(rendered.len(), 2);
    }
}
