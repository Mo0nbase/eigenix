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
pub fn MoneroMetrics(interval: Signal<i64>) -> Element {
    let mut show_height = use_signal(|| true);
    let mut show_target_height = use_signal(|| true);
    let mut show_difficulty = use_signal(|| true);
    let mut show_tx_count = use_signal(|| true);
    let mut show_balance = use_signal(|| true);

    let xmr_height_data = use_resource(move || async move {
        let url = format!(
            "http://localhost:1235/metrics/monero/interval?minutes={}",
            interval()
        );
        let response = Request::get(&url).send().await.ok()?;
        let data: Vec<serde_json::Value> = response.json().await.ok()?;
        Some(
            data.into_iter()
                .map(|item| MetricValue {
                    timestamp: item["timestamp"].as_str().unwrap_or("").to_string(),
                    value: item["height"].as_u64().unwrap_or(0) as f64,
                })
                .collect::<Vec<_>>(),
        )
    });

    let xmr_target_height_data = use_resource(move || async move {
        let url = format!(
            "http://localhost:1235/metrics/monero/interval?minutes={}",
            interval()
        );
        let response = Request::get(&url).send().await.ok()?;
        let data: Vec<serde_json::Value> = response.json().await.ok()?;
        Some(
            data.into_iter()
                .map(|item| MetricValue {
                    timestamp: item["timestamp"].as_str().unwrap_or("").to_string(),
                    value: item["target_height"].as_u64().unwrap_or(0) as f64,
                })
                .collect::<Vec<_>>(),
        )
    });

    let xmr_difficulty_data = use_resource(move || async move {
        let url = format!(
            "http://localhost:1235/metrics/monero/interval?minutes={}",
            interval()
        );
        let response = Request::get(&url).send().await.ok()?;
        let data: Vec<serde_json::Value> = response.json().await.ok()?;
        Some(
            data.into_iter()
                .map(|item| MetricValue {
                    timestamp: item["timestamp"].as_str().unwrap_or("").to_string(),
                    value: item["difficulty"].as_u64().unwrap_or(0) as f64,
                })
                .collect::<Vec<_>>(),
        )
    });

    let xmr_tx_count_data = use_resource(move || async move {
        let url = format!(
            "http://localhost:1235/metrics/monero/interval?minutes={}",
            interval()
        );
        let response = Request::get(&url).send().await.ok()?;
        let data: Vec<serde_json::Value> = response.json().await.ok()?;
        Some(
            data.into_iter()
                .map(|item| MetricValue {
                    timestamp: item["timestamp"].as_str().unwrap_or("").to_string(),
                    value: item["tx_count"].as_u64().unwrap_or(0) as f64,
                })
                .collect::<Vec<_>>(),
        )
    });

    let xmr_balance_data = use_resource(move || async move {
        let url = format!(
            "http://localhost:1235/metrics/monero/interval?minutes={}",
            interval()
        );
        let response = Request::get(&url).send().await.ok()?;
        let data: Vec<serde_json::Value> = response.json().await.ok()?;
        Some(
            data.into_iter()
                .filter_map(|item| {
                    item["wallet_balance"].as_f64().map(|balance| MetricValue {
                        timestamp: item["timestamp"].as_str().unwrap_or("").to_string(),
                        value: balance,
                    })
                })
                .collect::<Vec<_>>(),
        )
    });

    rsx! {
        div {
            style: "margin: 30px 0; padding: 20px; border: 1px solid #ff6600; border-radius: 12px; background: linear-gradient(135deg, #1a1a1a 0%, #2a1a0a 100%);",
            h2 {
                style: "color: #ff6600; margin: 0 0 20px 0; font-size: 1.5rem;",
                "🟧 Monero Node Metrics"
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
                            checked: show_height(),
                            onchange: move |evt| show_height.set(evt.checked())
                        }
                        span { "Block Height" }
                    }
                    label {
                        style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            checked: show_target_height(),
                            onchange: move |evt| show_target_height.set(evt.checked())
                        }
                        span { "Target Height" }
                    }
                    label {
                        style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            checked: show_difficulty(),
                            onchange: move |evt| show_difficulty.set(evt.checked())
                        }
                        span { "Network Difficulty" }
                    }
                    label {
                        style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            checked: show_tx_count(),
                            onchange: move |evt| show_tx_count.set(evt.checked())
                        }
                        span { "Transaction Count" }
                    }
                    label {
                        style: "display: flex; align-items: center; gap: 8px; color: #ccc; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            checked: show_balance(),
                            onchange: move |evt| show_balance.set(evt.checked())
                        }
                        span { "Wallet Balance" }
                    }
                }
            }

            div {
                style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(400px, 1fr)); gap: 20px;",
                if show_height() {
                    match xmr_height_data.read().as_ref() {
                        Some(Some(data)) if !data.is_empty() => rsx! {
                            MetricChart {
                                id: "xmr-height".to_string(),
                                title: "Block Height".to_string(),
                                data: data.clone(),
                                color: "rgb(255, 102, 0)".to_string(),
                                y_begin_at_zero: false
                            }
                        },
                        _ => rsx! { div { style: "padding: 40px; text-align: center; color: #888;", "Loading height data..." } }
                    }
                }

                if show_target_height() {
                    match xmr_target_height_data.read().as_ref() {
                        Some(Some(data)) if !data.is_empty() => rsx! {
                            MetricChart {
                                id: "xmr-target-height".to_string(),
                                title: "Target Height".to_string(),
                                data: data.clone(),
                                color: "rgb(255, 159, 64)".to_string(),
                                y_begin_at_zero: false
                            }
                        },
                        _ => rsx! { div { style: "padding: 40px; text-align: center; color: #888;", "Loading target height data..." } }
                    }
                }

                if show_difficulty() {
                    match xmr_difficulty_data.read().as_ref() {
                        Some(Some(data)) if !data.is_empty() => rsx! {
                            MetricChart {
                                id: "xmr-difficulty".to_string(),
                                title: "Network Difficulty".to_string(),
                                data: data.clone(),
                                color: "rgb(75, 192, 192)".to_string(),
                                y_begin_at_zero: true
                            }
                        },
                        _ => rsx! { div { style: "padding: 40px; text-align: center; color: #888;", "Loading difficulty data..." } }
                    }
                }

                if show_tx_count() {
                    match xmr_tx_count_data.read().as_ref() {
                        Some(Some(data)) if !data.is_empty() => rsx! {
                            MetricChart {
                                id: "xmr-tx-count".to_string(),
                                title: "Transaction Count".to_string(),
                                data: data.clone(),
                                color: "rgb(153, 102, 255)".to_string(),
                                y_begin_at_zero: true
                            }
                        },
                        _ => rsx! { div { style: "padding: 40px; text-align: center; color: #888;", "Loading transaction count data..." } }
                    }
                }

                if show_balance() {
                    match xmr_balance_data.read().as_ref() {
                        Some(Some(data)) if !data.is_empty() => rsx! {
                            MetricChart {
                                id: "xmr-balance".to_string(),
                                title: "Wallet Balance (XMR)".to_string(),
                                data: data.clone(),
                                color: "rgb(255, 206, 86)".to_string(),
                                y_begin_at_zero: true
                            }
                        },
                        _ => rsx! { div { style: "padding: 40px; text-align: center; color: #888;", "Loading balance data..." } }
                    }
                }
            }
        }
    }
}
