use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
	std::panic::set_hook(Box::new(console_error_panic_hook::hook));
	console_log::init_with_level(log::Level::Debug).expect("failed to initialize logger");

	Ok(())
}

#[wasm_bindgen]
pub fn greet(name: &str) {
	log::info!("Hello, {}!", name);
}
