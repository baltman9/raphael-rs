// Export the real init when on wasm32 *and* threads feature is enabled.
#[cfg(all(target_arch = "wasm32", feature = "wasm_threads"))]
pub use wasm_bindgen_rayon::init_thread_pool;

// When on wasm32 but WITHOUT threads, provide a no-op Promise so call sites don't change.
#[cfg(all(target_arch = "wasm32", not(feature = "wasm_threads")))]
pub fn init_thread_pool(_pool_size: Option<usize>) -> js_sys::Promise {
    // Immediately-resolving Promise (matches wasm_bindgen_rayon signature)
    js_sys::Promise::resolve(&wasm_bindgen::JsValue::UNDEFINED)
}

mod app;
pub use app::MacroSolverApp;

mod config;
mod context;
mod thread_pool;
mod widgets;

#[cfg(target_arch = "wasm32")]
pub static OOM_PANIC_OCCURED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
