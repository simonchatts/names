//! Basic functionality used throughout app.

// Re-export external stuff that we use almost everywhere
pub use dioxus::core::to_owned;
pub use dioxus::events::MouseEvent;
pub use dioxus::prelude::*;
pub use gloo::timers::future::TimeoutFuture;
pub use im_rc::HashMap;
pub use kstring::KString;
pub use wasm_bindgen_futures::{spawn_local, JsFuture};

// Re-export internal stuff that we use almost everywhere
pub use crate::api::*;
pub use crate::component::*;

/// A person's first name. Since these are typically short, using a more
/// space-efficient String variant is a win.
pub type Name = KString;

/// An error message to display.
pub type ErrMsg = String;

/// List of names
pub type Names = Vec<Name>;

/// Database of cached or in-flight API results
pub type Db = HashMap<Name, AllResults>;

/// Representation of the result of an API request.
#[derive(Clone, Debug)]
pub enum Remote<T> {
    Loading,
    Error,
    Success(T),
}

/// Result of both gender and country API requests for one name
#[derive(Default, Clone, Debug)]
pub struct AllResults {
    pub gender: Remote<GenderResult>,
    pub country: Remote<Vec<CountryResult>>,
}

// We can't just derive this due to
// https://github.com/rust-lang/rust/issues/26925
impl<T> Default for Remote<T> {
    fn default() -> Self { Remote::Loading }
}
