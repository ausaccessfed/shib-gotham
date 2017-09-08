use std::str;

use serde::de::{Deserializer, Visitor, MapAccess, DeserializeSeed};
use hyper::header::HeaderView;

use headers::HeadersDeserializationError;
use headers::deserialize_values::DeserializeValue;

pub struct DeserializeHeaders<'a, Iter>
where
    Iter: Iterator<Item = HeaderView<'a>> + 'a,
{
    iter: Iter,
}

impl<'a, Iter> DeserializeHeaders<'a, Iter>
where
    Iter: Iterator<Item = HeaderView<'a>> + 'a,
{
    pub(super) fn new(iter: Iter) -> Self {
        DeserializeHeaders { iter }
    }
}

impl<'de, 'a: 'de, Iter> Deserializer<'de> for DeserializeHeaders<'a, Iter>
where
    Iter: Iterator<Item = HeaderView<'a>>,
{
    type Error = HeadersDeserializationError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(AccessHeaders { iter: self.iter, current: None })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map enum identifier ignored_any
    }
}

struct AccessHeaders<'a, Iter>
where
    Iter: Iterator<Item = HeaderView<'a>> + 'a,
{
    iter: Iter,
    current: Option<HeaderView<'a>>,
}

impl<'de, 'a: 'de, Iter> MapAccess<'de> for AccessHeaders<'a, Iter>
where
    Iter: Iterator<Item = HeaderView<'a>>
        + 'a,
{
    type Error = HeadersDeserializationError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        self.current = None;

        while let Some(c) = self.iter.next() {
            if c.raw().len() == 1 {
                self.current = Some(c);
                break;
            }
        }

        match self.current {
            Some(ref header) => {
                let deserializer = DeserializeValue::new(header.name());
                Ok(Some(seed.deserialize(deserializer)?))
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.current {
            Some(ref header) => {
                let deserializer = DeserializeValue::new(header.value_string());
                Ok(seed.deserialize(deserializer)?)
            }
            None => unreachable!("header name but no value?"),
        }
    }
}
