use rlua::{FromLua, Table, Value};

use super::{BarChart, Chart, ChartType, Charts, DataPoint, XYScatter};

impl<'lua, C: FromLua<'lua>> FromLua<'lua> for DataPoint<C> {
    fn from_lua(lua_value: Value<'lua>, lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        todo!()
    }
}

impl<'lua, C: ChartType + FromLua<'lua>> FromLua<'lua> for Chart<C>
where
    C::DataPoint: FromLua<'lua>,
{
    fn from_lua(lua_value: rlua::Value<'lua>, lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        match lua_value.clone() {
            Value::Table(t) => {
                let datasets: Vec<_> = t.get("datasets")?;
                let extra = FromLua::from_lua(lua_value, lua)?;
                Ok(Self { datasets, extra })
            }
            _ => Err(rlua::Error::FromLuaConversionError {
                from: &lua_value.type_name(),
                to: "Chart",
                message: None,
            }),
        }
    }
}

impl<'lua> FromLua<'lua> for Charts {
    fn from_lua(lua_value: Value<'lua>, lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        match lua_value {
            Value::Table(t) => {
                let ctype: String = t.get("type")?;
                match ctype.as_str() {
                    "xy-scatter" => Ok(Charts::XYScatter(Chart::<XYScatter>::from_lua(
                        lua_value, lua,
                    )?)),
                    "bar" => Ok(Charts::Bar(FromLua::from_lua(lua_value, lua)?)),
                    _ => Err(rlua::Error::RuntimeError(format!(
                        "invalid type for chart '{}'",
                        ctype
                    ))),
                }
            }
            _ => Err(rlua::Error::FromLuaConversionError {
                from: lua_value.type_name(),
                to: "Charts",
                message: None,
            }),
        }
    }
}

impl<'lua> FromLua<'lua> for XYScatter {
    fn from_lua(lua_value: rlua::Value<'lua>, lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        Ok(XYScatter {})
    }
}

impl<'lua> FromLua<'lua> for BarChart {
    fn from_lua(lua_value: rlua::Value<'lua>, lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        Ok(BarChart {})
    }
}
