#![no_main]

use assistsupport_lib::validation::{
    validate_https_url, validate_ticket_id, validate_url, MAX_QUERY_BYTES,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let selector = data[0] % 4;
    let payload = String::from_utf8_lossy(&data[1..]);

    match selector {
        0 => {
            let _ = validate_ticket_id(&payload);
        }
        1 => {
            let _ = validate_url(&payload);
        }
        2 => {
            let _ = validate_https_url(&payload);
        }
        _ => {
            // Simulate query-size enforcement boundary checks used by search surfaces.
            let _ = payload.len() <= MAX_QUERY_BYTES;
        }
    }
});
