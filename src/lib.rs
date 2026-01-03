// Export `init_thread_pool` only when we're on wasm32 *and* the `wasm_threads`
// feature is enabled.
#[cfg(all(target_arch = "wasm32", feature = "wasm_threads"))]
pub use wasm_bindgen_rayon::init_thread_pool;

// When building for wasm32 without threads, provide a no-op with the same
// signature so call sites can remain unconditional.
#[cfg(all(target_arch = "wasm32", not(feature = "wasm_threads")))]
pub async fn init_thread_pool(_pool_size: Option<usize>) -> Result<(), wasm_bindgen::JsValue> {
    Ok(())
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
