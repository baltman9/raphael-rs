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

// ----- Force a TLS section to exist in the final wasm when threads are enabled -----
#[cfg(all(target_arch = "wasm32", feature = "wasm_threads"))]
#[link_section = ".tdata"]          // place in TLS data segment
#[thread_local]                      // mark as TLS so rustc emits TLS machinery
#[used]                              // prevent LTO/GC from stripping it
static __RAPHAEL_TLS_PIN__: u8 = 0;

// small helper that can be referenced from bin or anywhere to ensure the TLS
// symbol is actually read (keeps it obviously “live” to the optimizer).
#[cfg(all(target_arch = "wasm32", feature = "wasm_threads"))]
#[inline(always)]
pub fn touch_tls_anchor() {
    // SAFETY: benign volatile read from a TLS byte
    unsafe {
        core::ptr::read_volatile(core::ptr::addr_of!(__RAPHAEL_TLS_PIN__));
    }
}
