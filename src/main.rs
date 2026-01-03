// Prevents a console from being opened on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(target_arch = "wasm32", feature(alloc_error_hook))]
#![cfg_attr(all(target_arch = "wasm32", feature = "wasm_threads"), feature(thread_local))]

// Only compile TLS anchor when the `wasm_threads` feature is enabled
#[cfg(all(target_arch = "wasm32", feature = "wasm_threads"))]
#[thread_local]
static TLS_ANCHOR: u8 = 0;

#[cfg(all(target_arch = "wasm32", feature = "wasm_threads"))]
async fn init_wasm_threads() {
    // Pick a sane worker count from the browser if available
    let workers = web_sys::window()
        .and_then(|w| Some(w.navigator().hardware_concurrency()))
        .unwrap_or(4)
        .max(2) as usize;

    // Touch TLS so rustc/linker keep TLS sections (__wasm_init_tls)
    unsafe {
        let p: *const u8 = core::ptr::addr_of!(TLS_ANCHOR);
        core::ptr::read_volatile(p);
    }

    // Start the Rayon worker pool for wasm
    wasm_bindgen_rayon::init_thread_pool(workers)
        .await
        .expect("init wasm thread pool");
}

// No-op in single-thread builds (keeps code paths cleanly compiled out)
#[cfg(not(all(target_arch = "wasm32", feature = "wasm_threads")))]
async fn init_wasm_threads() {}

#[cfg(all(target_os = "windows", not(debug_assertions)))]
fn init_logging() {
    let mut file_path = eframe::storage_dir("Raphael XIV").unwrap();
    if !std::fs::exists(&file_path).unwrap() {
        std::fs::create_dir_all(&file_path).unwrap();
    }
    file_path.push("log.txt");
    let log_file_target = Box::new(std::fs::File::create(file_path).unwrap());
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .target(env_logger::Target::Pipe(log_file_target))
        .init();
    std::panic::set_hook(Box::new(|info| {
        log::error!("{}", info);
    }));
}

#[cfg(target_arch = "wasm32")]
fn init_logging() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
}

#[cfg(not(any(
    all(target_os = "windows", not(debug_assertions)),
    target_arch = "wasm32"
)))]
fn init_logging() {
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    init_logging();

    let present_mode = std::env::var("RAPHAEL_PRESENT_MODE").map_or(
        eframe::wgpu::PresentMode::default(),
        |env_var| match env_var.as_str() {
            "AutoVsync" => eframe::wgpu::PresentMode::AutoVsync,
            "AutoNoVsync" => eframe::wgpu::PresentMode::AutoNoVsync,
            "Fifo" => eframe::wgpu::PresentMode::Fifo,
            "FifoRelaxed" => eframe::wgpu::PresentMode::FifoRelaxed,
            "Immediate" => eframe::wgpu::PresentMode::Immediate,
            "Mailbox" => eframe::wgpu::PresentMode::Mailbox,
            _ => panic!("Unknown present mode: {}", env_var),
        },
    );

    let desired_maximum_frame_latency = std::env::var("RAPHAEL_DESIRED_MAXIMUM_FRAME_LATENCY")
        .ok()
        .and_then(|env_var| env_var.parse::<u32>().ok());

    let wgpu_options = eframe::egui_wgpu::WgpuConfiguration {
        present_mode,
        desired_maximum_frame_latency,
        ..Default::default()
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        wgpu_options,
        ..Default::default()
    };
    eframe::run_native(
        "Raphael XIV",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(raphael_xiv::MacroSolverApp::new(cc)))
        }),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    fn custom_alloc_error_hook(_layout: std::alloc::Layout) {
        raphael_xiv::OOM_PANIC_OCCURED.store(true, std::sync::atomic::Ordering::Relaxed);
        eframe::wasm_bindgen::throw_val("OOM panic".into());
    }
    std::alloc::set_alloc_error_hook(custom_alloc_error_hook);

    init_logging();

    fn get_canvas() -> Option<web_sys::HtmlCanvasElement> {
        use web_sys::wasm_bindgen::JsCast;
        let document = web_sys::window()?.document()?;
        let canvas = document.get_element_by_id("the_canvas_id")?;
        canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok()
    }

    fn remove_loading_spinner() -> Option<()> {
        let document = web_sys::window()?.document()?;
        let spinner = document.get_element_by_id("spinner")?;
        spinner.remove();
        Some(())
    }

    wasm_bindgen_futures::spawn_local(async {
        // Initializes thread pool when the feature is ON; no-op otherwise
        init_wasm_threads().await;

        let start_result = eframe::WebRunner::new()
            .start(
                get_canvas().unwrap(),
                eframe::WebOptions::default(),
                Box::new(|cc| {
                    egui_extras::install_image_loaders(&cc.egui_ctx);
                    Ok(Box::new(raphael_xiv::MacroSolverApp::new(cc)))
                }),
            )
            .await;
        remove_loading_spinner();
        if let Err(error) = start_result {
            eframe::wasm_bindgen::throw_val(error);
        }
    });
}
