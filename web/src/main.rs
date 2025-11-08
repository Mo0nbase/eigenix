use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        div {
            class: "container",
            h1 { "Eigenix" }
            p { "A Dioxus + Axum application" }

            div {
                class: "counter",
                h2 { "Counter: {count}" }
                button {
                    onclick: move |_| count += 1,
                    "Increment"
                }
                button {
                    onclick: move |_| count -= 1,
                    "Decrement"
                }
            }
        }
    }
}
