use std::str;
use std::collections::BTreeMap;

use serde::de::{DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use hyper::header::{HeaderView, Headers};

use headers::HeadersDeserializationError;
use headers::deserialize_values::DeserializeValue;

pub(super) struct DeserializeHeaders<'a> {
    headers: &'a Headers,
}

impl<'a> DeserializeHeaders<'a> {
    pub(super) fn new(headers: &'a Headers) -> Self {
        DeserializeHeaders { headers }
    }
}

macro_rules! reject {
    {$fn:ident, $msg:expr} => {
        fn $fn<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>
        {
            Err(HeadersDeserializationError::InvalidTopLevelType { msg: $msg })
        }
    };

    {$fn:ident, $msg:expr, ($($arg_i:ident : $arg_t:ty),*)} => {
        fn $fn<V>(self, $($arg_i : $arg_t),*, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>
        {
            Err(HeadersDeserializationError::InvalidTopLevelType { msg: $msg })
        }
    }
}

impl<'de, 'a: 'de> Deserializer<'de> for DeserializeHeaders<'a> {
    type Error = HeadersDeserializationError;

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mappings: BTreeMap<String, &'static str> = fields.iter().cloned()
            // Avoid copying anything which is already lower case
            .filter_map(|a| if a.chars().any(|c| c.is_uppercase()) {
            Some((a.to_lowercase(), a))
        } else {
            None
        })
        .collect();

        visitor.visit_map(AccessHeaders {
            iter: self.headers.iter(),
            mappings,
            current: None,
        })
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(TupleAccess {
            headers: self.headers,
        })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(AccessHeaders {
            iter: self.headers.iter(),
            mappings: BTreeMap::new(),
            current: None,
        })
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    reject!(
        deserialize_bool,
        "unsuitable scalar type (bool) as top-level user attributes type"
    );
    reject!(
        deserialize_i8,
        "unsuitable scalar type (i8) as top-level user attributes type"
    );
    reject!(
        deserialize_i16,
        "unsuitable scalar type (i16) as top-level user attributes type"
    );
    reject!(
        deserialize_i32,
        "unsuitable scalar type (i32) as top-level user attributes type"
    );
    reject!(
        deserialize_i64,
        "unsuitable scalar type (i64) as top-level user attributes type"
    );
    reject!(
        deserialize_u8,
        "unsuitable scalar type (u8) as top-level user attributes type"
    );
    reject!(
        deserialize_u16,
        "unsuitable scalar type (u16) as top-level user attributes type"
    );
    reject!(
        deserialize_u32,
        "unsuitable scalar type (u32) as top-level user attributes type"
    );
    reject!(
        deserialize_u64,
        "unsuitable scalar type (u64) as top-level user attributes type"
    );
    reject!(
        deserialize_f32,
        "unsuitable scalar type (f32) as top-level user attributes type"
    );
    reject!(
        deserialize_f64,
        "unsuitable scalar type (f64) as top-level user attributes type"
    );
    reject!(
        deserialize_char,
        "unsuitable scalar type (char) as top-level user attributes type"
    );
    reject!(
        deserialize_str,
        "unsuitable type (str) as top-level user attributes type"
    );
    reject!(
        deserialize_string,
        "unsuitable type (String) as top-level user attributes type"
    );
    reject!(
        deserialize_bytes,
        "unsuitable type (bytes) as top-level user attributes type"
    );
    reject!(
        deserialize_byte_buf,
        "unsuitable type (byte buffer) as top-level user attributes type"
    );
    reject!(
        deserialize_option,
        "unsuitable type (Option<_>) as top-level user attributes type"
    );
    reject!(
        deserialize_enum,
        "unsuitable type (enum) as top-level user attributes type",
        (_name: &'static str, _variants: &'static [&'static str])
    );

    reject!(
        deserialize_seq,
        "unsuitable type (sequence) as top-level user attributes type"
    );

    reject!(
        deserialize_identifier,
        "unsuitable type (identifier) as top-level user attributes type"
    );

    reject!(
        deserialize_any,
        "unsuitable type (any) as top-level user attributes type"
    );
}

struct AccessHeaders<'a, Iter>
where
    Iter: Iterator<Item = HeaderView<'a>> + 'a,
{
    iter: Iter,
    mappings: BTreeMap<String, &'a str>,
    current: Option<HeaderView<'a>>,
}

impl<'de, 'a: 'de, Iter> MapAccess<'de> for AccessHeaders<'a, Iter>
where
    Iter: Iterator<Item = HeaderView<'a>> + 'a,
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
                let name = header.name().to_lowercase();
                let key = match self.mappings.get(&name) {
                    Some(&n) => seed.deserialize(DeserializeValue::new(n)),
                    None => seed.deserialize(DeserializeValue::new(name)),
                };

                Ok(Some(key?))
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

struct TupleAccess<'a> {
    headers: &'a Headers,
}

impl<'de, 'a: 'de> SeqAccess<'de> for TupleAccess<'a> {
    type Error = HeadersDeserializationError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let deserializer = DeserializeHeaders::new(self.headers);
        Ok(Some(seed.deserialize(deserializer)?))
    }
}
