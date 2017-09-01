mod deserialize_headers;
mod deserialize_values;

use std::{error, fmt};
use std::error::Error;
use serde::de::{self, Deserialize};
use hyper::Headers;

#[derive(Debug)]
pub enum HeadersDeserializationError {
}

impl error::Error for HeadersDeserializationError {
    fn description(&self) -> &str {
        "unable to deserialize from HTTP headers"
    }
}

impl de::Error for HeadersDeserializationError {
    fn custom<T>(t: T) -> Self {
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

fn deserialize<T>(headers: &Headers) -> Result<T, HeadersDeserializationError>
where
    for<'de> T: Deserialize<'de>,
{
    let mut deserializer = deserialize_headers::DeserializeHeaders::new(headers.iter());
    T::deserialize(deserializer)
}

#[cfg(test)]
mod tests {
    use super::*;

    use hyper::{Method, Headers};

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
}
