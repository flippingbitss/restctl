#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    use log::LevelFilter;

    let mut builder = env_logger::Builder::from_default_env(); // Log to stderr (if you run with `RUST_LOG=debug`).
    builder.filter_level(LevelFilter::Debug);
    builder.init();

    let tokio_runtime = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    let _ = tokio_runtime.enter();
    let tokio_runtime_handle = tokio_runtime.handle().clone();

    let async_runtime_handle =
        restctl::async_runtime::AsyncRuntimeHandle::new_native(tokio_runtime_handle);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "restctl",
        native_options,
        Box::new(|cc| {
            restctl::customize_app_styles(cc);
            // cc.egui_ctx.set_theme(egui::Theme::Light);
            Ok(Box::new(restctl::App::new(cc, async_runtime_handle)))
        }),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    use restctl::async_runtime;

                    restctl::customize_app_styles(cc);
                    // cc.egui_ctx.set_theme(egui::Theme::Light);
                    Ok(Box::new(restctl::App::new(
                        cc,
                        async_runtime::AsyncRuntimeHandle::new_web(),
                    )))
                }),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
