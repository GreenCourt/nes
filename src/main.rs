pub mod bus;
pub mod cartridge;
pub mod controller;
pub mod cpu;
pub mod nes;
pub mod opcode;
pub mod ppu;

mod app;

#[cfg(target_arch = "wasm32")]
fn main() {
    {
        #[wasm_bindgen::prelude::wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_name = eval)]
            fn js_eval(s: &str);
        }
        js_eval("Error.stackTraceLimit = 50;");
    }

    use eframe::wasm_bindgen::JsCast as _;
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

        let _start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(app::NesApp::new(cc)))),
            )
            .await;
    });
}
