mod http_context;
mod root_context;

proxy_wasm::main! {{
    use crate::log::LogLevel;
    use proxy_wasm::traits::RootContext;

    proxy_wasm::set_log_level(LogLevel::Debug.into());
    proxy_wasm::set_root_context(|context_id| -> Box<dyn RootContext> {
        Box::new(root_context::RootLimitSize::new(context_id))
    });
}}
