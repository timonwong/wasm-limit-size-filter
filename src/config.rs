use core::convert::TryFrom;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    #[serde(default = "Configuration::default_max_size")]
    pub max_request_size: usize,

    #[serde(default = "Configuration::default_max_size")]
    pub max_response_size: usize,
}

impl Configuration {
    const DEFAULT_MAX_SIZE: usize = 500 * 1024; // 500KB

    pub const fn default() -> Self {
        Self {
            max_request_size: Self::default_max_size(),
            max_response_size: Self::default_max_size(),
        }
    }

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
