//! Everything that is pure presentation / visuals

use sycamore::prelude::*;

use crate::api::Gender;
use crate::common::*;
use crate::db::Db;
use crate::errors::Errors;
use crate::iso3166;

/// Percent probability threshold for an unconditional M or F
const CERTAIN_THRESHOLD: f32 = 85.0;

/// Percent probability threshold for a probable M or F
const PROBABLE_THRESHOLD: f32 = 75.0;

/// Top-level render function
pub fn render<G: Html>(
    db: Db,
    error_chan: MpscChannel<ErrMsg>,
    names: ReadSignal<Vec<Name>>,
    ta_value: Signal<String>,
    hidden_ta: NodeRef<G>,
    copied_label: ReadSignal<Option<u32>>,
    copy_button_disabled: ReadSignal<bool>,
    copy_mf_col: impl Fn(web_sys::Event) + 'static,
) -> View<G> {
    // View
    view! {
        div(class="container-xl") {
            div(class="navbar navbar-dark bg-info") {
                div(class="container-fluid") {
                  img(src="/badge.png", alt="logo")
                  div(class="navbar-brand mb-0 h1")
                    { "First Name Probabilistic Analysis" }
                }
              }
            Errors(error_chan)
            div(class="copy-button") {
                (if let Some(num_rows) = *copied_label.get() {
                    view! {
                        span(class="copy-label") {(
                            format!("✓ Copied {} rows to clipboard", num_rows)
                        )}
                    }
                } else {
                    view! {}
                })
                button(class="btn btn-outline-primary btn-sm",
                       disabled=*copy_button_disabled.get(),
                       on:click=copy_mf_col) {
                    "Copy M/F column to clipboard"
                }
            }
            h4 {
                span(class="arrow") { "⤹" }
                "Enter or paste first names into this box"
            }
            main() {
                // No need to auto-expand rows here, because the flexbox layout
                // means every non-empty row gets a table row, which is taller,
                // so in practice the textarea is always stretched to be taller
                // than its raw contents by the adjacent table.
                textarea(bind:value=ta_value, autofocus=true, cols=15,
                         placeholder="First names...")

                // Render the results table.
                table(class="table table-sm table-bordered") {
                    tbody {
                        Indexed(IndexedProps {
                            iterable: names,
                            template: move |name| table_row(name, db.clone())
                        })
                    }
                }
            }
        }
        footnote() {
            p() {
                "This is a direct interface over the "
                a(href="https://genderize.io") { "genderize.io" }
                " and "
                a(href="https://nationalize.io") { "nationalize.io" }
                " API services."
                }
            p() {
                "These have a free quota of 1000 name queries per day (per IP address)."
            }
            // Just for copying text, but keep it out the way anyway
            textarea(id="hidden-textarea", ref=hidden_ta)
        }
    }
}

impl crate::api::GenderResult {
    /// Plain-text representation of a gender result.
    pub fn summarised(&self) -> &'static str {
        let prob = f32::round(self.probability * 100.0);
        match (self.gender, prob >= CERTAIN_THRESHOLD, prob >= PROBABLE_THRESHOLD) {
            (Some(Gender::Female), true, _) => "F",
            (Some(Gender::Female), _, true) => "F?",
            (Some(Gender::Female), _, _) => "F??",
            (Some(Gender::Male), true, _) => "M",
            (Some(Gender::Male), _, true) => "M?",
            (Some(Gender::Male), _, _) => "M??",
            _ => "?",
        }
    }
}

/// View for one row of the results table
fn table_row<G: Html>(name: Name, db: Db) -> View<G> {
    let db = db.0.borrow();
    let api_value = db.get(&name).unwrap();
    let gender = api_value.gender.clone();
    let country = api_value.country.clone();
    view! {
        tr {
            td {( name )}
                (gender.get().as_ref().render(|r| {
                let prob = r.probability;
                let label = r.summarised();
                view! {
                    td {
                        ( confidence_bar(prob) )
                        ( label )
                    }
                }
            }))
            (country.get().as_ref().render(|v| {
                let spans = View::new_fragment(v.iter().map(|c| {
                    let country = iso3166::lookup(&c.country).unwrap_or(&c.country).to_owned();
                    let prob = c.probability;
                    view! {
                        div(class="country") {
                            ( confidence_bar(prob) )
                            ( country )
                        }
                    }
                }).collect());
                view! {
                    td {
                        div(class="countries") {( spans )}
                    }
                }
            }))
        }
    }
}

/// Render a confidence/probability bar.
fn confidence_bar<G: Html>(probability: f32) -> View<G> {
    let prob = f32::round(probability * 100.0) as u8;
    view! {
        div(class="confidence", style=format!("width: {}%", prob)) {(
            if prob > 0 {
                format!("{}%", prob)
            } else {
                format!("") }
        )}
    }
}

impl<T: Clone> Remote<T> {
    /// Render a [Remote] value.
    fn render<G: Html>(&self, render: impl Fn(&T) -> View<G>) -> View<G> {
        match self {
            Remote::Loading => view! {
                td {
                    div(class="progress") {
                        div(class="progress-bar progress-bar-striped progress-bar-animated",
                            style="width: 100%;")
                    }
                }
            },
            Remote::Error => view! {
                td {
                    span(class="badge badge-pill badge-danger px-5 py-1") {
                        "⚠ Error"
                    }
                }
            },
            Remote::Success(r) => render(r),
        }
    }
}
