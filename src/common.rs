//! Basic functionality used throughout app.

use std::cell::RefCell;

use kstring::KString;
use sycamore::prelude::{create_effect, Signal};

/// A person's first name. Since these are typically short, using a more
/// space-efficient String variant is a win.
pub type Name = KString;

/// An error message to display.
pub type ErrMsg = String;

//////////////////////////////////////////////////////////////////////////////
///
/// Representation of the result of an API request.
#[derive(Clone, Debug)]
pub enum Remote<T> {
    Loading,
    Error,
    Success(T),
}

// We can't just derive this due to
// https://github.com/rust-lang/rust/issues/26925
impl<T> Default for Remote<T> {
    fn default() -> Self { Remote::Loading }
}

//////////////////////////////////////////////////////////////////////////////
///
/// A reactive, multi-producer single-consumer channel.
/// Subscribers are triggered synchronously with each send.
#[derive(Clone)]
pub struct MpscChannel<T: 'static>(Signal<RefCell<Option<T>>>);

// We can't just derive this due to
// https://github.com/rust-lang/rust/issues/26925
impl<T> Default for MpscChannel<T> {
    fn default() -> Self { MpscChannel(Signal::default()) }
}

impl<T> MpscChannel<T> {
    /// Send an item to the channel's consumers. Can be invoked from as many
    /// places as necessary. Consumers are triggered synchronously.
    pub fn send<IT: Into<T>>(&self, item: IT) {
        self.0.get_untracked().borrow_mut().replace(item.into());
        self.0.trigger_subscribers();
    }

    /// Spawn a new reactive context, that listens in a loop for any new
    /// messages on this channel, and runs the specified action each time.
    /// Each channel can have exactly one of these.
    pub fn recv_loop(&self, mut action: impl FnMut(T) + 'static) {
        let chan = self.0.clone();
        create_effect(move || {
            if let Some(item) = chan.get().take() {
                action(item);
            }
        })
    }
}

//////////////////////////////////////////////////////////////////////////////
///
/// Macro (stolen from <https://github.com/rust-lang/rfcs/issues/2407)> to clone
/// one or more items - preferred over [sycamore::prelude::cloned] since this
/// doesn't intefere with rustfmt.
#[macro_export]
macro_rules! clone_all {
    ($($i:ident),+) => {
        $(let $i = $i.clone();)+
    }
}
