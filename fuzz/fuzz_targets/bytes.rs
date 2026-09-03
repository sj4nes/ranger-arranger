// libFuzzer target: feeds arbitrary bytes to the encoder/decoder/deserializer.
// Mirrors `fuzz_bytes` in `src/fuzz_api.rs` (also exercised by `cargo test`).
// Run: cargo +nightly fuzz run bytes
#![no_main]
use libfuzzer_sys::fuzz_target;
use vsql_ranger_arranger::fuzz_api::fuzz_bytes;

fuzz_target!(|data: &[u8]| {
    fuzz_bytes(data);
});
