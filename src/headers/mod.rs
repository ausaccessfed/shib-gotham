mod deserialize_headers;
mod deserialize_values;

use std::{error, fmt};
use std::error::Error;
use serde::de::{self, Deserialize};
use hyper::Headers;

#[derive(Debug)]
enum HeadersDeserializationError {
    InvalidTopLevelType { msg: &'static str },
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
}
