//! App presenting a simple UI around genderize.io and nationalize.io
#![warn(rust_2018_idioms, missing_docs, unused)]
#![feature(once_cell)]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc<'_> = wee_alloc::WeeAlloc::INIT;

mod api;
mod component;
mod db;
mod iso3166;
mod prelude;

use prelude::*;

/// Main app component
#[tracing::instrument(skip_all)]
fn app(cx: Scope<'_>) -> Element<'_> {
    // The [Names] entered by the user
    let names = use_state(&cx, Names::default);

    // The [Db] contained all pending and cached API results
    let db = use_ref(&cx, Db::new);
    db::start_any_requests(names, db.clone());

    // Top-level view
    cx.render(rsx! {
        div {
            class: "container-xl",
            div {
                class: "navbar navbar-dark bg-info",
                img { src: "/badge.png", alt: "logo" }
                div {
                    class: "navbar-brand mb-0 h1",
                    "First Name Probabilistic Analysis"
                }
            }
            Errors {}
            CopyButton { names: names.clone(), db: db.clone() }
            h4 {
                span { class: "arrow", "⤹" }
                "Enter or paste first names into this box"
            }
            main {
                Entry { names: names.clone() }
                Table { names: names.clone(), db: db.clone() }
            }
        }
        footer {
            p {
                "This is a direct interface over the "
                a { href: "https://genderize.io", "genderize.io" }
                " and "
                a { href: "https://nationalize.io", "nationalize.io" }
                " API services."
            }
            p {
                "These have a free quota of 1000 name queries per day (per IP address)."
            }
        }
    })
}

/// Start up the editor web app
pub fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    dioxus::web::launch(app);
}
