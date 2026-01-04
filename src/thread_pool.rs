use std::num::NonZeroUsize;

static THREAD_POOL_INIT: std::sync::Once = std::sync::Once::new();

#[cfg(target_arch = "wasm32")]
static THREAD_POOL_IS_INITIALIZED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub fn attempt_initialization(num_threads: Option<NonZeroUsize>) {
    THREAD_POOL_INIT.call_once(|| {
        initialize(num_threads);
    });
}

pub fn initialization_attempted() -> bool {
    THREAD_POOL_INIT.is_completed()
}

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

#[cfg(target_arch = "wasm32")]
fn initialize(num_threads: Option<NonZeroUsize>) {
    let num_threads = num_threads.unwrap_or_else(default_thread_count);

    // Start the Rayon pool for wasm (returns a Promise)
    let promise = wasm_bindgen_rayon::init_thread_pool(num_threads.get());
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
    // Use `map_or_else` so both branches are closures (fixes arity warning)
    std::thread::available_parallelism().map_or_else(
        || NonZeroUsize::new(4).unwrap(),
        |detected| {
            let num_threads = std::cmp::max(2, detected.get() / 2);
            NonZeroUsize::new(num_threads).unwrap()
        },
    )
}

#[cfg(target_arch = "wasm32")]
pub fn default_thread_count() -> NonZeroUsize {
    // Navigator reports u32; clamp a sane range and ensure nonzero
    let detected = web_sys::window()
        .map(|w| w.navigator().hardware_concurrency())
        .unwrap_or(4);

    let n = ((detected as usize) / 2).clamp(2, 8);
    NonZeroUsize::new(n).unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn is_initialized() -> bool {
    THREAD_POOL_INIT.is_completed()
}

#[cfg(target_arch = "wasm32")]
pub fn is_initialized() -> bool {
    THREAD_POOL_IS_INITIALIZED.load(std::sync::atomic::Ordering::Relaxed)
}
