use dioxus::prelude::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct MetricValue {
    timestamp: String,
    value: f64,
}

#[component]
fn MetricChart(
    id: String,
    title: String,
    data: Vec<MetricValue>,
    color: String,
    y_begin_at_zero: bool,
) -> Element {
    let chart_id = id.clone();
    let chart_data = data.clone();
    let chart_color = color.clone();
    let chart_title = title.clone();
    let y_zero = y_begin_at_zero.clone();

    let setup_chart = move |_| {
        let labels: Vec<String> = chart_data.iter().map(|d| d.timestamp.clone()).collect();
        let values: Vec<f64> = chart_data.iter().map(|d| d.value).collect();

        let datasets = vec![js_sys::Object::new()];
        let dataset = js_sys::Object::new();
        js_sys::Reflect::set(&dataset, &"label".into(), &chart_title.clone().into()).unwrap();
        let values_array = js_sys::Array::new();
        for val in values.iter() {
            values_array.push(&JsValue::from_f64(*val));
        }
        js_sys::Reflect::set(&dataset, &"data".into(), &values_array).unwrap();
        js_sys::Reflect::set(&dataset, &"borderColor".into(), &chart_color.clone().into()).unwrap();
        js_sys::Reflect::set(&dataset, &"tension".into(), &0.1.into()).unwrap();

        let data_obj = js_sys::Object::new();
        let labels_array = js_sys::Array::new();
        for label in labels.iter() {
            labels_array.push(&JsValue::from_str(label));
        }
        js_sys::Reflect::set(&data_obj, &"labels".into(), &labels_array).unwrap();

        let datasets_array = js_sys::Array::new();
        datasets_array.push(&dataset);
        js_sys::Reflect::set(&data_obj, &"datasets".into(), &datasets_array).unwrap();

        let scales = js_sys::Object::new();
        let y = js_sys::Object::new();
        js_sys::Reflect::set(&y, &"beginAtZero".into(), &y_zero.into()).unwrap();
        js_sys::Reflect::set(&scales, &"y".into(), &y).unwrap();

        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"scales".into(), &scales).unwrap();
        js_sys::Reflect::set(&options, &"responsive".into(), &true.into()).unwrap();
        js_sys::Reflect::set(&options, &"maintainAspectRatio".into(), &false.into()).unwrap();

        let config = js_sys::Object::new();
        js_sys::Reflect::set(&config, &"type".into(), &"line".into()).unwrap();
        js_sys::Reflect::set(&config, &"data".into(), &data_obj).unwrap();
        js_sys::Reflect::set(&config, &"options".into(), &options).unwrap();

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id(&chart_id).unwrap();
        let ctx = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap()
            .get_context("2d")
            .unwrap()
            .unwrap();

        let chart_constructor = js_sys::Reflect::get(&window, &JsValue::from_str("Chart")).unwrap();
        let _chart = js_sys::Reflect::construct(
            chart_constructor.as_ref().unchecked_ref(),
            &js_sys::Array::of2(&ctx, &config),
        )
        .unwrap();

        dioxus_logger::tracing::info!("Chart created for {}", chart_id);
    };

    rsx! {
        div {
            style: "width: 100%; height: 300px; background: #222; border-radius: 8px; padding: 15px; box-shadow: 0 2px 4px rgba(0,0,0,0.2);",
            h3 {
                style: "color: #ccc; margin: 0 0 10px 0; font-size: 1.1rem; text-align: center;",
                "{title}"
            }
            canvas { id: "{id}", onmounted: setup_chart }
        }
    }
}

#[component]
pub fn ElectrsMetrics(interval: Signal<i64>) -> Element {
    let mut show_status = use_signal(|| true);
    let mut show_blocks = use_signal(|| true);

    let electrs_up_data = use_resource(move || async move {
        let url = format!(
            "http://localhost:{}/metrics/electrs/up_status?minutes={}",
            env!("API_PORT"),
            interval()
        );
        Request::get(&url)
            .send()
            .await
            .ok()?
            .json::<Vec<MetricValue>>()
            .await
            .ok()
    });

    let electrs_indexed_data = use_resource(move || async move {
        let url = format!(
            "http://localhost:{}/metrics/electrs/indexed_blocks?minutes={}",
            env!("API_PORT"),
            interval()
        );
        Request::get(&url)
            .send()
            .await
            .ok()?
            .json::<Vec<MetricValue>>()
            .await
            .ok()
    });

    rsx! {
        div {
            style: "margin: 30px 0; padding: 20px; border: 1px solid #e74c3c; border-radius: 12px; background: linear-gradient(135deg, #1a1a1a 0%, #2a0a0a 100%);",
            h2 {
                style: "color: #e74c3c; margin: 0 0 20px 0; font-size: 1.5rem;",
                "⚡ Electrs Bitcoin Indexer Metrics"
            }

            div {
                h3 {
                    style: "color: #fff; margin: 0 0 10px 0;",
                    "Show Charts"
                }
                div {
                    style: "display: flex; gap: 20px; flex-wrap: wrap;",
                    label {
                        style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            checked: show_status(),
                            onchange: move |evt| show_status.set(evt.checked())
                        }
                        span { "Service Status" }
                    }
                    label {
                        style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            checked: show_blocks(),
                            onchange: move |evt| show_blocks.set(evt.checked())
                        }
                        span { "Indexed Blocks" }
                    }
                }
            }

            div {
                style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(400px, 1fr)); gap: 20px;",
                if show_status() {
                    match electrs_up_data.read().as_ref() {
                        Some(Some(data)) if !data.is_empty() => rsx! {
                            MetricChart {
                                id: "electrs-up".to_string(),
                                title: "Service Status (1=Up, 0=Down)".to_string(),
                                data: data.clone(),
                                color: "rgb(231, 76, 60)".to_string(),
                                y_begin_at_zero: true
                            }
                        },
                        _ => rsx! { div { style: "padding: 40px; text-align: center; color: #888;", "Loading status data..." } }
                    }
                }

                if show_blocks() {
                    match electrs_indexed_data.read().as_ref() {
                        Some(Some(data)) if !data.is_empty() => rsx! {
                            MetricChart {
                                id: "electrs-blocks".to_string(),
                                title: "Indexed Blocks".to_string(),
                                data: data.clone(),
                                color: "rgb(255, 99, 132)".to_string(),
                                y_begin_at_zero: false
                            }
                        },
                        _ => rsx! { div { style: "padding: 40px; text-align: center; color: #888;", "Loading block index data..." } }
                    }
                }
            }
        }
    }
}
