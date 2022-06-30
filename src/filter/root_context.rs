use proxy_wasm::traits::{Context, HttpContext, RootContext};
use proxy_wasm::types::{BufferType, ContextType};

use crate::config::Configuration;
use crate::log::IdentLogger;
use crate::util::serde::ErrorLocation;

use super::http_context::HttpLimitSize;

#[derive(Debug)]
pub(super) struct RootLimitSize {
    context_id: u32,
    log_id: String,
    vm_configuration: Option<Vec<u8>>,
    configuration: Configuration,
}

impl RootLimitSize {
    pub fn new(context_id: u32) -> Self {
        Self {
            context_id,
            log_id: format!("({}/root)", context_id),
            vm_configuration: None,
            configuration: Configuration::default(),
        }
    }
}

impl IdentLogger for RootLimitSize {
    fn ident(&self) -> &str {
        self.log_id.as_str()
    }
}

impl Context for RootLimitSize {}

impl RootContext for RootLimitSize {
    fn on_vm_start(&mut self, vm_configuration_size: usize) -> bool {
        info!(
            self,
            "{} version {} booting up.",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
        );
        info!(
            self,
            "on_vm_start: vm_configuration_size is {}", vm_configuration_size
        );

        let vm_config = proxy_wasm::hostcalls::get_buffer(
            BufferType::VmConfiguration,
            0,
            vm_configuration_size,
        );
        if let Err(e) = vm_config {
            error!(
                self,
                "on_vm_start: error retrieving VM configuration: {:#?}", e
            );
            return false;
        }

        self.vm_configuration = vm_config.unwrap();

        if let Some(conf) = self.vm_configuration.as_ref() {
            info!(
                self,
                "on_vm_start: VM configuration is {}",
                core::str::from_utf8(conf).unwrap()
            );
        } else {
            // We currently don't need a VM config, so don't
            // fail if there's none.
            warn!(self, "on_vm_start: empty VM config");
        }

        true
    }

    fn on_configure(&mut self, plugin_configuration_size: usize) -> bool {
        use core::convert::TryFrom;

        info!(
            self,
            "on_configure: plugin_configuration_size is {}", plugin_configuration_size
        );

        let conf = match proxy_wasm::hostcalls::get_buffer(
            BufferType::PluginConfiguration,
            0,
            plugin_configuration_size,
        ) {
            Ok(Some(conf)) => conf,
            Ok(None) => {
                warn!(self, "empty module configuration - module has no effect");
                return true;
            }
            Err(e) => {
                error!(self, "error retrieving module configuration: {:#?}", e);
                return false;
            }
        };

        debug!(self, "loaded raw config");

        let conf = match Configuration::try_from(conf.as_slice()) {
            Ok(conf) => conf,
            Err(e) => {
                if let Ok(el) = ErrorLocation::try_from(&e) {
                    let conf_str = String::from_utf8_lossy(conf.as_slice());
                    for line in el.error_lines(conf_str.as_ref(), 4, 4) {
                        error!(self, "{}", line);
                    }
                } else {
                    // not a configuration syntax/data error (ie. programmatic)
                    error!(self, "fatal configuration error: {:#?}", e);
                }
                return false;
            }
        };

        self.configuration = conf;
        info!(
            self,
            "on_configure: plugin configuration {:#?}", self.configuration
        );

        true
    }

    fn create_http_context(&self, context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(HttpLimitSize {
            context_id,
            log_id: format!("({}/http {}/root)", context_id, self.context_id),
            configuration: self.configuration,
            acc_req_size: 0,
            acc_resp_size: 0,
            bailed_out: false,
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}
