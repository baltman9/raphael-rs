// When building for wasm32 with threads enabled, re-export the real init.
#[cfg(all(target_arch = "wasm32", feature = "wasm_threads"))]
pub use wasm_bindgen_rayon::init_thread_pool;

// When building for wasm32 WITHOUT threads, provide a no-op Promise so call sites
// (that expect a Promise) still compile and run.
#[cfg(all(target_arch = "wasm32", not(feature = "wasm_threads")))]
pub fn init_thread_pool(_pool_size: Option<usize>) -> js_sys::Promise {
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
