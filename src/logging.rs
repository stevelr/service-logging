//! Asynchronous logging
//!
use crate::time::current_time_millis;
use async_trait::async_trait;
use serde::Serialize;
use serde_repr::Serialize_repr;
use std::{fmt, rc::Rc, sync::Mutex};

/// Severity level
#[derive(Debug, Serialize_repr, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Severity {
    /// The most verbose level, aka Trace
    Debug = 1,
    /// Verbose logging
    Verbose = 2,
    /// Information level: warnings plus major events
    Info = 3,
    /// all errors and warnings, and no informational messages
    Warning = 4,
    /// errors only
    Error = 5,
    /// critical errors only
    Critical = 6,
}

/// Logging level, alias for Severity
pub type LogLevel = Severity;

impl Default for Severity {
    fn default() -> Self {
        Severity::Info
    }
}

impl std::str::FromStr for Severity {
    type Err = String;
    fn from_str(s: &str) -> Result<Severity, Self::Err> {
        match s {
            "debug" | "Debug" | "DEBUG" => Ok(Severity::Debug),
            "verbose" | "Verbose" | "VERBOSE" => Ok(Severity::Verbose),
            "info" | "Info" | "INFO" => Ok(Severity::Info),
            "warning" | "Warning" | "WARNING" => Ok(Severity::Warning),
            "error" | "Error" | "ERROR" => Ok(Severity::Error),
            "critical" | "Critical" | "CRITICAL" => Ok(Severity::Critical),
            _ => Err(format!("Invalid severity: {}", s)),
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Severity::Debug => "Debug",
                Severity::Verbose => "Verbose",
                Severity::Info => "Info",
                Severity::Warning => "Warning",
                Severity::Error => "Error",
                Severity::Critical => "Critical",
            }
        )
    }
}

/// LogEntry, usually created with the log! macro.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    /// Current timestamp, milliseconds since epoch in UTC
    pub timestamp: u64,
    /// Severity of this entry
    pub severity: Severity,
    /// Text value of this entry. When created with the log! macro, this field contains
    /// json-encoded key-value pairs, sorted by key
    pub text: String,
    /// Optional category string (application-defined)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Optional class_name (application-defined)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_name: Option<String>,
    /// Optional method_name (application-defined)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method_name: Option<String>,
    /// Optional thread_id (not used for wasm)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

impl fmt::Display for LogEntry {
    // omits some fields for brevity
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.timestamp, self.severity, self.text)
    }
}

impl Default for LogEntry {
    fn default() -> LogEntry {
        LogEntry {
            timestamp: current_time_millis(),
            severity: Severity::Debug,
            text: String::from(""),
            category: None,
            class_name: None,
            method_name: None,
            thread_id: None,
        }
    }
}

/// Log payload for Coralogix service
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CxLogMsg<'a> {
    /// api key
    pub private_key: &'a str,
    /// application name - dimension field
    pub application_name: &'a str,
    /// subsystem name - dimension field
    pub subsystem_name: &'a str,
    /// log messages
    pub log_entries: Vec<LogEntry>,
}

/// Queue of log entries to be sent to log service
#[derive(Debug)]
pub struct LogQueue {
    entries: Vec<LogEntry>,
}

impl Default for LogQueue {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl LogQueue {
    /// Constructs a new empty log queue
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns all queued items, emptying self
    pub fn take(&mut self) -> Vec<LogEntry> {
        let mut ve: Vec<LogEntry> = Vec::new();
        ve.append(&mut self.entries);
        ve
    }

