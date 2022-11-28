use rlua::{Lua, Value};
use serde::{
    de::{self, SeqAccess},
    Deserialize,
};
use thiserror::Error;

struct Deserializer<'lua> {
    input: Value<'lua>,
}

pub fn from_lua<'de, V: Deserialize<'de>>(v: Value) -> Result<V, DeErr> {
    let de = Deserializer { input: v };
    V::deserialize(de)
}

impl<'de, 'lua> Deserializer<'lua> {}

#[derive(Error, Debug)]
pub enum LuaDeserializeErr {
    #[error("lua error {0}")]
    Lua(rlua::Error),
    #[error("{0}")]
    Custom(String),
    #[error("wrong type expecting {0} found {1}")]
    WrongType(String, String),
    #[error("{0}")]
    Other(Box<dyn std::error::Error>),
    #[error("expecting length {0} got {1}")]
    WrongLength(usize, usize),
    #[error("unimplemented")]
    Unimplemented,
}
fn type_err<R>(f: &Value, t: &str) -> Result<R, LuaDeserializeErr> {
    Err(LuaDeserializeErr::WrongType(
        f.type_name().to_owned(),
        t.to_owned(),
    ))
}
unsafe impl Send for LuaDeserializeErr {}
unsafe impl Sync for LuaDeserializeErr {}

fn unimpl<R>() -> Result<R, LuaDeserializeErr> {
    Err(LuaDeserializeErr::Unimplemented)
}

type DeErr = LuaDeserializeErr;

impl de::Error for LuaDeserializeErr {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        LuaDeserializeErr::Custom(msg.to_string())
    }
}
trait ValueExt<'lua> {
    fn as_table<F: FnOnce(rlua::Table) -> Result<R, LuaDeserializeErr>, R>(
        self,
        f: F,
    ) -> Result<R, LuaDeserializeErr>;

    fn as_str<F: FnOnce(&str) -> Result<R, LuaDeserializeErr>, R>(
        self,
        f: F,
    ) -> Result<R, LuaDeserializeErr>;
}
impl<'lua> ValueExt<'lua> for Value<'lua> {
    fn as_table<F: FnOnce(rlua::Table<'lua>) -> Result<R, LuaDeserializeErr>, R>(
        self,
        f: F,
    ) -> Result<R, LuaDeserializeErr> {
        match self {
            Value::Table(t) => f(t),
            _ => type_err(&self, "table"),
        }
    }

    fn as_str<F: FnOnce(&str) -> Result<R, LuaDeserializeErr>, R>(
        self,
        f: F,
    ) -> Result<R, LuaDeserializeErr> {
        match self {
            Value::String(s) => match s.to_str() {
                Err(e) => Err(DeErr::Lua(e)),
                Ok(s) => f(s),
            },
            _ => type_err(&self, "string"),
        }
    }
}
struct Enum<'lua> {
    v: rlua::String<'lua>,
}

impl<'lua> Enum<'lua> {
    fn new(v: rlua::String<'lua>) -> Self {
        Self { v }
    }
}

impl<'de, 'lua> de::EnumAccess<'de> for Enum<'lua> {
    type Error = DeErr;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let val = seed.deserialize(Deserializer {
            input: Value::String(self.v.clone().into()),
        })?;
        Ok((val, self))
    }
}
impl<'de, 'lua> de::VariantAccess<'de> for Enum<'lua> {
    type Error = DeErr;

    fn unit_variant(self) -> Result<(), Self::Error> {
        todo!()
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        todo!()
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }
}

struct TableSeq<'lua> {
    values: Vec<Value<'lua>>,
}
struct TableMap<'lua> {
    keys: Vec<Value<'lua>>,
    values: Vec<Value<'lua>>,
}
impl<'lua> TableSeq<'lua> {
    fn new(tbl: rlua::Table<'lua>) -> Self {
        let mut values: Vec<_> = tbl
            .sequence_values()
            .into_iter()
            .filter(|r| r.is_ok())
            .map(|r| r.unwrap())
            .collect();
        values.reverse();
        Self { values }
    }
}

impl<'lua> TableMap<'lua> {
    fn new(tbl: rlua::Table<'lua>) -> Self {
        let (mut keys, mut values) = tbl
            .pairs()
            .into_iter()
            .filter(|k| k.is_ok())
            .map(|k| k.unwrap())
            .fold((Vec::new(), Vec::new()), |(mut ks, mut vs), (k, v)| {
                ks.push(k);
                vs.push(v);
                (ks, vs)
            });
        keys.reverse();
        values.reverse();

        Self { keys, values }
    }
}

impl<'de, 'lua> de::MapAccess<'de> for TableMap<'lua> {
    type Error = DeErr;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.keys.pop() {
            None => Ok(None),
            Some(k) => seed.deserialize(Deserializer { input: k }).map(Some),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.values.pop() {
            None => Err(DeErr::WrongLength(1, 0)),
            Some(v) => seed.deserialize(Deserializer { input: v }),
        }
    }
}

