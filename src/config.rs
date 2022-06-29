use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "Config::default_max_size")]
    pub max_request_size: usize,

    #[serde(default = "Config::default_max_size")]
    pub max_response_size: usize,
}

impl Config {
    const DEFAULT_MAX_SIZE: usize = 500 * 1024; // 500KB

    pub fn default() -> Self {
        Self {
            max_request_size: Self::default_max_size(),
            max_response_size: Self::default_max_size(),
        }
    }

    fn default_max_size() -> usize {
        return Self::DEFAULT_MAX_SIZE;
    }
}
