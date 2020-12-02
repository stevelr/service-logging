/// Returns current time in UTC, as integer milliseconds since EPOCH
#[cfg(target_arch = "wasm32")]
// logging api supports floating pt timestamps for fractional millis,
// but we don't need that resolution, and serde float support is bulky
pub fn current_time_millis() -> u64 {
    js_sys::Date::now() as u64
}

/// Returns current time in UTC, as integer milliseconds since EPOCH
#[cfg(not(target_arch = "wasm32"))]
pub fn current_time_millis() -> u64 {
    use std::time::SystemTime;
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_millis() as u64,
        Err(_) => 0, // panic!("SystemTime before UNIX EPOCH!"),
    }
}
