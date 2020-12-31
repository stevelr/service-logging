// Example use of ConsoleLogger in non-wasm32 builds.
//
#[cfg(not(target_arch = "wasm32"))]
use service_logging::{log, ConsoleLogger, LogQueue, Severity};

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_console_log() {
    let mut log_queue = LogQueue::default();
    log!(log_queue, Severity::Info, one:"Thing One", two: "Thing Two");
    let logger = ConsoleLogger::init();
    logger
        .send("test_console_log", log_queue.take())
        .await
        .expect("send");
}
