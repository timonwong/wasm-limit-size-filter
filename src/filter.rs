mod http_context;
mod root_context;

proxy_wasm::main! {{
    use proxy_wasm::types::LogLevel;
    use proxy_wasm::traits::RootContext;

    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|context_id| -> Box<dyn RootContext> {
        Box::new(root_context::RootLimitSize::new(context_id))
    });
}}
