use std::collections::HashMap;

use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde_json::Value;

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_context_id| -> Box<dyn RootContext> {
        Box::new(AddHeaderRootContext::new())
    });
}

#[derive(Debug)]
struct AddHeaderRootContext {
    root_headers_map: HashMap<String, String>,
}

impl AddHeaderRootContext {
    fn new() -> Self {
        return Self {
            root_headers_map: Default::default(),
        };
    }
}

impl Context for AddHeaderRootContext {}

impl RootContext for AddHeaderRootContext {
    fn on_configure(&mut self, _: usize) -> bool {
        if let Some(config_bytes) = self.get_configuration() {
            let config: Value = serde_json::from_slice(config_bytes.as_slice()).unwrap();
            for (key, value) in config.as_object().unwrap().iter() {
                self.root_headers_map
                    .insert(key.to_owned(), String::from(value.as_str().unwrap()));
            }

            info!("Got configuration: {:?}", self.root_headers_map);
        }

        true
    }

    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(AddHeader {
            headers_map: self.root_headers_map.clone(),
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

#[derive(Debug)]
struct AddHeader {
    headers_map: HashMap<String, String>,
}

impl Context for AddHeader {}

impl HttpContext for AddHeader {
    fn on_http_response_headers(&mut self, _num_headers: usize) -> Action {
        // 默认返回一个 WA-Demo: true 的头
        self.set_http_response_header("WA-Demo", Some("true"));

        for (k, v) in self.headers_map.iter() {
            self.set_http_response_header(k, Some(v));
        }

        Action::Continue
    }
}
