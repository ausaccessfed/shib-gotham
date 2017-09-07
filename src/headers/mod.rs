mod deserialize_headers;
mod deserialize_values;

use std::{error, fmt};
use std::error::Error;
use serde::de::{self, Deserialize};
use hyper::Headers;

#[derive(Debug)]
enum HeadersDeserializationError {
    InvalidTopLevelType { msg: &'static str },
    InvalidState { msg: &'static str },
}

impl error::Error for HeadersDeserializationError {
    fn description(&self) -> &str {
        "unable to deserialize from HTTP headers"
    }
}

impl de::Error for HeadersDeserializationError {
    fn custom<T>(_: T) -> Self {
        unimplemented!()
    }
}

impl fmt::Display for HeadersDeserializationError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        out.write_str("HeadersDeserializationError(")?;
        out.write_str(self.description())?;
        out.write_str(")")
    }
}

#[allow(dead_code)]
fn deserialize<T>(headers: &Headers) -> Result<T, HeadersDeserializationError>
where
    for<'de> T: Deserialize<'de>,
{
    let deserializer = deserialize_headers::DeserializeHeaders::new(headers);
    T::deserialize(deserializer)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use serde_bytes;
    use hyper::{Method, Headers};

    #[test]
    fn test_deserialize_unit() {
        let mut headers = Headers::new();
        deserialize::<()>(&headers).unwrap();
    }

    #[derive(Deserialize)]
    struct NoAttributes;

    #[test]
    fn test_deserialize_unit_struct() {
        let mut headers = Headers::new();
        deserialize::<NoAttributes>(&headers).unwrap();
    }

    #[derive(Deserialize)]
    struct SingleAttribute {
        #[serde(rename = "auEduPersonSharedToken")]
        shared_token: String,
    }

    #[test]
    fn test_deserialize_single() {
        let value = "BuyTkNadqZW_wYOeY4ppThkRRYE";

        let mut headers = Headers::new();
        headers.set_raw("auEduPersonSharedToken", value);

        let attrs: SingleAttribute = deserialize::<SingleAttribute>(&headers).unwrap();
        assert_eq!(&attrs.shared_token, value);
    }

    #[test]
    fn test_ignored_attribute() {
        let value = "BuyTkNadqZW_wYOeY4ppThkRRYE";

        let mut headers = Headers::new();
        headers.set_raw("auEduPersonSharedToken", value);
        headers.set_raw("anotherAttribute", "unused_value");

        let attrs: SingleAttribute = deserialize::<SingleAttribute>(&headers).unwrap();
        assert_eq!(&attrs.shared_token, value);
    }

    #[derive(Deserialize)]
    struct WrappedAttribute(SingleAttribute);

    #[test]
    fn test_newtype_struct() {
        let value = "BuyTkNadqZW_wYOeY4ppThkRRYE";

        let mut headers = Headers::new();
        headers.set_raw("auEduPersonSharedToken", value);
        headers.set_raw("anotherAttribute", "unused_value");

        let attrs = deserialize::<WrappedAttribute>(&headers).unwrap();
        assert_eq!(&attrs.0.shared_token, value);
    }

    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(rename_all = "kebab-case")]
    enum Affiliation {
        Faculty,
        Student,
        Staff,
        Employee,
        Member,
        Affiliate,
        Alum,
        LibraryWalkIn,
    }

    #[derive(Deserialize)]
    struct OnlyAffiliation {
        #[serde(rename = "eduPersonAffiliation")]
        affiliation: Affiliation,
    }

    #[test]
    fn test_enum_attribute() {
        let mut headers = Headers::new();
        headers.set_raw("eduPersonAffiliation", "library-walk-in");

        let attrs = deserialize::<OnlyAffiliation>(&headers).unwrap();
        assert_eq!(attrs.affiliation, Affiliation::LibraryWalkIn);

        let mut headers = Headers::new();
        headers.set_raw("eduPersonAffiliation", "employee");

        let attrs = deserialize::<OnlyAffiliation>(&headers).unwrap();
        assert_eq!(attrs.affiliation, Affiliation::Employee);
    }

    #[derive(Deserialize)]
    struct MultiValued {
        #[serde(rename = "eduPersonEntitlement")]
        entitlements: Vec<String>,
    }

    #[test]
    fn test_multi_valued_attribute() {
        let mut headers = Headers::new();
        headers.set_raw(
            "eduPersonEntitlement",
            "urn:x-aaf:dev:1;urn:x-aaf:dev:2;urn:x-aaf:dev:3",
        );

        let attrs = deserialize::<MultiValued>(&headers).unwrap();
        assert_eq!(
            &attrs.entitlements[..],
            &["urn:x-aaf:dev:1", "urn:x-aaf:dev:2", "urn:x-aaf:dev:3"]
        );

        let mut headers = Headers::new();
        // `\` is used to escape the `;` characters
        headers.set_raw("eduPersonEntitlement", r"value1\;value2\;value3\");

        let attrs = deserialize::<MultiValued>(&headers).unwrap();
        assert_eq!(&attrs.entitlements[..], &[r"value1;value2;value3\"]);
    }

    #[derive(Deserialize)]
    struct OptionalAttribute {
        #[serde(rename = "displayName")]
        display_name: Option<String>,
    }

    #[test]
    fn test_optional_attribute() {
        let attrs = deserialize::<OptionalAttribute>(&Headers::new()).unwrap();
        assert!(attrs.display_name.is_none());

        let mut headers = Headers::new();
        headers.set_raw("displayName", "John Doe");

        let attrs = deserialize::<OptionalAttribute>(&headers).unwrap();
        assert_eq!(attrs.display_name.unwrap(), "John Doe");
    }

    #[test]
    fn test_tuple() {
        let mut headers = Headers::new();
        headers.set_raw("displayName", "John Doe");
        headers.set_raw("eduPersonAffiliation", "library-walk-in");
        headers.set_raw(
            "eduPersonEntitlement",
            "urn:x-aaf:dev:1;urn:x-aaf:dev:2;urn:x-aaf:dev:3",
        );

        let (aff_attrs, mv_attrs, opt_attrs) =
            deserialize::<(OnlyAffiliation, MultiValued, OptionalAttribute)>(&headers).unwrap();

        assert_eq!(aff_attrs.affiliation, Affiliation::LibraryWalkIn);
        assert_eq!(
            &mv_attrs.entitlements[..],
            &["urn:x-aaf:dev:1", "urn:x-aaf:dev:2", "urn:x-aaf:dev:3"]
        );
        assert_eq!(opt_attrs.display_name.unwrap(), "John Doe");
    }

    #[derive(Deserialize)]
    struct TupleStruct(OnlyAffiliation, MultiValued, OptionalAttribute);

    #[test]
    fn test_tuple_struct() {
        let mut headers = Headers::new();
        headers.set_raw("displayName", "John Doe");
        headers.set_raw("eduPersonAffiliation", "library-walk-in");
        headers.set_raw(
            "eduPersonEntitlement",
            "urn:x-aaf:dev:1;urn:x-aaf:dev:2;urn:x-aaf:dev:3",
        );

        let (aff_attrs, mv_attrs, opt_attrs) =
            deserialize::<(OnlyAffiliation, MultiValued, OptionalAttribute)>(&headers).unwrap();

        assert_eq!(aff_attrs.affiliation, Affiliation::LibraryWalkIn);
        assert_eq!(
            &mv_attrs.entitlements[..],
            &["urn:x-aaf:dev:1", "urn:x-aaf:dev:2", "urn:x-aaf:dev:3"]
        );
        assert_eq!(opt_attrs.display_name.unwrap(), "John Doe");
    }

    #[test]
    fn test_map() {
        let mut headers = Headers::new();
        headers.set_raw("displayName", "John Doe");

        let attrs = deserialize::<HashMap<String, String>>(&headers).unwrap();

        assert_eq!(
            attrs.get("displayName").map(String::as_ref),
            Some("John Doe")
        );
    }

    #[derive(Deserialize)]
    struct PrimitiveValues {
        a_u8: u8,
        a_u16: u16,
        a_u32: u32,
        a_u64: u64,
        an_i8: i8,
        an_i16: i16,
        an_i32: i32,
        an_i64: i64,
        a_bool: bool,
        an_f32: f32,
        an_f64: f64,
        a_char: char,
    }

    #[test]
    fn test_primitive_values() {
        let mut headers = Headers::new();
        headers.set_raw("a_u8", "8");
        headers.set_raw("a_u16", "16");
        headers.set_raw("a_u32", "32");
        headers.set_raw("a_u64", "64");
        headers.set_raw("an_i8", "18");
        headers.set_raw("an_i16", "116");
        headers.set_raw("an_i32", "132");
        headers.set_raw("an_i64", "164");
        headers.set_raw("a_bool", "true");
        headers.set_raw("an_f32", "3.14159265359");
        headers.set_raw("an_f64", "2.71828182846");
        headers.set_raw("a_char", "\u{39e}");

        let attrs = deserialize::<PrimitiveValues>(&headers).unwrap();

        assert_eq!(attrs.a_u8, 8);
        assert_eq!(attrs.a_u16, 16);
        assert_eq!(attrs.a_u32, 32);
        assert_eq!(attrs.a_u64, 64);
        assert_eq!(attrs.an_i8, 18);
        assert_eq!(attrs.an_i16, 116);
        assert_eq!(attrs.an_i32, 132);
        assert_eq!(attrs.an_i64, 164);
        assert_eq!(attrs.a_bool, true);
        assert_eq!(attrs.an_f32, 3.14159265359);
        assert_eq!(attrs.an_f64, 2.71828182846);
        assert_eq!(attrs.a_char, '\u{39e}');
    }

    #[derive(Deserialize)]
    struct SingleAttributeBytes {
        #[serde(with = "serde_bytes", rename = "auEduPersonSharedToken")]
        shared_token: Vec<u8>,
    }

    #[test]
    fn test_bytes() {
        let value = b"BuyTkNadqZW_wYOeY4ppThkRRYE";

        let mut headers = Headers::new();
        headers.set_raw(
            "auEduPersonSharedToken",
            String::from_utf8(value.to_vec()).unwrap(),
        );

        let attrs = deserialize::<SingleAttributeBytes>(&headers).unwrap();
        assert_eq!(&attrs.shared_token[..], value);
    }
}