    /// Returns true if there are no items to log
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Removes all log entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl AppendsLog for LogQueue {
    /// Appends a log entry to the queue
    fn log(&mut self, e: LogEntry) {
        self.entries.push(e)
    }
}

/// Can append log entries.
/// (technically, the first param of the log! macro just needs to implement this fn signature,
/// and doesn't need to impl this trait)
pub trait AppendsLog {
    /// Appends entry to log queue
    fn log(&mut self, e: LogEntry);
}

/// Can append log entries.
/// Used for objects with inner mutability
pub trait AppendsLogInnerMut {
    /// Appends entry to log queue
    fn log(&self, e: LogEntry);
}

impl AppendsLogInnerMut for Rc<Mutex<LogQueue>> {
    /// Appends log entry to deferred log queue
    /// Since the parameter is not mut, this only works with Mutex or cell types
    fn log(&self, e: LogEntry) {
        let mut queue = self.lock().unwrap();
        queue.log(e);
    }
}

impl fmt::Display for LogQueue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = String::with_capacity(256);
        for entry in self.entries.iter() {
            if !buf.is_empty() {
                buf.push('\n');
            }
            buf.push_str(&entry.to_string());
        }
        write!(f, "{}", buf)
    }
}

/// Trait for logging service that receives log messages
#[async_trait(?Send)]
pub trait Logger {
    /// Send entries to logger
    async fn send(
        &self,
        sub: &'static str,
        entries: Vec<LogEntry>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

/// Configuration parameters for Coralogix service
pub struct CoralogixConfig {
    /// API key, provided by Coralogix
    pub api_key: &'static str,
    /// Application name, included as a feature for all log messages
    pub application_name: &'static str,
    /// URL prefix for service invocation, e.g. `https://api.coralogix.con/api/v1/logs`
    pub endpoint: &'static str,
}

/// Implementation of Logger for Coralogix
pub struct CoralogixLogger {
    config: CoralogixConfig,
    client: reqwest::Client,
}
unsafe impl Send for CoralogixLogger {}

impl CoralogixLogger {
    /// Initialize logger with configuration
    pub fn init(config: CoralogixConfig) -> Result<Box<dyn Logger>, reqwest::Error> {
        use reqwest::header::{self, HeaderValue, CONNECTION, CONTENT_TYPE};
        let mut headers = header::HeaderMap::new();
        // all our requests are json. this header is recommended by Coralogix
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        // just in case this helps us drop connection more quickly
        headers.insert(CONNECTION, HeaderValue::from_static("close"));
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(Box::new(Self { config, client }))
    }
}

#[async_trait(?Send)]
impl Logger for CoralogixLogger {
    /// Send logs to Coralogix service. May return error if there was a problem sending.
    async fn send(
        &self,
        sub: &'static str,
        entries: Vec<LogEntry>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let msg = CxLogMsg {
            subsystem_name: sub,
            log_entries: entries,
            private_key: self.config.api_key,
            application_name: self.config.application_name,
        };
        let resp = self
            .client
            .post(self.config.endpoint)
            .json(&msg)
            .send()
            .await?;
        check_status(resp).await?;
        Ok(())
    }
}

/// Logger that sends all messages to console.log (browser). Only available for "wasm32" target builds
#[cfg(any(doc, target_arch = "wasm32"))]
#[derive(Default, Debug)]
pub struct ConsoleLogger {}

#[cfg(any(doc, target_arch = "wasm32"))]
impl ConsoleLogger {
    /// Initialize console logger. (Only available for "wasm32" target builds)
    pub fn init() -> Box<dyn Logger + Send> {
        Box::new(ConsoleLogger::default())
    }
}

#[cfg(any(doc, target_arch = "wasm32"))]
#[async_trait(?Send)]
impl Logger for ConsoleLogger {
    /// Sends logs to console.log handler
    async fn send(
        &self,
        sub: &'static str,
        entries: Vec<LogEntry>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for e in entries.iter() {
            let msg = format!("{} {} {} {}", e.timestamp, sub, e.severity, e.text);
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&msg));
        }
        Ok(())
    }
}

// Error handling for Coralogix
// Instead of just returning error for non-2xx status (via resp.error_for_status)
// include response body which may have additional diagnostic info
async fn check_status(resp: reqwest::Response) -> Result<(), Box<dyn std::error::Error>> {
    match resp.status().is_success() {
        true => Ok(()),
        false => {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            Err(Box::new(Error::Cx(format!(
                "Logging Error: status:{} {}",
                status, body
            ))))
        }
    }
}

#[derive(Debug)]
enum Error {
    // Error sending coralogix logs
    Cx(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::Cx(s) => s,
            }
        )
    }
}

impl std::error::Error for Error {}
