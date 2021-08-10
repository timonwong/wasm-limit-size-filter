use std::collections::HashMap;

use log::error;
use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use std::cell::RefCell;
use std::rc::Rc;

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_context_id| -> Box<dyn RootContext> {
        Box::new(AddHeaderRootContext::new())
    });
}

#[derive(Debug)]
struct AddHeaderRootContext {
    root_headers_map: Rc<RefCell<HashMap<String, String>>>,
}

impl AddHeaderRootContext {
    fn new() -> Self {
        Self {
            root_headers_map: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl Context for AddHeaderRootContext {}

impl RootContext for AddHeaderRootContext {
    fn on_configure(&mut self, _: usize) -> bool {
        let mut root_headers_map = self.root_headers_map.borrow_mut();
        if let Some(config_bytes) = self.get_configuration() {
            let v: serde_json::Result<HashMap<String, String>> =
                serde_json::from_slice(config_bytes.as_slice());
            match v {
                Ok(config) => {
                    for (key, value) in config.iter() {
                        root_headers_map.insert(key.to_owned(), String::from(value));
                    }

                    info!("Got configuration: {:?}", root_headers_map);
                }
                Err(err) => {
                    error!("Unable to parse JSON: {:?}", err);
                }
            };
        }

        true
    }

    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(AddHeader {
            headers_map: Rc::clone(&self.root_headers_map),
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

#[derive(Debug)]
struct AddHeader {
    headers_map: Rc<RefCell<HashMap<String, String>>>,
}

impl Context for AddHeader {}

impl HttpContext for AddHeader {
    fn on_http_response_headers(&mut self, _num_headers: usize) -> Action {
        // 默认返回一个 WA-Demo: true 的头
        self.set_http_response_header("WA-Demo", Some("true"));
        self.set_http_response_header("X-Powered-By", Some("add-header-ts"));

        // 自定义 Header
        let headers_map = self.headers_map.borrow();
        for (k, v) in headers_map.iter() {
            self.set_http_response_header(k, Some(v));
        }

        Action::Continue
    }
}
