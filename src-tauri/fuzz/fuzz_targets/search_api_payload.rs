#![no_main]

use assistsupport_lib::commands::search_api::{HybridSearchResponse, SearchApiStatsData};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(parsed) = serde_json::from_slice::<HybridSearchResponse>(data) {
        let _ = serde_json::to_string(&parsed);
    }

    if let Ok(stats) = serde_json::from_slice::<SearchApiStatsData>(data) {
        let _ = serde_json::to_string(&stats);
    }
});
