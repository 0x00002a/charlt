use rlua::{FromLua, Value};

use super::Colour;

impl<'lua> FromLua<'lua> for Colour {
    fn from_lua(lua_value: rlua::Value<'lua>, lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        match lua_value {
            Value::String(s) => Ok(Colour::HEX(s.to_str()?.to_owned())),
            _ => Err(rlua::Error::FromLuaConversionError {
                from: lua_value.type_name(),
                to: "Colour",
                message: Some("expected string".to_owned()),
            }),
        }
    }
}
