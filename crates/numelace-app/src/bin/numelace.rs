//! Numelace desktop application using egui/eframe.
//!
//! This is the main entry point for the desktop Numelace application.

use numelace_app::NumelaceApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    const APP_ID: &str = "io.github.gifnksm.numelace";

    better_panic::install();
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_app_id(APP_ID)
            .with_resizable(true)
            .with_inner_size((800.0, 600.0))
            .with_min_inner_size((400.0, 300.0))
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!(
                    "../../../../assets/icon-256.png"
                ))
                .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "Numelace",
        options,
        Box::new(|cc| Ok(Box::new(NumelaceApp::new(cc)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn install_panic_alert_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        previous(panic_info);

        let message = panic_info.to_string();
        if let Some(window) = web_sys::window() {
            let _ = window.alert_with_message(&format!(
                "Numelace has crashed.\n\n{message}\n\nClearing cache and reloading may fix the issue.\n\nSee the developer console for details."
            ));
        }
    }));
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    install_panic_alert_hook();

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    log::info!(
        "Starting Numelace WASM application, version={}",
        numelace_app::version::build_version()
    );

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
                Box::new(|cc| Ok(Box::new(NumelaceApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(()) => {
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