impl<'de, 'lua> de::SeqAccess<'de> for TableSeq<'lua> {
    type Error = DeErr;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.values.len() == 0 {
            Ok(None)
        } else {
            let v = self.values.pop().unwrap();
            seed.deserialize(Deserializer { input: v }).map(Some)
        }
    }
}

impl<'de, 'lua> de::Deserializer<'de> for Deserializer<'lua> {
    type Error = LuaDeserializeErr;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimpl()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input {
            Value::Boolean(b) => visitor.visit_bool(b),
            _ => type_err(&self.input, "bool"),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input {
            Value::Integer(i) => match i.try_into() {
                Err(e) => Err(DeErr::Other(Box::new(e))),
                Ok(i) => visitor.visit_i8(i),
            },
            _ => type_err(&self.input, "i8"),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input {
            Value::Integer(i) => match i.try_into() {
                Err(e) => Err(DeErr::Other(Box::new(e))),
                Ok(i) => visitor.visit_i16(i),
            },
            _ => type_err(&self.input, "i16"),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input {
            Value::Integer(i) => match i.try_into() {
                Err(e) => Err(DeErr::Other(Box::new(e))),
                Ok(i) => visitor.visit_i32(i),
            },
            _ => type_err(&self.input, "i32"),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input {
            Value::Integer(i) => match i.try_into() {
                Err(e) => Err(DeErr::Other(Box::new(e))),
                Ok(i) => visitor.visit_i64(i),
            },
            _ => type_err(&self.input, "i64"),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input {
            Value::Integer(i) => match i.try_into() {
                Err(e) => Err(DeErr::Other(Box::new(e))),
                Ok(i) => visitor.visit_u8(i),
            },
            _ => type_err(&self.input, "u8"),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input {
            Value::Integer(i) => match i.try_into() {
                Err(e) => Err(DeErr::Other(Box::new(e))),
                Ok(i) => visitor.visit_u16(i),
            },
            _ => type_err(&self.input, "u16"),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input {
            Value::Integer(i) => match i.try_into() {
                Err(e) => Err(DeErr::Other(Box::new(e))),
                Ok(i) => visitor.visit_u32(i),
            },
            _ => type_err(&self.input, "u32"),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input {
            Value::Integer(i) => match i.try_into() {
                Err(e) => Err(DeErr::Other(Box::new(e))),
                Ok(i) => visitor.visit_u64(i),
            },
            _ => type_err(&self.input, "u64"),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input {
            Value::Number(i) => match i.try_into() {
                Err(e) => Err(DeErr::Other(Box::new(e))),
                Ok(i) => visitor.visit_f64(i),
            },
            _ => type_err(&self.input, "f64"),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input {
            Value::Number(i) => match i.try_into() {
                Err(e) => Err(DeErr::Other(Box::new(e))),
                Ok(i) => visitor.visit_f64(i),
            },
            _ => type_err(&self.input, "f64"),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input {
            Value::String(s) => match s.to_str() {
                Err(e) => Err(DeErr::Lua(e)),
                Ok(s) => {
                    if s.len() == 1 {
                        visitor.visit_char(s.chars().nth(0).unwrap())
                    } else {
                        Err(DeErr::WrongLength(1, s.len()))
                    }
                }
            },
            _ => type_err(&self.input, "string"),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input {
            Value::String(s) => match s.to_str() {
                Err(e) => Err(DeErr::Lua(e)),
                Ok(s) => visitor.visit_str(s),
            },
            _ => type_err(&self.input, "string"),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input {
            Value::String(s) => match s.to_str() {
                Err(e) => Err(DeErr::Lua(e)),
                Ok(s) => visitor.visit_string(s.to_owned()),
            },
            _ => type_err(&self.input, "string"),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimpl()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimpl()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input {
            Value::Nil => visitor.visit_none(),
            v => visitor.visit_some(Deserializer { input: v }),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input {
            Value::Nil => visitor.visit_unit(),
            _ => type_err(&self.input, "unit"),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.input.as_table(|t| match t.get(name) {
            Err(e) => Err(DeErr::Lua(e)),
            Ok(v) => match v {
                Value::Nil => visitor.visit_unit(),
                _ => type_err(&v, "unit"),
            },
        })
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.input.as_table(|t| match t.get(name) {
            Err(e) => Err(DeErr::Lua(e)),
            Ok(v) => visitor.visit_newtype_struct(Deserializer { input: v }),
        })
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.input.as_table(|t| visitor.visit_seq(TableSeq::new(t)))
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.input.as_table(|t| visitor.visit_map(TableMap::new(t)))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input {
            Value::String(s) => visitor.visit_enum(Enum::new(s)),
            _ => type_err(&self.input, "string"),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}
