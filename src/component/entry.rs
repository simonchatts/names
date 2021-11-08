//! Text entry box

use unidecode::unidecode;

use crate::prelude::*;

/// Name to ignore, so if this is the column title in a source spreadsheet,
/// where the entire column is pasted into the input textarea, we strip it out
/// of the query. Sorry if your name is "First".
const IGNORED_NAME: &str = "First";

// @@@ /// doesn't work here

// Entry component: textarea for entering of names, and update a vec of ASCII
// names in response.
#[tracing::instrument(skip_all)]
#[inline_props]
pub fn Entry(cx: Scope, names: UseState<Names>) -> Element<'_> {
    // Handle updates
    let oninput = |ev: dioxus::events::FormEvent| {
        let names_vec: Vec<_> = ev
            .value
            .split('\n')
            .map(str::trim)
            .filter(|&s| !s.is_empty() && s != IGNORED_NAME)
            .map(normalise_name)
            .collect();
        names.set(names_vec);
    };

    cx.render(rsx! {
        // No need to auto-expand rows here, because the flexbox layout
        // means every non-empty row gets a table row, which is taller,
        // so in practice the textarea is always stretched to be taller
        // than its raw contents by the adjacent table.
        textarea {
            autofocus: "true", cols: "15", placeholder: "First names...",
            oninput: oninput
        }
    })
}

/// Normalise a name to a single ASCII word (all the API seems to handle).
fn normalise_name(name: &str) -> Name {
    let ascii_name = unidecode(name);
    let first_word = match ascii_name.split_once(|c: char| !c.is_alphabetic()) {
        Some((first_word, _rest)) => first_word.to_owned(),
        None => ascii_name,
    };
    KString::from_string(first_word)
}
