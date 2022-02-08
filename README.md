# dtn7esp

## This fork aims to run `dtn7-rs` on an ESP32

### Introduction

To run `dtn7-rs` on an ESP32 some functionality and especially dependencies had to be removed. This project does not
provide a general purpose DTN implementation as `dtn7-rs`, but instead only provides a simple broadcast node, which
sends every received bundle to all known peers.

In the end, I was not able to run and test this project on an ESP32 because of the limited RAM.


### Usage

While the `esp` branch hosts the version that successfully compiles for the ESP32, the `master` branch contains a
version which compiles for an x86 system. This version still uses `hyper`, `axum` and in some parts `tokio` and provides
a working broadcast node.

Every bundle send to `dtn://broadcaster/in` will be forwarded to all known peers.


### License

All the changes I have made are visible in the git history. These contributions follow the license model defined
by `dtn7-rs`:

Licensed under either of <a href="LICENSE-APACHE">Apache License, Version 2.0</a> or <a href="LICENSE-MIT">MIT
license</a> at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `dtn7-rs`/`dtn7esp` by
you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
