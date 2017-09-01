use serde::de::{Deserializer, DeserializeSeed, Visitor, EnumAccess, VariantAccess};

use std::borrow::Cow;
use std::marker::PhantomData;

use headers::HeadersDeserializationError;

pub(super) trait VisitableString<'de> {
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

impl<'de, S> Deserializer<'de> for DeserializeValue<'de, S>
where
    S: VisitableString<'de>,
{
    type Error = HeadersDeserializationError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
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
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(ValueEnum::new(self.value))
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct
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

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        unimplemented!()
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
}
