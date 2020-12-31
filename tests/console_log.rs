wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
// using ConsoleLogger to write to javascript/browser console.log

use service_logging::{log, ConsoleLogger, LogQueue, Severity};

use wasm_bindgen_test::*;

#[wasm_bindgen_test]
async fn test_console_log() {
    let mut log_queue = LogQueue::default();
    log!(log_queue, Severity::Info, one:"Thing One", two: "Thing Two");
    let logger = ConsoleLogger::init();
    logger
        .send("test_console_log", log_queue.take())
        .await
        .expect("send");
}
