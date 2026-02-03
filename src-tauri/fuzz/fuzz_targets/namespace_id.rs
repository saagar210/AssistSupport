#![no_main]

use assistsupport_lib::validation::{
    normalize_and_validate_namespace_id, normalize_namespace_id, validate_namespace_id,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);
    let normalized = normalize_namespace_id(&input);
    if !normalized.is_empty() {
        let _ = validate_namespace_id(&normalized);
    }
    let _ = normalize_and_validate_namespace_id(&input);
});
