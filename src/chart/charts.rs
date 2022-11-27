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
    ) -> Vec<geo::GeometryCollection> {
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
    ) -> Vec<geo::GeometryCollection> {
        datasets
            .iter()
            .map(|sets| {
                let mut out = Vec::new();
                for n in 1..out.len() {
                    let curr_pt = sets[n].clone();
                    let last_pt = sets[n - 1].clone();
                    let pt = geo::Line::new(last_pt, curr_pt);
                    out.push(pt.into());
                }
                geo::GeometryCollection::new_from(out)
            })
            .collect()
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
