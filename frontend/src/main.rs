use leptos::{wasm_bindgen::prelude::Closure, *};
use tauri_glue::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, MouseEvent, TouchEvent};

#[tauri_glue::bind_command(name = log)]
pub async fn log(log: &str);

macro_rules! console_log {
    ($($t:tt)*) => (spawn_local(async move { log(&format_args!($($t)*).to_string()).await }))
}

#[component]
fn Canvas(cx: Scope) -> Element {
    let is_painting = create_rw_signal(cx, false);
    let line_width = create_rw_signal(cx, 1);
    let line_color = create_rw_signal(cx, "#000000".to_string());
    let (start_x, start_y) = (create_rw_signal(cx, 0), create_rw_signal(cx, 0));

    let canvas = {
        let canvas = create_element("canvas");
        canvas.set_id("drawing-board");

        let canv_clone = canvas
            .clone()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        canv_clone.set_height(window().inner_height().unwrap().as_f64().unwrap() as u32);
        canv_clone.set_width(window().inner_width().unwrap().as_f64().unwrap() as u32);

        canvas
    };

    let update_canvas_size = move || {
        let canv = document()
            .get_element_by_id("drawing-board")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let (new_width, new_height) = (
            window().inner_width().unwrap().as_f64().unwrap() as u32 - canv.offset_left() as u32,
            window().inner_height().unwrap().as_f64().unwrap() as u32 - canv.offset_top() as u32,
        );

        if canv.width() != new_width {
            canv.set_width(new_width);
        }
        if canv.height() != new_height {
            canv.set_height(new_height);
        }
    };

    {
        let onresize = Closure::wrap(Box::new(move || update_canvas_size()) as Box<dyn FnMut()>);
        window().set_onresize(Some(onresize.as_ref().unchecked_ref()));
        onresize.forget();
    }

    add_event_listener(&canvas, "touchend", move |_e: TouchEvent| {
        is_painting.set(false);

        let canvas_ctx = document()
            .get_element_by_id("drawing-board")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap()
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
        canvas_ctx.stroke();
        canvas_ctx.begin_path();
    });

    add_event_listener(&canvas, "touchstart", move |e: TouchEvent| {
        update_canvas_size();

        let canv = document()
            .get_element_by_id("drawing-board")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        is_painting.set(true);
        start_x.set(e.page_x() - canv.offset_width() as i32);
        start_y.set(e.page_y() - canv.offset_height() as i32);
    });

    add_event_listener(&canvas, "touchmove", move |e: TouchEvent| {
        if !is_painting.get() {
            return;
        }

        let canv = document()
            .get_element_by_id("drawing-board")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let (x_offset, y_offset) = (canv.offset_left(), canv.offset_top());

        let canvas_ctx = canv
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        canvas_ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str(&line_color.get()));
        canvas_ctx.set_line_width(line_width.get() as f64);
        canvas_ctx.set_line_cap("round");
        canvas_ctx.line_to(
            e.page_x() as f64 - x_offset as f64,
            e.page_y() as f64 - y_offset as f64,
        );

        canvas_ctx.stroke();
    });

    view! { cx,
        <section class="container">
            <div id="toolbar">
                <label for="stroke">"Color"</label>
                <input id="stroke" name="stroke" type="color" on:change={move |e: Event| {
                    line_color.set(event_target_value(&e));
                }} />
                <label for="lineWidth">"Size"</label>
                <input id="lineWidth" name="lineWidth" type="range" min="1" max="20" prop:value=line_width.get() on:input=move |e| {
                    line_width.set(event_target_value(&e).parse::<u32>().unwrap())
                }/>
                <button id="clear" on:click={move |_| {
                    let canvas = document()
                        .get_element_by_id("drawing-board")
                        .unwrap()
                        .dyn_into::<web_sys::HtmlCanvasElement>()
                        .map_err(|_| ())
                        .unwrap();

                    canvas
                        .get_context("2d")
                        .unwrap()
                        .unwrap()
                        .dyn_into::<web_sys::CanvasRenderingContext2d>()
                        .unwrap()
                        .clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
                }}>"Clear"</button>
            </div>
            <div class="drawing-board">
                {canvas}
            </div>
        </section>
    }
}

pub fn main() {
    console_log!("page loaded");
    console_error_panic_hook::set_once();
    mount_to_body(|cx| {
        view! { cx,
            <Canvas />
        }
    })
}
