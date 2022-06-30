use core::convert::TryFrom;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    #[serde(default = "Configuration::default_max_size")]
    pub max_request_size: usize,

    #[serde(default = "Configuration::default_max_size")]
    pub max_response_size: usize,

    #[serde(default)]
    pub status_codes: StatusCodes,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatusCodes {
    #[serde(default = "StatusCodes::default_request")]
    pub request: u32,

    #[serde(default = "StatusCodes::default_response")]
    pub response: u32,
}

impl Default for StatusCodes {
    fn default() -> Self {
        Self::default()
    }
}

impl StatusCodes {
    #[inline]
    pub const fn default() -> Self {
        Self {
            request: Self::default_request(),
            response: Self::default_response(),
        }
    }

    const fn default_request() -> u32 {
        413
    }

    const fn default_response() -> u32 {
        502
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self::default()
    }
}

impl Configuration {
    const DEFAULT_MAX_SIZE: usize = 500 * 1024; // 500KB

    #[inline]
    pub const fn default() -> Self {
        Self {
            max_request_size: Self::default_max_size(),
            max_response_size: Self::default_max_size(),
            status_codes: StatusCodes::default(),
        }
    }

    #[inline]
    const fn default_max_size() -> usize {
        Self::DEFAULT_MAX_SIZE
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
            "maxRequestSize": 55555,
            "statusCodes": {
                "response": 500
            }
        }"#;
    }

    #[test]
    fn it_parses_a_configuration_string() {
        let conf: Configuration =
            show_parsed_deserialization(fixtures::CONFIG1, serde_json::from_str);
        assert_eq!(conf.max_request_size, 55555);
        assert_eq!(conf.max_response_size, 666666);
        assert_eq!(conf.status_codes.request, StatusCodes::default_request());
        assert_eq!(conf.status_codes.response, StatusCodes::default_response());

        let conf: Configuration =
            show_parsed_deserialization(fixtures::CONFIG2, serde_json::from_str);
        assert_eq!(conf.max_request_size, 55555);
        assert_eq!(conf.max_response_size, Configuration::default_max_size());
        assert_eq!(conf.status_codes.request, StatusCodes::default_request());
        assert_eq!(conf.status_codes.response, 500);
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
