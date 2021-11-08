//! Component to handle app-global errors

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use gloo::timers::callback::Timeout;
use sycamore::prelude::*;

use crate::common::{ErrMsg, MpscChannel};

/// Internal identifier for one error message
type Id = i32;

/// Number of milliseconds after which an error message is automatically
/// removed.
const AUTO_REMOVAL_TIME: u32 = 10_000;

/// Component to display any app-global errors. They can be removed either with
/// a click, or otherwise automatically disappear after [AUTO_REMOVAL_TIME].
#[component(Errors<G>)]
pub fn errors(error_chan: MpscChannel<ErrMsg>) -> Template<G> {
    // The internal data structure is a [BTreeMap] from [Id]s to [ErrMsg]s. The
    // [Id] enables individual error messages to be deleted, and using a
    // [BTreeMap] rather than a HashMap means that iterating the structure
    // preserves insertion order, provied the [Id] keys are monotonically
    // increasing over time.
    let errs: Signal<RefCell<BTreeMap<Id, ErrMsg>>> = Default::default();
    let removals: MpscChannel<Id> = Default::default();
    let next_id: Rc<RefCell<Id>> = Default::default();

    // One "reactive event loop": add new errors when requested.
    error_chan.recv_loop({
        clone_all!(errs, removals);
        move |new_err| {
            let mut id = next_id.borrow_mut();
            *id += 1;
            let id = *id;
            errs.get_untracked().borrow_mut().insert(id, new_err);
            errs.trigger_subscribers();

            // Set a timer to auto-delete the error
            clone_all!(removals);
            Timeout::new(AUTO_REMOVAL_TIME, move || removals.send(id)).forget();
        }
    });

    // Another "reative event loop": remove errors when requested.
    removals.recv_loop({
        clone_all!(errs);
        move |id| {
            errs.get_untracked().borrow_mut().remove(&id);
            errs.trigger_subscribers();
        }
    });

    // View
    template! {
        div(class="errors") {(
            Template::new_fragment({
                errs.get().borrow().iter()
                    .map(|(&id, msg)| {
                        clone_all!(removals, msg);
                        template! {
                            div(class="alert alert-danger") {
                                ( msg )
                                button(class="close", on:click=move |_| removals.send(id))
                                    { "×" }
                            }
                        }
                    })
                    .collect()
            })
        )}
    }
}
