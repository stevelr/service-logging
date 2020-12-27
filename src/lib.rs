#![deny(missing_docs)]
//! Library for aggregating logs and sending to logging service.
//! Contains implementations for [Coralogix](https://coralogix.com/)
//! and (for wasm) console.log
mod logging;
mod time;

#[cfg(any(doc, target_arch = "wasm32"))]
pub use logging::ConsoleLogger;
pub use logging::{
    CoralogixConfig, CoralogixLogger, LogEntry, LogLevel, LogQueue, Logger, Severity,
};

/// The `log!` macro can be used to create structured log entries for later use by [Logger.send](Logger::send)
///
/// ```
/// use service_logging::{log, Logger, LogQueue, Severity};
/// let mut lq = LogQueue::default();
/// let url = "https://example.com";
///
/// log!(lq, Severity::Info,
///     method: "GET",
///     url: url,
///     status: 200
/// );
/// ```
///
/// Parameters are of the form: (queue, severity, key:value, key:value, ...).
/// `queue` is any object that implements `fn add(&mut self, e: [LogEntry])`
/// (such as [LogQueue] or [Context](https://docs.rs/wasm-service/0.2/wasm_service/struct.Context.html))
///
/// Values can be anything that implements [ToString]
/// Key names must use the same syntax as a rust identifier, e.g., no spaces, punctuation, etc.
///
/// The following keys are "special" (known to Coralogix and used for categorization
/// in the coralogix dashboard):  `text`, `category`, `class_name`, `method_name`, `thread_id`
/// If `text` is not defined, all non-coralogix keys are converted into a json string and
/// passed as the value of 'text'. (If `text` is also defined, any non-coralogix keys will be
/// silently dropped).
#[macro_export]
macro_rules! log {
    ( $queue:expr,  $sev:expr,  $( $key:tt $_t:tt  $val:expr ),* ) => {{
        let mut fields: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
        let mut has_text = false;
        let mut entry = service_logging::LogEntry { severity: ($sev), ..service_logging::LogEntry::default() };
        $(
            let val = $val.to_string();
            let key = stringify!($key);
            match key {
                "text" => { entry.text = val; has_text = true; },
                "category" => { entry.category = Some(val); },
                "class_name" => { entry.class_name = Some(val); },
                "method_name" => { entry.method_name = Some(val); },
                "thread_id" => { entry.thread_id = Some(val); },
                _ => { fields.insert(key.to_string(), val); }
            }
        )*
        if !has_text {
            entry.text = match serde_json::to_string(&fields) {
                Ok(s) => s,
                Err(e) => format!("error serializing message: {}",e),
            };
        }
        $queue.log(entry);
    }};
}
