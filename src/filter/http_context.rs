use std::convert::TryFrom;

use proxy_wasm::traits::{Context, HttpContext};
use proxy_wasm::types::Action;

use crate::config::Configuration;
use crate::thislog::IdentLogger;

#[derive(Debug)]
pub struct HttpLimitSize {
    pub context_id: u32,
    pub log_id: String,
    pub configuration: Configuration,
    pub acc_size: u64,
    pub bailed: bool,
}

impl IdentLogger for HttpLimitSize {
    fn ident(&self) -> &str {
        self.log_id.as_str()
    }
}

impl Context for HttpLimitSize {}

impl HttpContext for HttpLimitSize {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
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
        if let Some(action) = self.limit_arbitrary_body(
            self.max_request_size(),
            body_size,
            Self::bail_request_payload_too_large,
        ) {
            return action;
        }

        Action::Continue
    }

    fn on_http_response_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        self.acc_size = 0; // reset

        if self.bailed {
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
        if self.bailed {
            return Action::Continue;
        }

        if let Some(action) = self.limit_arbitrary_body(
            self.max_response_size(),
            body_size,
            Self::bail_response_payload_too_large,
        ) {
            return action;
        }

        Action::Continue
    }
}

impl HttpLimitSize {
    #[inline(always)]
    fn limit_content_length(
        &mut self,
        max_size: Option<u64>,
        header_fn: fn(&Self, &str) -> Option<String>,
        bail_fn: fn(&mut Self) -> Action,
    ) -> Option<Action> {
        use std::str::FromStr;

        let max_size = max_size?;
        let cl = header_fn(self, "Content-Length")?;
        let length = u64::from_str(cl.as_str()).ok()?;
        if length > max_size {
            return Some(bail_fn(self));
        }

        None
    }

    #[inline(always)]
    fn limit_arbitrary_body(
        &mut self,
        max_size: Option<u64>,
        body_size: usize,
        bail_fn: fn(&mut Self) -> Action,
    ) -> Option<Action> {
        self.acc_size += u64::try_from(body_size).ok()?;

        let max_size = max_size?;
        if self.acc_size > max_size {
            return Some(bail_fn(self));
        }

        None
    }

    #[inline(always)]
    fn bail_request_payload_too_large(&mut self) -> Action {
        self.send_http_response(413, vec![], Some(b"Payload Too Large"));
        self.bailed = true;
        Action::Pause
    }

    #[inline(always)]
    fn bail_response_payload_too_large(&mut self) -> Action {
        self.send_http_response(502, vec![], Some(b"Bad Gateway: Payload Too Large"));
        self.bailed = true;
        Action::Pause
    }

    #[inline(always)]
    fn max_request_size(&self) -> Option<u64> {
        self.configuration.max_request_size
    }

    #[inline(always)]
    fn max_response_size(&self) -> Option<u64> {
        self.configuration.max_response_size
    }
}
