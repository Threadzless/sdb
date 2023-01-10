#[cfg(all(feature = "http", target_family="wasm"))]
mod http_wasm;
#[cfg(all(feature = "http", target_family="wasm"))]
pub use http_wasm::*;

#[cfg(all(feature = "http", not(target_family="wasm")))]
mod http_rest;
#[cfg(all(feature = "http", not(target_family="wasm")))]
pub use http_rest::*;