#![cfg(not(feature = "ffi"))]

extern crate console_error_panic_hook;

use crate::gameboy::Gameboy;
use crate::input::KeypadKey;

use core::cell::{RefCell, RefMut};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::{CanvasRenderingContext2d, ImageData};

use std::panic;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

/// Activate the emulator with a RunLicense license. Verifies the license,
/// then mounts the full Game Boy UI into the target element.
/// If `target_id` is `None`, defaults to `"game"`.
#[wasm_bindgen]
pub fn activate(license_json: &str, target_id: Option<String>) {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let license_json = license_json.to_string();
    let target_id = target_id.clone();
    wasm_bindgen_futures::spawn_local(async move {
        log("[gameboy] Validating license...");
        let result = runlicense_sdk_webassembly_rust::verify_license!(&license_json).await;

        match result {
            Ok(_token) => {
                log("[gameboy] License activated successfully");
                if let Err(e) = mount_inner(target_id) {
                    log(&format!("[gameboy] Failed to mount UI: {:?}", e));
                }
            }
            Err(e) => {
                log(&format!("[gameboy] License verification failed: {:?}", e));
            }
        }
    });
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

const GAMEBOY_CSS: &str = r#"
:root {
 --pink: #F38BA3;
 --blue: #F38BA3;
 --stroke: navy;
 --green: #C2E688;
 --red: #8bf3db;
 --light-brow: #8bf3db;
 --gray: #8bf3db;
 --fonts-regular:-apple-system, BlinkMacSystemFont, "Segoe UI", "Roboto", "Oxygen", "Ubuntu", "Cantarell", "Fira Sans", "Droid Sans", "Helvetica Neue", sans-serif;
}
.gb-wrapper { font-family: var(--fonts-regular); display: flex; flex-direction: column; align-items: center; }
.gb-wrapper * { box-sizing: border-box; }
.gb-wrapper button { outline:0 !important; cursor: pointer; }
.gb-wrapper button:focus { box-shadow: inset 0 0 !important; }
.gb-wrapper button:disabled { opacity: 0.5; cursor: default; }
.gb-menu { text-align: center; margin-bottom: 16px; }
.gb-menu label, .gb-menu strong { color: inherit; }
.gb-menu .btn { text-decoration: none; font-weight: thin; color: #FFF; padding: 5px 10px; border-radius: 5px; background: #F38BA3; cursor: pointer; text-shadow: none; border: none; }
.gb-container { display: flex; align-items: center; justify-content: center; }
.gb-body {
 background-color: var(--blue);
 width: 320px; height: 590px;
 border-radius: 20px 20px 50px;
 border: 10px solid var(--stroke);
 box-shadow: inset 10px 0 rgba(255,255,255,0.7), inset -10px 0 rgba(0,0,0,0.2);
 position: relative;
}
.gb-body header { width: 100%; height: 36px; border-bottom: 6px solid var(--stroke); position: relative; box-shadow: 0 6px rgba(0,0,0,0.2); }
.gb-body header:after, .gb-body header:before { content: ""; height: 100%; width: 6px; background-color: var(--stroke); position: absolute; }
.gb-body header:before { left: 30px; }
.gb-body header:after { right: 30px; }
.gb-screen {
 width: 246px; background-color: var(--light-brow);
 margin: 30px 0 0 30px; border: 6px solid var(--stroke);
 padding: 30px; border-radius: 10px 10px 30px; position: relative;
}
.gb-screen .glass {
 width: 100%; height: 160px; background-color: var(--green);
 border: 6px solid var(--stroke); box-shadow: inset 6px 6px rgba(0,0,0,0.2);
 overflow: hidden; position: relative;
}
.gb-screen .glass:after, .gb-screen .glass:before { content: ""; height: 200%; width: 30px; background-color: rgba(255,255,255,0.4); position: absolute; transform: rotate(45deg); top: -80px; }
.gb-screen .glass:after { right: -20px; width: 50px !important; }
.gb-screen > span { width: 10px; height: 10px; background-color: var(--red); position: absolute; left: 10px; border-radius: 50%; border: 2px solid var(--stroke); top: 50%; margin-top: -10px; }
.gb-screen:after, .gb-screen:before { content:""; height: 3px; position: absolute; top: 12px; background-color: var(--stroke); }
.gb-screen:before { width: 100px; left: 10px; }
.gb-screen:after { width: 30px; right: 10px; }
.gb-actions { padding: 30px 14px 0; }
.gb-directions { width: 120px; height: 120px; border: 6px solid transparent; position: relative; display: flex; float: left; }
.gb-directions button { border: 6px solid var(--stroke); background-color: var(--light-brow); width: 30px; height: 30px; position: absolute; z-index: 9; padding: 0; border-radius: 0; }
.gb-directions .arrow-left { border-right: none !important; box-shadow: inset 0 4px #fff, inset 0 -4px rgba(0,0,0,0.2); top: 50%; margin-top: -15px; left: 13.5px; }
.gb-directions .arrow-top { border-bottom: none !important; box-shadow: inset 0 4px #fff; left: 50%; margin-left: -15px; top: 13.5px; }
.gb-directions .arrow-right { border-left: none !important; box-shadow: inset 0 4px #fff, inset 0 -4px rgba(0,0,0,0.2); top: 50%; margin-top: -15px; right: 13.5px; }
.gb-directions .arrow-bottom { border-top: none !important; box-shadow: inset 0 -4px rgba(0,0,0,0.2); left: 50%; margin-left: -15px; bottom: 13.5px; }
.gb-directions:after { content: ""; width: 30px; height: 30px; background-color: var(--light-brow); position: relative; top: 50%; left: 50%; margin: -15px; z-index: 1; }
.gb-buttons { border: 6px solid transparent; float: right; position: relative; transform: rotate(-20deg); top: 30px; }
.gb-buttons button { background-color: var(--red); border: 6px solid var(--stroke); border-radius: 50%; width: 50px; height: 50px; padding: 0; box-shadow: inset 4px 0 rgba(255,255,255,0.7), inset -4px 0 rgba(0,0,0,0.2); margin: 0 5px; }
.gb-start-reset { list-style: none; text-align: center; width: 100%; float: left; padding: 0; }
.gb-start-reset li { display: inline-block; margin: 0 15px; }
.gb-start-reset li button { border: 6px solid var(--stroke); background-color: var(--light-brow); width: 20px; height: 50px; padding: 0; border-radius: 10px; box-shadow: inset 3px 0 #fff, 3px 0 rgba(0,0,0,0.2); transform: rotate(60deg); position: relative; left: -25px; top: -10px; }
.gb-points { text-align: center; position: absolute; right: 20px; bottom: 20px; }
.gb-points span { font-size: 25px; line-height: 0px; letter-spacing: 3px; margin-top: -3px; display: block; color: var(--stroke); }
.gb-helper { margin-top: 12px; font-size: 12px; color: #666; text-align: center; }
"#;

const GAMEBOY_HTML: &str = r#"
<div class="gb-wrapper">
  <div class="gb-menu">
    <p>
      <strong><label for="gb-file">Select a ROM file to play</label></strong><br/>
      <input type="file" id="gb-file" accept=".gb,.gbc" />
    </p>
    <p>
      <button id="gb-play" disabled class="btn">Load Game</button>
    </p>
  </div>
  <div class="gb-container">
    <main class="gb-body">
      <header></header>
      <section class="gb-screen">
        <span></span>
        <div class="glass" id="game"></div>
      </section>
      <section class="gb-actions">
        <div class="gb-directions">
          <button class="arrow-left"></button>
          <button class="arrow-top"></button>
          <button class="arrow-right"></button>
          <button class="arrow-bottom"></button>
        </div>
        <div class="gb-buttons">
          <button class="button-a"></button>
          <button class="button-b"></button>
        </div>
        <ul class="gb-start-reset">
          <li><button class="start"></button></li>
          <li><button class="reset"></button></li>
        </ul>
        <div class="gb-points">
          <span>&bull;&bull;&bull;&bull;&bull;</span><br/>
          <span>&bull;&bull;&bull;&bull;&bull;&bull;&bull;</span><br/>
          <span>&bull;&bull;&bull;&bull;&bull;</span><br/>
          <span>&bull;&bull;&bull;&bull;&bull;&bull;</span><br/>
          <span>&bull;&bull;&bull;&bull;</span>
        </div>
      </section>
    </main>
  </div>
  <div class="gb-helper">
    <span>&#x2190;&#x2191;&#x2192;&#x2193; Move</span> &middot;
    <span>A: A</span> &middot; <span>S: B</span> &middot;
    <span>Z: Select</span> &middot; <span>X: Start</span>
  </div>
</div>
"#;

/// Inject the CSS into the document via a <style> element appended to the container.
fn inject_css(document: &web_sys::Document, container: &web_sys::Element) -> Result<(), JsValue> {
    let style = document.create_element("style")?;
    style.set_inner_html(GAMEBOY_CSS);
    container.prepend_with_node_1(&style)?;
    Ok(())
}

/// Start the emulation render loop for a loaded ROM.
fn start_emulation(
    rom: Vec<u8>,
    document: &web_sys::Document,
) -> Result<(), JsValue> {
    let game = document
        .get_element_by_id("game")
        .ok_or_else(|| JsValue::from_str("No #game element found"))?;

    let canvas = document
        .create_element("canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    canvas.set_attribute("style", "width:100%;height:100%;image-rendering:pixelated;")?;
    game.set_inner_html("");
    game.append_child(&canvas)?;
    canvas.set_width(160);
    canvas.set_height(144);

    let context = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("Could not get 2d context"))?
        .dyn_into::<CanvasRenderingContext2d>()?;

    let mut gb = Gameboy::new(rom, None);

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    let current_key_code: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
    gb.frame();

    {
        let current_key_code = current_key_code.clone();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let key: RefMut<_> = current_key_code.borrow_mut();

            match *key {
                65 => gb.keydown(KeypadKey::A),
                -65 => gb.keyup(KeypadKey::A),
                83 => gb.keydown(KeypadKey::B),
                -83 => gb.keyup(KeypadKey::B),
                90 => gb.keydown(KeypadKey::Select),
                -90 => gb.keyup(KeypadKey::Select),
                88 => gb.keydown(KeypadKey::Start),
                -88 => gb.keyup(KeypadKey::Start),
                37 => gb.keydown(KeypadKey::Left),
                -37 => gb.keyup(KeypadKey::Left),
                39 => gb.keydown(KeypadKey::Right),
                -39 => gb.keyup(KeypadKey::Right),
                38 => gb.keydown(KeypadKey::Up),
                -38 => gb.keyup(KeypadKey::Up),
                40 => gb.keydown(KeypadKey::Down),
                -40 => gb.keyup(KeypadKey::Down),
                _ => (),
            }

            gb.frame();
            let data: &mut [u8] = gb.image_mut();
            if let Ok(d) = ImageData::new_with_u8_clamped_array_and_sh(
                wasm_bindgen::Clamped(data),
                160,
                144,
            ) {
                context.put_image_data(&d, 0.0, 0.0).ok();
            }

            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));
    }

    // Keyboard: keydown
    {
        let current_key_code = current_key_code.clone();
        let closure =
            Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
                *current_key_code.borrow_mut() = event.key_code() as i32;
            });
        window().add_event_listener_with_callback(
            "keydown",
            closure.as_ref().unchecked_ref(),
        )?;
        closure.forget();
    }

    // Keyboard: keyup
    {
        let closure =
            Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
                *current_key_code.borrow_mut() = (event.key_code() as i32) * -1;
            });
        window().add_event_listener_with_callback(
            "keyup",
            closure.as_ref().unchecked_ref(),
        )?;
        closure.forget();
    }

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

/// Inner mount logic shared by `mount` and `activate`.
fn mount_inner(target_id: Option<String>) -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let container_id = target_id.as_deref().unwrap_or("game");
    let container = document
        .get_element_by_id(container_id)
        .ok_or_else(|| JsValue::from_str(&format!("Element '{}' not found", container_id)))?;

    container.set_inner_html(GAMEBOY_HTML);
    inject_css(&document, &container)?;

    let rom: Rc<RefCell<Option<Vec<u8>>>> = Rc::new(RefCell::new(None));

    // File input: read ROM bytes when a file is selected
    let file_input = document
        .get_element_by_id("gb-file")
        .ok_or_else(|| JsValue::from_str("No #gb-file element"))?
        .dyn_into::<web_sys::HtmlInputElement>()?;

    let play_button = document
        .get_element_by_id("gb-play")
        .ok_or_else(|| JsValue::from_str("No #gb-play element"))?
        .dyn_into::<web_sys::HtmlButtonElement>()?;

    // On file change: read the selected ROM
    {
        let rom = rom.clone();
        let play_button = play_button.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::Event| {
            let input: web_sys::HtmlInputElement = event
                .target()
                .unwrap()
                .dyn_into()
                .unwrap();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let name = file.name();
                    play_button.set_disabled(false);
                    play_button.set_inner_html(&format!("Play {}", name));

                    let reader = web_sys::FileReader::new().unwrap();
                    let reader_clone = reader.clone();
                    let rom = rom.clone();
                    let onload = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::Event| {
                        if let Ok(result) = reader_clone.result() {
                            let array = js_sys::Uint8Array::new(&result);
                            let mut bytes = vec![0u8; array.length() as usize];
                            array.copy_to(&mut bytes);
                            *rom.borrow_mut() = Some(bytes);
                        }
                    });
                    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                    onload.forget();
                    reader.read_as_array_buffer(&file).unwrap();
                }
            }
        });
        file_input.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // On play click: start emulation with the loaded ROM
    {
        let closure = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::Event| {
            let rom_data = rom.borrow_mut().take();
            if let Some(data) = rom_data {
                let document = web_sys::window().unwrap().document().unwrap();
                let btn = document.get_element_by_id("gb-play").unwrap()
                    .dyn_into::<web_sys::HtmlButtonElement>().unwrap();
                btn.set_disabled(true);

                if let Err(e) = start_emulation(data, &document) {
                    web_sys::console::error_1(&e);
                }
            }
        });
        play_button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    Ok(())
}

/// Build the full Game Boy UI inside the target element and wire up file
/// input + play button. No license check — use `activate` for licensed access.
/// If `target_id` is `None`, defaults to `"game"`.
#[wasm_bindgen]
pub fn mount(target_id: Option<String>) -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    mount_inner(target_id)
}

/// Direct render with a ROM already in memory. Injects UI into target element
/// and immediately starts emulation.
#[wasm_bindgen]
pub async fn render(rom: Vec<u8>, target_id: Option<String>) -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let document = web_sys::window().unwrap().document().unwrap();
    let container_id = target_id.as_deref().unwrap_or("game");
    let container = document
        .get_element_by_id(container_id)
        .ok_or_else(|| JsValue::from_str(&format!("Element '{}' not found", container_id)))?;

    container.set_inner_html(GAMEBOY_HTML);
    inject_css(&document, &container)?;

    // Hide file picker since ROM is already provided
    if let Some(menu) = document.query_selector(".gb-menu").unwrap() {
        menu.dyn_into::<web_sys::HtmlElement>()
            .unwrap()
            .style()
            .set_property("display", "none")?;
    }

    start_emulation(rom, &document)
}
