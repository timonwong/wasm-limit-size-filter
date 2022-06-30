use core::convert::TryFrom;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    #[serde(default = "Configuration::default_max_size")]
    pub max_request_size: Option<u64>,

    #[serde(default = "Configuration::default_max_size")]
    pub max_response_size: Option<u64>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self::default()
    }
}

impl Configuration {
    #[inline]
    pub const fn default() -> Self {
        Self {
            max_request_size: Self::default_max_size(),
            max_response_size: Self::default_max_size(),
        }
    }

    #[inline]
    const fn default_max_size() -> Option<u64> {
        None
    }
}

impl TryFrom<&[u8]> for Configuration {
    type Error = serde_json::Error;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice(buf)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::serde::ErrorLocation;
    use std::convert::TryInto;

    fn show_parsed_deserialization<'de, T, E, F>(input: &'de str, deser_fn: F) -> T
    where
        F: FnOnce(&'de str) -> Result<T, E>,
        T: std::fmt::Debug + serde::Deserialize<'de>,
        E: std::error::Error + std::fmt::Display,
        for<'e> &'e E: TryInto<ErrorLocation<'e, E>>,
    {
        let res = deser_fn(input);
        if let Err(ref e) = res {
            if let Ok(el) = e.try_into() {
                eprintln!("{}", el.error_to_string(input));
            } else {
                eprintln!("{}", e);
            }
        }
        assert!(res.is_ok());
        let parsed = res.unwrap();
        eprintln!("PARSED:\n{:#?}", parsed);
        parsed
    }

    fn get_config() -> Configuration {
        Configuration::default()
    }

    mod fixtures {
        pub const CONFIG1: &str = r#"{
            "maxRequestSize": 55555,
            "maxResponseSize": 666666
        }"#;

        pub const CONFIG2: &str = r#"{
            "maxRequestSize": 55555
        }"#;
    }

    #[test]
    fn it_parses_a_configuration_string() {
        let conf: Configuration =
            show_parsed_deserialization(fixtures::CONFIG1, serde_json::from_str);
        assert_eq!(conf.max_request_size.unwrap(), 55555);
        assert_eq!(conf.max_response_size.unwrap(), 666666);

        let conf: Configuration =
            show_parsed_deserialization(fixtures::CONFIG2, serde_json::from_str);
        assert_eq!(conf.max_request_size.unwrap(), 55555);
        assert_eq!(conf.max_response_size, Configuration::default_max_size());
    }

    #[test]
    fn print_config() {
        let config = get_config();
        let str = serde_json::to_string(&config);
        match &str {
            Err(e) => eprintln!("Failed to serialize configuration: {:#?}", e),
            Ok(s) => println!("{}", s),
        }
        assert!(str.is_ok());
        let s = str.unwrap();

        let _: Configuration = show_parsed_deserialization(&s, serde_json::from_str);
    }
}
