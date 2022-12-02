use std::{borrow::Borrow, rc::Rc};

use rlua::{Lua, Value};
use serde::{
    de, forward_to_deserialize_any,
    ser::{self, SerializeMap},
    Deserialize,
};
use thiserror::Error;

#[derive(Clone)]
pub struct Deserializer<'lua> {
    input: Value<'lua>,
}

pub fn from_lua<'de, V: Deserialize<'de>>(v: Value) -> Result<V, DeErr> {
    let de = Deserializer { input: v };
    V::deserialize(de)
}

impl<'de, 'lua> Deserializer<'lua> {
    pub fn new(input: Value<'lua>) -> Self {
        Self { input }
    }
}

#[derive(Error, Debug)]
pub enum LuaDeserializeErr {
    #[error("lua error {0}")]
    Lua(rlua::Error),
    #[error("serde err: {0}")]
    Custom(String),
    #[error("wrong type expecting {0} found {1}")]
    WrongType(String, String),
    #[error("{0}")]
    Other(Box<dyn std::error::Error>),
    #[error("expecting length {0} got {1}")]
    WrongLength(usize, usize),
}
fn type_err<R>(f: &Value, t: &str) -> Result<R, LuaDeserializeErr> {
    Err(LuaDeserializeErr::WrongType(
        f.type_name().to_owned(),
        t.to_owned(),
    ))
}
unsafe impl Send for LuaDeserializeErr {}
unsafe impl Sync for LuaDeserializeErr {}

type DeErr = LuaDeserializeErr;

impl de::Error for LuaDeserializeErr {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        LuaDeserializeErr::Custom(msg.to_string())
    }
}
impl ser::Error for LuaDeserializeErr {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        LuaDeserializeErr::Custom(msg.to_string())
    }
}
impl From<rlua::Error> for LuaDeserializeErr {
    fn from(e: rlua::Error) -> Self {
        Self::Lua(e)
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

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        todo!()
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
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

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input.clone() {
            Value::Boolean(b) => visitor.visit_bool(b),
            Value::Nil => visitor.visit_none(),
            Value::Integer(n) => visitor.visit_i64(n),
            Value::Number(n) => visitor.visit_f64(n),
            Value::String(s) => visitor.visit_string(s.to_str().unwrap().to_owned()),
            Value::Table(t) => {
                if t.raw_len() > 0 {
                    self.input.as_table(|t| visitor.visit_seq(TableSeq::new(t)))
                } else {
                    self.input.as_table(|t| visitor.visit_map(TableMap::new(t)))
                }
            }
            Value::Error(e) => Err(DeErr::Lua(e)),
            _ => unimplemented!(),
        }
    }
    forward_to_deserialize_any! {bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
    bytes byte_buf option unit unit_struct newtype_struct seq tuple
    tuple_struct map struct enum identifier ignored_any}
}

struct Serializer<'lua> {
    lua: rlua::Context<'lua>,
}

impl<'lua> Serializer<'lua> {
    fn new(lua: rlua::Context<'lua>) -> Self {
        Self { lua }
    }

    fn to_lua<C: rlua::ToLua<'lua>>(self, v: C) -> Result<Value<'lua>, DeErr> {
        Ok(v.to_lua(self.lua)?)
    }
}

struct SeqSerializer<'lua> {
    vals: Vec<Value<'lua>>,
    ctx: rlua::Context<'lua>,
}

impl<'lua> ser::SerializeSeq for SeqSerializer<'lua> {
    type Ok = Value<'lua>;
    type Error = DeErr;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        self.vals
            .push(value.serialize(Serializer::new(self.ctx.clone()))?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let tbl = self
            .ctx
            .create_table_from(self.vals.into_iter().enumerate().map(|(k, v)| (k + 1, v)))?;
        Ok(Value::Table(tbl))
    }
}
impl<'a> ser::SerializeTuple for SeqSerializer<'a> {
    type Ok = Value<'a>;
    type Error = DeErr;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        <Self as ser::SerializeSeq>::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as ser::SerializeSeq>::end(self)
    }
}
impl<'lua> ser::SerializeTupleStruct for SeqSerializer<'lua> {
    type Ok = Value<'lua>;
    type Error = DeErr;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        <Self as ser::SerializeSeq>::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as ser::SerializeSeq>::end(self)
    }
}
impl<'lua> ser::SerializeTupleVariant for SeqSerializer<'lua> {
    type Ok = Value<'lua>;
    type Error = DeErr;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        <Self as ser::SerializeSeq>::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as ser::SerializeSeq>::end(self)
    }
}
struct MapSerialize<'lua> {
    lua: rlua::Context<'lua>,
    keys: Vec<Value<'lua>>,
    values: Vec<Value<'lua>>,
}
impl<'lua> ser::SerializeMap for MapSerialize<'lua> {
    type Ok = Value<'lua>;
    type Error = DeErr;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        self.keys
            .push(key.serialize(Serializer::new(self.lua.clone()))?);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        self.values
            .push(value.serialize(Serializer::new(self.lua.clone()))?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Table(self.lua.create_table_from(
            self.keys.into_iter().zip(self.values.into_iter()),
        )?))
    }
}
impl<'lua> ser::SerializeStruct for MapSerialize<'lua> {
    type Ok = Value<'lua>;
    type Error = DeErr;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        <Self as SerializeMap>::serialize_key(self, key)?;
        <Self as SerializeMap>::serialize_value(self, value)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as SerializeMap>::end(self)
    }
}
impl<'lua> ser::SerializeStructVariant for MapSerialize<'lua> {
    type Ok = Value<'lua>;
    type Error = DeErr;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        <Self as SerializeMap>::serialize_key(self, key)?;
        <Self as SerializeMap>::serialize_value(self, value)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as SerializeMap>::end(self)
    }
}

impl<'lua> serde::ser::Serializer for Serializer<'lua> {
    type Ok = Value<'lua>;

    type Error = DeErr;

    type SerializeSeq = SeqSerializer<'lua>;

    type SerializeTuple = SeqSerializer<'lua>;

    type SerializeTupleStruct = SeqSerializer<'lua>;

    type SerializeTupleVariant = SeqSerializer<'lua>;

    type SerializeMap = MapSerialize<'lua>;

    type SerializeStruct = MapSerialize<'lua>;

    type SerializeStructVariant = MapSerialize<'lua>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.to_lua(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.to_lua(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.to_lua(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.to_lua(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.to_lua(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.to_lua(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.to_lua(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.to_lua(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.to_lua(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.to_lua(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.to_lua(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.to_lua(v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.to_lua(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.to_lua(v.to_vec())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Nil)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Nil)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Nil)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Nil)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let mut vals = Vec::new();
        vals.reserve(len.unwrap_or(0));
        Ok(SeqSerializer {
            vals,
            ctx: self.lua.clone(),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(MapSerialize {
            lua: self.lua.clone(),
            keys: Vec::with_capacity(len),
            values: Vec::with_capacity(len),
        })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}
