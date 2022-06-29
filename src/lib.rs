use std::convert::TryFrom;
use std::str::FromStr;

use log::error;
use log::info;
use proxy_wasm::traits::{Context, HttpContext, RootContext};
use proxy_wasm::types::{Action, ContextType, LogLevel};

mod config;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_context_id| -> Box<dyn RootContext> {
        Box::new(LimitSizeRootContext::new())
    });
}}

#[derive(Debug)]
struct LimitSizeRootContext {
    root_config: config::Config,
}

#[derive(Debug)]
struct LimitSize {
    config: config::Config,
    is_ct: bool,
    acc_size: usize,
}

impl LimitSizeRootContext {
    fn new() -> Self {
        Self {
            root_config: config::Config::default(),
        }
    }
}

impl Context for LimitSizeRootContext {}

impl RootContext for LimitSizeRootContext {
    fn on_configure(&mut self, _: usize) -> bool {
        if let Some(config_bytes) = self.get_plugin_configuration() {
            let v = serde_json::from_slice::<config::Config>(config_bytes.as_slice());
            match v {
                Ok(config) => {
                    info!("Got configuration: {:?}", config);
                    self.root_config = config;
                }
                Err(err) => {
                    error!("Unable to parse JSON: {:?}", err);
                }
            };
        }

        true
    }

    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(LimitSize {
            config: self.root_config,
            is_ct: false,
            acc_size: 0,
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

impl Context for LimitSize {}

impl HttpContext for LimitSize {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        // 优先使用 Content-Length
        match self.get_http_request_header("Content-Length") {
            Some(cl) => {
                let max_size: i64 = i64::try_from(self.max_request_size()).unwrap_or(i64::MAX);
                match i64::from_str(cl.as_str()) {
                    Ok(length) if length > max_size => {
                        self.send_http_response(413, vec![], Some(b"Payload Too Large"));
                        return Action::Pause;
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        match self.get_http_request_header("Transfer-Encoding") {
            Some(te) if te == "chunked" => {
                self.is_ct = true;
            }
            _ => {}
        }

        Action::Continue
    }

    fn on_http_request_body(&mut self, body_size: usize, _end_of_stream: bool) -> Action {
        if self.is_ct {
            self.acc_size += body_size;
            if self.acc_size > self.max_request_size() {
                self.send_http_response(413, vec![], Some(b"Payload Too Large"));
                return Action::Pause;
            }
        }

        Action::Continue
    }

    fn on_http_response_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        Action::Continue
    }

    fn on_http_response_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {
        Action::Continue
    }
}

impl LimitSize {
    fn max_request_size(&self) -> usize {
        self.config.max_request_size
    }

    fn max_response_size(&self) -> usize {
        self.config.max_response_size
    }
}
