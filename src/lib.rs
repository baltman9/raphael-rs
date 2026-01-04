// Enable TLS feature only when building threaded WASM
#![cfg_attr(all(target_arch = "wasm32", feature = "wasm_threads"), feature(thread_local))]

// Threads ON: re-export real initializer
#[cfg(all(target_arch = "wasm32", feature = "wasm_threads"))]
pub use wasm_bindgen_rayon::init_thread_pool;

// Threads OFF: provide a no-op Promise
#[cfg(all(target_arch = "wasm32", not(feature = "wasm_threads")))]
pub fn init_thread_pool(_num_threads: usize) -> js_sys::Promise {
    js_sys::Promise::resolve(&wasm_bindgen::JsValue::UNDEFINED)
}

mod app;
pub use app::MacroSolverApp;

mod config;
mod context;
pub mod thread_pool;
mod widgets;

#[cfg(target_arch = "wasm32")]
pub static OOM_PANIC_OCCURED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

// ---------- TLS anchor to force __wasm_init_tls export (threads build) ----------
#[cfg(all(target_arch = "wasm32", feature = "wasm_threads"))]
#[thread_local]
static TLS_ANCHOR: u8 = 0;

/// Public helper to ensure the TLS section is retained so the final WASM
/// exports `__wasm_init_tls` (required by wasm-bindgen for threads).
#[cfg(all(target_arch = "wasm32", feature = "wasm_threads"))]
#[inline(never)]
pub fn ensure_wasm_tls() {
    // Volatile read prevents DCE of TLS data / symbol.
    unsafe { core::ptr::read_volatile(&TLS_ANCHOR) };
}

// No-op on other targets so callers don't need cfg-gating.
#[cfg(not(all(target_arch = "wasm32", feature = "wasm_threads")))]
#[inline(always)]
pub fn ensure_wasm_tls() {}
