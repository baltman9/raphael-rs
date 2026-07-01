// Enable thread_local only for wasm+threads builds (nightly is already used in CI)
#![cfg_attr(all(target_arch = "wasm32", feature = "wasm_threads"), feature(thread_local))]

// Threads ON: re-export real initializer
#[cfg(all(target_arch = "wasm32", feature = "wasm_threads"))]
pub use wasm_bindgen_rayon::init_thread_pool;

// Threads OFF: provide a no-op Promise for callers that still invoke it on web
#[cfg(all(target_arch = "wasm32", not(feature = "wasm_threads")))]
pub fn init_thread_pool(_num_threads: usize) -> js_sys::Promise {
    js_sys::Promise::resolve(&wasm_bindgen::JsValue::UNDEFINED)
}

/* ───────────────────────── TLS ANCHOR (lib crate) ────────────────────────────
   This must live in the *library crate* that gets fed to wasm-bindgen so that
   rustc emits WebAssembly TLS sections and the `__wasm_init_tls` symbol.
   We also export a tiny function that touches the symbol to keep it alive.
*/
#[cfg(all(target_arch = "wasm32", feature = "wasm_threads"))]
#[used]
#[thread_local]
static TLS_ANCHOR: u8 = 0;

#[cfg(all(target_arch = "wasm32", feature = "wasm_threads"))]
#[unsafe(no_mangle)]
#[inline(never)]
pub extern "C" fn raphael_touch_tls_anchor() {
    unsafe {
        let p: *const u8 = core::ptr::addr_of!(TLS_ANCHOR);
        core::ptr::read_volatile(p);
    }
}

/// Safe wrapper so binaries can call this without `extern "C"`.
#[cfg(all(target_arch = "wasm32", feature = "wasm_threads"))]
pub fn touch_tls_anchor() {
    // Call the exported symbol as well, which helps defeat LTO over-eagerness.
    raphael_touch_tls_anchor();
}

// Modules & public API
mod app;
pub use app::MacroSolverApp;

mod config;
mod context;
mod elements;
mod fonts;
mod solve;
pub mod thread_pool;

#[cfg(not(target_arch = "wasm32"))]
mod update;

#[cfg(target_arch = "wasm32")]
pub static OOM_PANIC_OCCURED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
