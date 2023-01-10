#[cfg(all(feature = "ws", target_family="wasm"))]
mod ws_wasm;
#[cfg(all(feature = "ws", target_family="wasm"))]
pub use ws_wasm::*;

#[cfg(all(feature = "ws", not(target_family="wasm")))]
mod ws_rest;
#[cfg(all(feature = "ws", not(target_family="wasm")))]
pub use ws_rest::*;
