//! Render the results table

use crate::prelude::*;

/// Percent probability threshold for an unconditional M or F
const CERTAIN_THRESHOLD: f32 = 85.0;

/// Percent probability threshold for a probable M or F
const PROBABLE_THRESHOLD: f32 = 75.0;

// Component to display results table
#[tracing::instrument(skip_all)]
#[inline_props]
pub fn Table(cx: Scope<'_>, names: UseState<Names>, db: UseRef<Db>) -> Element<'_> {
    let table_row = |i, name| {
        let db = db.read();
        rsx! {
            tr {
                key: "{i}-{name}",
                td { "{name}" }

                // Gender
                db.get(name).unwrap().gender.render(|r| {
                    let label = r.summarised();
                    rsx! {
                        td {
                            ConfidenceBar { probability: r.probability }
                            "{label}"
                        }
                    }
                })

                // Countries
                db.get(name).unwrap().country.render(|r| {
                    rsx! {
                        td {
                            div {
                                class: "countries",
                                r.iter().enumerate().map(|(n, c)| {
                                    let country = crate::iso3166::lookup(&c.country).unwrap_or(&c.country);
                                    rsx! {
                                        div {
                                            key: "{n}",
                                            class: "country",
                                            ConfidenceBar { probability: c.probability }
                                            "{country}"
                                        }
                                    }
                                })
                            }
                        }
                    }
                })
            }
        }
    };
    cx.render(rsx! {
        table {
            class: "table table-sm table-bordered",
            names.iter().enumerate().map(|(i, name)| {
                table_row(i, name)
            })
        }
    })
}

// Render a confidence bar and label, for a given probability
#[inline_props]
fn ConfidenceBar(cx: Scope<'_>, probability: f32) -> Element<'_> {
    let prob = f32::round(probability * 100.0) as u8;
    let prob_str = if prob > 0 { format!("{prob}%") } else { String::from("") };
    cx.render(rsx! {
        div {
            class: "confidence",
            style: "width: {prob}%",
            "{prob_str}"
        }
    })
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

impl<T> Remote<T> {
    /// Render a [Remote<T>], given a closure for the success case
    fn render<'a>(&self, render: impl Fn(&T) -> LazyNodes<'a, '_>) -> LazyNodes<'a, '_> {
        match self {
            Remote::Loading => rsx! {
                td {
                    div {
                        class: "progress",
                        div {
                            class: "progress-bar progress-bar-striped progress-bar-animated",
                            style: "width: 100%;"
                        }
                    }
                }
            },
            Remote::Error => rsx! {
                td {
                    span {
                        class: "badge badge-pill badge-danger px-5 py-1",
                        "⚠ Error"
                    }
                }
            },
            Remote::Success(r) => render(r),
        }
    }
}
