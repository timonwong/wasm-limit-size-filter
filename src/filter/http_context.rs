use proxy_wasm::traits::{Context, HttpContext};
use proxy_wasm::types::Action;

use crate::config::Configuration;
use crate::thislog::IdentLogger;

#[derive(Debug)]
pub struct HttpLimitSize {
    pub context_id: u32,
    pub log_id: String,
    pub configuration: Configuration,
    pub acc_req_size: usize,
    pub acc_resp_size: usize,
    pub bailed_out: bool,
}

impl IdentLogger for HttpLimitSize {
    fn ident(&self) -> &str {
        self.log_id.as_str()
    }
}

impl Context for HttpLimitSize {}

impl HttpContext for HttpLimitSize {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        debug!(
            self,
            "on_http_request_headers, _num_headers={:?}, _end_of_stream={:?}",
            _num_headers,
            _end_of_stream
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
            self,
            "on_http_request_body, body_size={:?}, end_of_stream={:?}", body_size, _end_of_stream
        );

        self.acc_req_size += body_size;
        if self.acc_req_size > self.max_request_size() {
            return self.bail_request_payload_too_large();
        }

        Action::Continue
    }

    fn on_http_response_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        debug!(
            self,
            "on_http_response_headers: _num_headers={:?} _end_of_stream={:?}",
            _num_headers,
            _end_of_stream
        );

        if self.bailed_out {
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
            self,
            "on_http_response_body, body_size={:?}, end_of_stream={:?}", body_size, _end_of_stream
        );

        if self.bailed_out {
            return Action::Continue;
        }

        self.acc_resp_size += body_size;
        if self.acc_resp_size > self.max_response_size() {
            return self.bail_response_payload_too_large();
        }

        Action::Continue
    }
}

impl HttpLimitSize {
    #[inline(always)]
    fn limit_content_length(
        &mut self,
        max_size: usize,
        header_fn: fn(&Self, &str) -> Option<String>,
        bail_fn: fn(&mut Self) -> Action,
    ) -> Option<Action> {
        use std::convert::TryFrom;
        use std::str::FromStr;

        let cl = header_fn(self, "Content-Length")?;
        debug!(self, "Got content length: {:?}", cl);
        let length = i64::from_str(cl.as_str()).ok()?;
        let max_size = i64::try_from(max_size).ok()?;
        if length > max_size {
            return Some(bail_fn(self));
        }

        None
    }

    #[inline(always)]
    fn bail_request_payload_too_large(&mut self) -> Action {
        self.send_http_response(413, vec![], Some(b"Payload Too Large"));
        self.bailed_out = true;
        Action::Pause
    }

    #[inline(always)]
    fn bail_response_payload_too_large(&mut self) -> Action {
        self.send_http_response(502, vec![], Some(b"Bad Gateway: Payload Too Large"));
        self.bailed_out = true;
        Action::Pause
    }

    #[inline(always)]
    fn max_request_size(&self) -> usize {
        self.configuration.max_request_size
    }

    #[inline(always)]
    fn max_response_size(&self) -> usize {
        self.configuration.max_response_size
    }
}
