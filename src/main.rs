//! App presenting a simple UI around genderize.io and nationalize.io
#![warn(unused_crate_dependencies, rust_2018_idioms, missing_docs, unused)]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc<'_> = wee_alloc::WeeAlloc::INIT;

use gloo::timers::callback::Timeout;
use kstring::KString;
use sycamore::prelude::*;
use unidecode::unidecode;
use wasm_bindgen::JsCast;
use web_sys::{HtmlDocument, HtmlTextAreaElement};

#[macro_use]
mod common;
mod api;
mod db;
mod errors;
mod iso3166;
mod view;

use common::*;
use db::Db;

/// Name to ignore, so if this is the column title in a source spreadsheet,
/// where the entire column is pasted into the input textarea, we strip it out
/// of the query. Sorry if your name is "First".
const IGNORED_NAME: &str = "First";

/// Main app component
#[component(App<G>)]
fn app() -> View<G> {
    let ta_value: Signal<String> = Default::default();
    let hidden_ta: NodeRef<G> = Default::default();
    let names: Signal<Vec<Name>> = Default::default();
    let copied_label: Signal<Option<u32>> = Default::default();
    let error_chan: MpscChannel<ErrMsg> = Default::default();
    let db: Db = Default::default();
    let html_doc = html_doc();

    // Closure to invoke when the "copy M/F column" button is pressed.
    let copy_mf_col = {
        clone_all!(copied_label, db, hidden_ta, names);
        move |_| {
            let num_copied = copy_mf_column(
                hidden_ta.clone(),
                names.get().as_ref(),
                db.clone(),
                &html_doc,
            );
            // Display the label indicating the copy happened.
            copied_label.set(Some(num_copied));
        }
    };

    // Reactive context to auto-delete the "copied to clipboard" 3 seconds after
    // it is displayed. The button is diabled during this time to avoid races.
    create_effect({
        clone_all!(copied_label);
        move || {
            if copied_label.get().is_some() {
                clone_all!(copied_label);
                Timeout::new(3_000, move || copied_label.set(None)).forget();
            }
        }
    });

    // Derived state: disable the copy button if there is either nothing to
    // copy, or we are displaying the label indicating a copy just happened.
    let copy_button_disabled = create_memo({
        clone_all!(copied_label, names);
        move || names.get().is_empty() || copied_label.get().is_some()
    });

    // Reactive context that monitors the textarea changes:
    // - parse into a vec of names
    // - request an API operation for any new ones
    // - update the signal used to render the results table
    create_effect({
        clone_all!(db, error_chan, names, ta_value);
        move || {
            // Collect a vec of all names that haven't been presented to the API
            // before, since we last reloaded.
            let ta = &*ta_value.get();
            let names_vec: Vec<_> = ta
                .split('\n')
                .map(str::trim)
                .filter(|&s| !s.is_empty() && s != IGNORED_NAME)
                .map(normalise_name)
                .collect();

            // The order matters here to simplify downstream code: by first
            // creating Remote::Loading states for all new news, and then
            // inserting the names in the signal used by the view, the view can
            // assume the Remote status is unconditionally present.
            db.start_any_requests(&names_vec, error_chan.clone());
            names.set(names_vec);
        }
    });

    // Render the view
    view::render(
        db,
        error_chan,
        names.handle(),
        ta_value,
        hidden_ta,
        copied_label.handle(),
        copy_button_disabled,
        copy_mf_col,
    )
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

/// Assemble a plain text version of the M/F column, and copy it to the
/// clipboard, ready for pasting into an Excel column. The copy goes via a
/// hidden textarea.
fn copy_mf_column<G: Html>(
    hidden_ta: NodeRef<G>,
    names: &[Name],
    db: Db,
    html_doc: &HtmlDocument,
) -> u32 {
    // The hidden textarea element.
    let hta: HtmlTextAreaElement = hidden_ta.get::<DomNode>().unchecked_into();

    // Assemble the plain text string.
    let mut num_rows = 0;
    let mf_col = names
        .iter()
        .map(|n| {
            num_rows += 1;
            match db.0.borrow().get(n).unwrap().gender.get().as_ref() {
                Remote::Success(gender) => gender.summarised(),
                _ => "?",
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Set the hiddent text area to this string, and copy it to clipboard.
    hta.set_value(&mf_col);
    hta.select();
    html_doc.exec_command("copy").unwrap();

    // Return the number of rows copied
    num_rows
}

/// Get the window's HtmlDocument
fn html_doc() -> HtmlDocument {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .dyn_into::<web_sys::HtmlDocument>()
        .unwrap()
}

/// Start up the editor web app
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|| view! { App() });
}
