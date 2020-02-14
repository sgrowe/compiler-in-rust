use compiler_core::code_gen::*;
use compiler_core::parser::{parse, ParseError};
use compiler_core::wasm::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

fn compile(source: &str) -> Result<String, ParseError> {
    let ast = parse(&source)?;

    let wasm = ast_to_wasm(&ast);

    let mut output = vec![];

    wasm.write_text(&mut output, WasmFormat::default()).unwrap();

    Ok(std::str::from_utf8(&output).unwrap().to_owned())
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    get_document()
        .get_element_by_id("loading")
        .expect("should have #loading on the page")
        .dyn_ref::<web_sys::HtmlElement>()
        .expect("#loading should be an `HtmlElement`")
        .style()
        .set_property("display", "none")?;

    let on_input = Closure::wrap(Box::new(move |_event: web_sys::InputEvent| {
        update_output().unwrap();
    }) as Box<dyn FnMut(_)>);

    get_input_el().add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())?;

    on_input.forget();

    update_output()?;

    Ok(())
}

fn update_output() -> Result<(), JsValue> {
    let input = get_input_value();

    let error_el = get_document()
        .get_element_by_id("error")
        .expect("should have #error on the page")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("#error should be an `HtmlElement`");

    match compile(&input) {
        Ok(compiled) => {
            set_output_text(&compiled);
            error_el.style().set_property("display", "none")?;
        }
        Err(_err) => {
            set_output_text("");
            error_el.style().set_property("display", "block")?;
        }
    }

    Ok(())
}

fn get_document() -> web_sys::Document {
    let window = web_sys::window().expect("no global `window` exists");
    window.document().expect("should have a document on window")
}

fn get_input_el() -> web_sys::HtmlTextAreaElement {
    get_document()
        .get_element_by_id("input")
        .expect("should have #input on the page")
        .dyn_into::<web_sys::HtmlTextAreaElement>()
        .expect("#input should be an `HtmlTextAreaElement`")
}

fn get_input_value() -> String {
    get_input_el().value()
}

fn set_output_text(text: &str) {
    get_document()
        .get_element_by_id("output")
        .expect("should have #output on the page")
        .dyn_ref::<web_sys::HtmlElement>()
        .expect("#output should be an `HtmlElement`")
        .set_inner_text(text);
}
