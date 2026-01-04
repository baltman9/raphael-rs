use std::num::NonZeroUsize;

static THREAD_POOL_INIT: std::sync::Once = std::sync::Once::new();

#[cfg(target_arch = "wasm32")]
static THREAD_POOL_IS_INITIALIZED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Try to initialize the global pool once. Safe to call many times.
pub fn attempt_initialization(num_threads: Option<NonZeroUsize>) {
    THREAD_POOL_INIT.call_once(|| {
        initialize(num_threads);
    });
}

pub fn initialization_attempted() -> bool {
    THREAD_POOL_INIT.is_completed()
}

/* --------------------------- native (non-wasm) --------------------------- */

#[cfg(not(target_arch = "wasm32"))]
fn initialize(num_threads: Option<NonZeroUsize>) {
    let num_threads = num_threads.unwrap_or_else(default_thread_count);

    match rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads.get())
        .build_global()
    {
        Ok(()) => log::debug!(
            "Created global thread pool with num_threads = {}",
            num_threads
        ),
        Err(error) => log::debug!(
            "Creation of global thread pool failed with error = {:?}",
            error
        ),
    }
}

/* ------------------------------- wasm32 --------------------------------- */

#[cfg(target_arch = "wasm32")]
fn initialize(num_threads: Option<NonZeroUsize>) {
    let num_threads = num_threads.unwrap_or_else(default_thread_count);

    // Our `init_thread_pool` (exposed from lib.rs on wasm) returns a js_sys::Promise.
    // Wrap it so we can `.await` it and log the result.
    let promise = crate::init_thread_pool(num_threads.get());
    let future = wasm_bindgen_futures::JsFuture::from(promise);

    wasm_bindgen_futures::spawn_local(async move {
        let result = future.await;
        log::debug!(
            "Initialized Pool with num_threads = {}, result = {:?}",
            num_threads,
            result
        );
        THREAD_POOL_IS_INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed);
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn default_thread_count() -> NonZeroUsize {
    // Use half the available cores, clamped to at least 2.
    std::thread::available_parallelism()
        .map(|detected| {
            let n = std::cmp::max(2, detected.get() / 2);
            NonZeroUsize::new(n).unwrap()
        })
        .unwrap_or_else(|| NonZeroUsize::new(4).unwrap())
}

#[cfg(target_arch = "wasm32")]
pub fn default_thread_count() -> NonZeroUsize {
    // navigator.hardwareConcurrency is u32; convert to usize and clamp.
    let detected = web_sys::window()
        .map(|w| w.navigator().hardware_concurrency() as usize)
        .unwrap_or(4);

    // Keep it reasonable on browsers: at least 2, at most 8.
    NonZeroUsize::new(detected.saturating_div(2).clamp(2, 8)).unwrap()
}

/* ------------------------------ status ---------------------------------- */

#[cfg(not(target_arch = "wasm32"))]
pub fn is_initialized() -> bool {
    THREAD_POOL_INIT.is_completed()
}

#[cfg(target_arch = "wasm32")]
pub fn is_initialized() -> bool {
    THREAD_POOL_IS_INITIALIZED.load(std::sync::atomic::Ordering::Relaxed)
}
