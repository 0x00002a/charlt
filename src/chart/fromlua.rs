use rlua::{FromLua, Value};

use super::{Chart, ChartType, DataPoint};

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
                to: "table",
                message: None,
            }),
        }
    }
}
