use std::convert::TryFrom;
use std::str::FromStr;

use log::{debug, error, info};
use proxy_wasm::traits::{Context, HttpContext, RootContext};
use proxy_wasm::types::{Action, ContextType, LogLevel};

mod config;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Debug);
    proxy_wasm::set_root_context(|_context_id| -> Box<dyn RootContext> {
        Box::new(LimitSizeRootContext::new())
    });
}}

#[derive(Debug)]
struct LimitSizeRootContext {
    root_config: config::Config,
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
            acc_req_size: 0,
            acc_resp_size: 0,
            bailed_out: false,
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

#[derive(Debug)]
struct LimitSize {
    config: config::Config,
    acc_req_size: usize,
    acc_resp_size: usize,
    bailed_out: bool,
}

impl Context for LimitSize {}

impl HttpContext for LimitSize {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        debug!(
            "on_http_request_headers, _num_headers={:?}, _end_of_stream={:?}",
            _num_headers, _end_of_stream
        );

        // 优先使用 Content-Length
        if let Some(action) = self.limit_content_length(
            self.max_request_size(),
            Self::get_http_request_header,
            Self::bail_request_payload_too_large,
        ) {
            return action;
        }

        Action::Continue
    }

    fn on_http_request_body(&mut self, body_size: usize, _end_of_stream: bool) -> Action {
        debug!(
            "on_http_request_body, body_size={:?}, end_of_stream={:?}",
            body_size, _end_of_stream
        );

        self.acc_req_size += body_size;
        if self.acc_req_size > self.max_request_size() {
            return self.bail_request_payload_too_large();
        }

        Action::Continue
    }

    fn on_http_response_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        debug!(
            "on_http_response_headers: _num_headers={:?} _end_of_stream={:?}",
            _num_headers, _end_of_stream
        );

        if self.bailed_out {
            info!("on_http_response_headers:: bailed!");
            return Action::Continue;
        }

        // 优先使用 Content-Length
        if let Some(action) = self.limit_content_length(
            self.max_response_size(),
            Self::get_http_response_header,
            Self::bail_response_payload_too_large,
        ) {
            return action;
        }

        Action::Continue
    }

    fn on_http_response_body(&mut self, body_size: usize, _end_of_stream: bool) -> Action {
        debug!(
            "on_http_response_body, body_size={:?}, end_of_stream={:?}",
            body_size, _end_of_stream
        );

        if self.bailed_out {
            info!("on_http_response_body:: bailed!");
            return Action::Continue;
        }

        self.acc_resp_size += body_size;
        if self.acc_resp_size > self.max_response_size() {
            return self.bail_response_payload_too_large();
        }

        Action::Continue
    }
}

impl LimitSize {
    fn limit_content_length(
        &mut self,
        max_size: usize,
        header_fn: fn(&Self, &str) -> Option<String>,
        bail_fn: fn(&mut Self) -> Action,
    ) -> Option<Action> {
        let cl = header_fn(self, "Content-Length")?;
        debug!("Got content length: {:?}", cl);
        let length = i64::from_str(cl.as_str()).ok()?;
        let max_size = i64::try_from(max_size).ok()?;
        if length > max_size {
            return Some(bail_fn(self));
        }

        None
    }

    fn bail_request_payload_too_large(&mut self) -> Action {
        self.send_http_response(413, vec![], Some(b"Payload Too Large"));
        self.bailed_out = true;
        Action::Pause
    }

    fn bail_response_payload_too_large(&mut self) -> Action {
        self.send_http_response(502, vec![], Some(b"Bad Gateway: Payload Too Large"));
        self.bailed_out = true;
        Action::Pause
    }

    fn max_request_size(&self) -> usize {
        self.config.max_request_size
    }

    fn max_response_size(&self) -> usize {
        self.config.max_response_size
    }
}
