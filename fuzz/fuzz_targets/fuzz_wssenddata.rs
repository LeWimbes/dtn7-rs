#![no_main]
use dtn7_plus::client::WsSendData;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    helpers::from_cbor_slice::<WsSendData>(&data);
});
