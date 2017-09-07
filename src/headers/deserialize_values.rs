use serde::de::{Deserializer, DeserializeSeed, Visitor, EnumAccess, VariantAccess, SeqAccess};

use std::error::Error;
use std::vec::IntoIter;
use std::ops::Deref;
use std::marker::PhantomData;

use headers::HeadersDeserializationError;

pub(super) trait VisitableString<'de>: Deref<Target = str> {
    fn be_visited<V>(self, visitor: V) -> Result<V::Value, HeadersDeserializationError>
    where
        V: Visitor<'de>;
}

impl<'de> VisitableString<'de> for String {
    fn be_visited<V>(self, visitor: V) -> Result<V::Value, HeadersDeserializationError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self)
    }
}

impl<'de, 'a: 'de> VisitableString<'de> for &'a str {
    fn be_visited<V>(self, visitor: V) -> Result<V::Value, HeadersDeserializationError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self)
    }
}

pub(super) struct DeserializeValue<'de, S>
where
    S: VisitableString<'de>,
{
    value: S,
    phantom: PhantomData<&'de str>,
}

impl<'de, S> DeserializeValue<'de, S>
where
    S: VisitableString<'de>,
{
    pub(super) fn new(value: S) -> Self {
        DeserializeValue {
            value,
            phantom: PhantomData,
        }
    }
}

fn translate_parse_error<E>(e: E) -> HeadersDeserializationError
where
    E: Error,
{
    unimplemented!()
}

macro_rules! primitive {
    ($fn:ident, $visit_fn:ident) => {
        fn $fn<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>
        {
            match self.value.parse() {
                Ok(v) => visitor.$visit_fn(v),
                Err(e) => Err(translate_parse_error(e))
            }
        }
    }
}

impl<'de, S> Deserializer<'de> for DeserializeValue<'de, S>
where
    S: VisitableString<'de>,
{
    type Error = HeadersDeserializationError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.value.be_visited(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(ValueEnum::new(self.value))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(MultiValued::new(self.value))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    primitive!(deserialize_bool, visit_bool);
    primitive!(deserialize_i8, visit_i8);
    primitive!(deserialize_i16, visit_i16);
    primitive!(deserialize_i32, visit_i32);
    primitive!(deserialize_i64, visit_i64);
    primitive!(deserialize_u8, visit_u8);
    primitive!(deserialize_u16, visit_u16);
    primitive!(deserialize_u32, visit_u32);
    primitive!(deserialize_u64, visit_u64);
    primitive!(deserialize_f32, visit_f32);
    primitive!(deserialize_f64, visit_f64);

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value.chars().next() {
            Some(c) => visitor.visit_char(c),
            None => Err(HeadersDeserializationError::InvalidState {
                msg: "empty string provided for HTTP header, unable to extract char value",
            }),
        }
    }

    forward_to_deserialize_any! {
        bytes
        byte_buf unit unit_struct newtype_struct tuple
        tuple_struct map struct
    }
}

struct MultiValued {
    value_iter: IntoIter<String>,
}

impl MultiValued {
    fn new<'de, S>(value: S) -> Self
    where
        S: VisitableString<'de>,
    {
        let mut curr = None;

        // For an attribute which has these three values:
        //
        // value1\
        // value2\
        // value3\
        //
        // ... the multi-valued attribute string is represented as:
        //
        // value1\;value2\;value3\
        //
        // This is impossible to distinguish from a single attribute value of:
        //
        // value1;value2;value3\
        //
        // This is deliberate behaviour in shib-gotham to correctly handle what we get from
        // `mod_shib`. This exact example has a test case.
        let iter = str::split(&value, |c| {
            let prev = curr;
            curr = Some(c);

            match prev {
                Some('\\') => false,
                _ => c == ';',
            }
        });

        let values: Vec<String> = iter.map(|s| s.replace(r"\;", ";")).collect();
        MultiValued { value_iter: values.into_iter() }
    }
}

impl<'de> SeqAccess<'de> for MultiValued {
    type Error = HeadersDeserializationError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value_iter.next() {
            Some(v) => {
                let de = DeserializeValue::new(v);
                Ok(Some(seed.deserialize(de)?))
            }
            None => Ok(None),
        }
    }
}

struct ValueEnum<'de, S>
where
    S: VisitableString<'de>,
{
    value: S,
    phantom: PhantomData<&'de str>,
}

impl<'de, S> ValueEnum<'de, S>
where
    S: VisitableString<'de>,
{
    fn new(value: S) -> Self {
        ValueEnum {
            value,
            phantom: PhantomData,
        }
    }
}

impl<'de, S> EnumAccess<'de> for ValueEnum<'de, S>
where
    S: VisitableString<'de>,
{
    type Error = HeadersDeserializationError;
    type Variant = UnitVariant;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        Ok((
            seed.deserialize(DeserializeValue::new(self.value))?,
            UnitVariant,
        ))
    }
}

struct UnitVariant;

impl<'de> VariantAccess<'de> for UnitVariant {
    type Error = HeadersDeserializationError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        unimplemented!()
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
}
