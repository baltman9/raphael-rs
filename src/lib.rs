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
mod thread_pool;
mod widgets;

#[cfg(target_arch = "wasm32")]
pub static OOM_PANIC_OCCURED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
