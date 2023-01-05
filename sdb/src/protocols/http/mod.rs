#[cfg(target_family="wasm")]
mod http_wasm;
#[cfg(target_family="wasm")]
pub use http_wasm::*;

#[cfg(not(target_family="wasm"))]
mod http_rest;
#[cfg(not(target_family="wasm"))]
pub use http_rest::*;